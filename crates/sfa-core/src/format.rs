use std::io::{Read, Write};

use bitflags::bitflags;

use crate::codec;
use crate::config::{
    DataCodec, FrameHashAlgo, IntegrityMode, ManifestCodec, ManifestHashAlgo, PackConfig,
};
use crate::integrity;
use crate::model::{BundleKind, BundlePlanRecord, EntryKind, EntryRecord, ExtentRecord, Manifest};
use crate::{Error, Result};

pub const HEADER_LEN: usize = 128;
pub const MANIFEST_HEADER_LEN: usize = 64;
pub const FRAME_HEADER_LEN: usize = 48;
pub const TRAILER_LEN: usize = 64;

const HEADER_MAGIC: &[u8; 8] = b"SFA\0\r\n\x1A\n";
const MANIFEST_MAGIC: &[u8; 4] = b"MFST";
const FRAME_MAGIC: &[u8; 4] = b"FRME";
const TRAILER_MAGIC: &[u8; 4] = b"TRLR";

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FeatureFlags: u64 {
        const HAS_SYMLINK = 1 << 0;
        const HAS_HARDLINK = 1 << 1;
        const HAS_SPECIAL_FILE = 1 << 2;
        const HAS_METADATA = 1 << 3;
        const HAS_TRAILER = 1 << 4;
        const PRESERVE_OWNER = 1 << 5;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderV1 {
    pub version_major: u16,
    pub version_minor: u16,
    pub data_codec: DataCodec,
    pub manifest_codec: ManifestCodec,
    pub integrity_mode: IntegrityMode,
    pub frame_hash_algo: FrameHashAlgo,
    pub manifest_hash_algo: ManifestHashAlgo,
    pub suggested_parallelism: u16,
    pub bundle_target_bytes: u32,
    pub small_file_threshold: u32,
    pub entry_count: u64,
    pub extent_count: u64,
    pub bundle_count: u64,
    pub manifest_raw_len: u64,
    pub manifest_encoded_len: u64,
    pub feature_flags: FeatureFlags,
    pub manifest_hash: [u8; 32],
    pub writer_version_major: u16,
    pub writer_version_minor: u16,
}

impl HeaderV1 {
    pub fn from_manifest(
        manifest: &Manifest,
        config: &PackConfig,
        manifest_encoded_len: usize,
        manifest_hash: [u8; 32],
    ) -> Self {
        Self {
            version_major: 1,
            version_minor: 0,
            data_codec: config.codec,
            manifest_codec: config.manifest_codec,
            integrity_mode: config.integrity,
            frame_hash_algo: FrameHashAlgo::Xxh3_64,
            manifest_hash_algo: ManifestHashAlgo::Blake3_256,
            suggested_parallelism: config.threads.min(u16::MAX as usize) as u16,
            bundle_target_bytes: config.bundle_target_bytes,
            small_file_threshold: config.small_file_threshold,
            entry_count: manifest.entry_count(),
            extent_count: manifest.extent_count(),
            bundle_count: manifest.bundle_count(),
            manifest_raw_len: manifest.raw_len() as u64,
            manifest_encoded_len: manifest_encoded_len as u64,
            feature_flags: feature_flags_from_manifest(manifest, config),
            manifest_hash,
            writer_version_major: 0,
            writer_version_minor: 1,
        }
    }

    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let mut out = [0u8; HEADER_LEN];
        out[0..8].copy_from_slice(HEADER_MAGIC);
        out[8..10].copy_from_slice(&(HEADER_LEN as u16).to_le_bytes());
        out[10..12].copy_from_slice(&self.version_major.to_le_bytes());
        out[12..14].copy_from_slice(&self.version_minor.to_le_bytes());
        out[14..16].copy_from_slice(&(self.data_codec as u16).to_le_bytes());
        out[16..18].copy_from_slice(&(self.manifest_codec as u16).to_le_bytes());
        out[18] = self.integrity_mode as u8;
        out[19] = self.frame_hash_algo as u8;
        out[20] = self.manifest_hash_algo as u8;
        out[22..24].copy_from_slice(&self.suggested_parallelism.to_le_bytes());
        out[24..28].copy_from_slice(&self.bundle_target_bytes.to_le_bytes());
        out[28..32].copy_from_slice(&self.small_file_threshold.to_le_bytes());
        out[32..40].copy_from_slice(&self.entry_count.to_le_bytes());
        out[40..48].copy_from_slice(&self.extent_count.to_le_bytes());
        out[48..56].copy_from_slice(&self.bundle_count.to_le_bytes());
        out[56..64].copy_from_slice(&self.manifest_raw_len.to_le_bytes());
        out[64..72].copy_from_slice(&self.manifest_encoded_len.to_le_bytes());
        out[72..80].copy_from_slice(&self.feature_flags.bits().to_le_bytes());
        out[80..112].copy_from_slice(&self.manifest_hash);
        out[116..118].copy_from_slice(&self.writer_version_major.to_le_bytes());
        out[118..120].copy_from_slice(&self.writer_version_minor.to_le_bytes());
        let crc = crc32fast::hash(&out);
        out[112..116].copy_from_slice(&crc.to_le_bytes());
        out
    }

    pub fn decode(bytes: [u8; HEADER_LEN]) -> Result<Self> {
        if &bytes[0..8] != HEADER_MAGIC {
            return Err(Error::InvalidHeader("bad magic"));
        }
        let header_len = u16::from_le_bytes([bytes[8], bytes[9]]);
        if header_len as usize != HEADER_LEN {
            return Err(Error::InvalidHeader("unexpected header length"));
        }
        let crc = u32::from_le_bytes(bytes[112..116].try_into().unwrap());
        let mut check = bytes;
        check[112..116].fill(0);
        if crc32fast::hash(&check) != crc {
            return Err(Error::InvalidHeader("crc mismatch"));
        }
        Ok(Self {
            version_major: u16::from_le_bytes(bytes[10..12].try_into().unwrap()),
            version_minor: u16::from_le_bytes(bytes[12..14].try_into().unwrap()),
            data_codec: DataCodec::from_u16(u16::from_le_bytes(bytes[14..16].try_into().unwrap()))?,
            manifest_codec: ManifestCodec::from_u16(u16::from_le_bytes(
                bytes[16..18].try_into().unwrap(),
            ))?,
            integrity_mode: IntegrityMode::from_u8(bytes[18])?,
            frame_hash_algo: FrameHashAlgo::from_u8(bytes[19])?,
            manifest_hash_algo: ManifestHashAlgo::from_u8(bytes[20])?,
            suggested_parallelism: u16::from_le_bytes(bytes[22..24].try_into().unwrap()),
            bundle_target_bytes: u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            small_file_threshold: u32::from_le_bytes(bytes[28..32].try_into().unwrap()),
            entry_count: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            extent_count: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            bundle_count: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
            manifest_raw_len: u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
            manifest_encoded_len: u64::from_le_bytes(bytes[64..72].try_into().unwrap()),
            feature_flags: FeatureFlags::from_bits_retain(u64::from_le_bytes(
                bytes[72..80].try_into().unwrap(),
            )),
            manifest_hash: bytes[80..112].try_into().unwrap(),
            writer_version_major: u16::from_le_bytes(bytes[116..118].try_into().unwrap()),
            writer_version_minor: u16::from_le_bytes(bytes[118..120].try_into().unwrap()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeaderV1 {
    pub bundle_id: u64,
    pub raw_len: u32,
    pub encoded_len: u32,
    pub frame_hash: u64,
    pub flags: u16,
}

impl FrameHeaderV1 {
    pub fn encode(&self) -> [u8; FRAME_HEADER_LEN] {
        let mut out = [0u8; FRAME_HEADER_LEN];
        out[0..4].copy_from_slice(FRAME_MAGIC);
        out[4..6].copy_from_slice(&(FRAME_HEADER_LEN as u16).to_le_bytes());
        out[6..8].copy_from_slice(&self.flags.to_le_bytes());
        out[8..16].copy_from_slice(&self.bundle_id.to_le_bytes());
        out[16..20].copy_from_slice(&self.raw_len.to_le_bytes());
        out[20..24].copy_from_slice(&self.encoded_len.to_le_bytes());
        out[24..32].copy_from_slice(&self.frame_hash.to_le_bytes());
        out
    }

    pub fn decode(bytes: [u8; FRAME_HEADER_LEN]) -> Result<Self> {
        if &bytes[0..4] != FRAME_MAGIC {
            return Err(Error::InvalidFrame("bad magic"));
        }
        let header_len = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        if header_len as usize != FRAME_HEADER_LEN {
            return Err(Error::InvalidFrame("unexpected header length"));
        }
        Ok(Self {
            flags: u16::from_le_bytes(bytes[6..8].try_into().unwrap()),
            bundle_id: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            raw_len: u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            encoded_len: u32::from_le_bytes(bytes[20..24].try_into().unwrap()),
            frame_hash: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrailerV1 {
    pub bundle_count: u64,
    pub total_raw_bytes: u64,
    pub total_encoded_bytes: u64,
    pub archive_hash: [u8; 32],
}

impl TrailerV1 {
    pub fn encode(&self) -> [u8; TRAILER_LEN] {
        let mut out = [0u8; TRAILER_LEN];
        out[0..4].copy_from_slice(TRAILER_MAGIC);
        out[4..6].copy_from_slice(&(TRAILER_LEN as u16).to_le_bytes());
        out[8..16].copy_from_slice(&self.bundle_count.to_le_bytes());
        out[16..24].copy_from_slice(&self.total_raw_bytes.to_le_bytes());
        out[24..32].copy_from_slice(&self.total_encoded_bytes.to_le_bytes());
        out[32..64].copy_from_slice(&self.archive_hash);
        out
    }

    pub fn decode(bytes: [u8; TRAILER_LEN]) -> Result<Self> {
        if &bytes[0..4] != TRAILER_MAGIC {
            return Err(Error::InvalidFrame("bad trailer magic"));
        }
        let len = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        if len as usize != TRAILER_LEN {
            return Err(Error::InvalidFrame("unexpected trailer length"));
        }
        Ok(Self {
            bundle_count: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            total_raw_bytes: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            total_encoded_bytes: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            archive_hash: bytes[32..64].try_into().unwrap(),
        })
    }
}

pub fn encode_manifest(
    manifest: &Manifest,
    codec_kind: ManifestCodec,
) -> Result<(Vec<u8>, [u8; 32])> {
    let raw = manifest_to_raw_bytes(manifest);
    let hash = integrity::manifest_hash(ManifestHashAlgo::Blake3_256, &raw);
    let encoded = codec::encode_manifest(codec_kind, &raw)?;
    Ok((encoded, hash))
}

pub fn decode_manifest(header: &HeaderV1, bytes: &[u8]) -> Result<Manifest> {
    let raw = codec::decode_manifest(
        header.manifest_codec,
        bytes,
        header.manifest_raw_len as usize,
    )?;
    let expected_hash = integrity::manifest_hash(header.manifest_hash_algo, &raw);
    if expected_hash != header.manifest_hash {
        return Err(Error::ManifestHashMismatch);
    }
    manifest_from_raw_bytes(&raw)
}

pub fn read_header<R: Read>(reader: &mut R) -> Result<HeaderV1> {
    let mut bytes = [0u8; HEADER_LEN];
    reader.read_exact(&mut bytes).map_err(Error::from)?;
    HeaderV1::decode(bytes)
}

pub fn write_header<W: Write>(writer: &mut W, header: &HeaderV1) -> Result<()> {
    writer.write_all(&header.encode()).map_err(Error::from)
}

fn feature_flags_from_manifest(manifest: &Manifest, config: &PackConfig) -> FeatureFlags {
    let mut flags = FeatureFlags::empty();
    if manifest
        .entries
        .iter()
        .any(|entry| matches!(entry.kind, EntryKind::Symlink))
    {
        flags |= FeatureFlags::HAS_SYMLINK;
    }
    if manifest
        .entries
        .iter()
        .any(|entry| matches!(entry.kind, EntryKind::Hardlink))
    {
        flags |= FeatureFlags::HAS_HARDLINK;
    }
    if manifest.entries.iter().any(|entry| {
        matches!(
            entry.kind,
            EntryKind::CharDevice | EntryKind::BlockDevice | EntryKind::Fifo
        )
    }) {
        flags |= FeatureFlags::HAS_SPECIAL_FILE;
    }
    if !manifest.meta_blob.is_empty() {
        flags |= FeatureFlags::HAS_METADATA;
    }
    if integrity::requires_trailer(config.integrity) {
        flags |= FeatureFlags::HAS_TRAILER;
    }
    if config.preserve_owner {
        flags |= FeatureFlags::PRESERVE_OWNER;
    }
    flags
}

fn manifest_to_raw_bytes(manifest: &Manifest) -> Vec<u8> {
    let mut out = Vec::with_capacity(manifest.raw_len());
    out.extend_from_slice(MANIFEST_MAGIC);
    out.extend_from_slice(&(MANIFEST_HEADER_LEN as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(manifest.entries.len() as u64).to_le_bytes());
    out.extend_from_slice(&(manifest.extents.len() as u64).to_le_bytes());
    out.extend_from_slice(&(manifest.bundles.len() as u64).to_le_bytes());
    out.extend_from_slice(&(manifest.name_arena.len() as u64).to_le_bytes());
    out.extend_from_slice(&(manifest.meta_blob.len() as u64).to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);

    for entry in &manifest.entries {
        out.extend_from_slice(&entry.parent_id.to_le_bytes());
        out.push(entry.kind as u8);
        out.push(entry.flags);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&entry.mode.to_le_bytes());
        out.extend_from_slice(&entry.uid.to_le_bytes());
        out.extend_from_slice(&entry.gid.to_le_bytes());
        out.extend_from_slice(&entry.mtime_sec.to_le_bytes());
        out.extend_from_slice(&entry.mtime_nsec.to_le_bytes());
        out.extend_from_slice(&entry.size.to_le_bytes());
        out.extend_from_slice(&entry.name_off.to_le_bytes());
        out.extend_from_slice(&entry.name_len.to_le_bytes());
        out.extend_from_slice(&entry.link_off.to_le_bytes());
        out.extend_from_slice(&entry.link_len.to_le_bytes());
        out.extend_from_slice(&entry.first_extent.to_le_bytes());
        out.extend_from_slice(&entry.extent_count.to_le_bytes());
        out.extend_from_slice(&entry.hardlink_master_entry_id.to_le_bytes());
        out.extend_from_slice(&entry.dev_major.to_le_bytes());
        out.extend_from_slice(&entry.dev_minor.to_le_bytes());
        out.extend_from_slice(&entry.meta_off.to_le_bytes());
        out.extend_from_slice(&entry.meta_len.to_le_bytes());
        out.extend_from_slice(&[0u8; 8]);
    }

    for extent in &manifest.extents {
        out.extend_from_slice(&extent.bundle_id.to_le_bytes());
        out.extend_from_slice(&extent.file_offset.to_le_bytes());
        out.extend_from_slice(&extent.raw_offset_in_bundle.to_le_bytes());
        out.extend_from_slice(&extent.raw_len.to_le_bytes());
        out.extend_from_slice(&extent.flags.to_le_bytes());
        out.extend_from_slice(&extent.entry_id.to_le_bytes());
    }

    for bundle in &manifest.bundles {
        out.extend_from_slice(&bundle.bundle_id.to_le_bytes());
        out.extend_from_slice(&bundle.raw_len.to_le_bytes());
        out.extend_from_slice(&bundle.file_count.to_le_bytes());
        out.extend_from_slice(&bundle.extent_count.to_le_bytes());
        out.push(bundle.kind as u8);
        out.push(bundle.flags);
        out.extend_from_slice(&[0u8; 10]);
    }

    out.extend_from_slice(&manifest.name_arena);
    out.extend_from_slice(&manifest.meta_blob);
    out
}

fn manifest_from_raw_bytes(raw: &[u8]) -> Result<Manifest> {
    if raw.len() < MANIFEST_HEADER_LEN {
        return Err(Error::InvalidManifest("manifest too short"));
    }
    if &raw[0..4] != MANIFEST_MAGIC {
        return Err(Error::InvalidManifest("bad manifest magic"));
    }
    let header_len = u16::from_le_bytes(raw[4..6].try_into().unwrap()) as usize;
    if header_len != MANIFEST_HEADER_LEN {
        return Err(Error::InvalidManifest("unexpected manifest header length"));
    }
    let entry_count = u64::from_le_bytes(raw[8..16].try_into().unwrap()) as usize;
    let extent_count = u64::from_le_bytes(raw[16..24].try_into().unwrap()) as usize;
    let bundle_count = u64::from_le_bytes(raw[24..32].try_into().unwrap()) as usize;
    let name_arena_bytes = u64::from_le_bytes(raw[32..40].try_into().unwrap()) as usize;
    let meta_blob_bytes = u64::from_le_bytes(raw[40..48].try_into().unwrap()) as usize;
    let expected_len = MANIFEST_HEADER_LEN
        + entry_count * 96
        + extent_count * 32
        + bundle_count * 32
        + name_arena_bytes
        + meta_blob_bytes;
    if raw.len() != expected_len {
        return Err(Error::InvalidManifest("manifest length mismatch"));
    }
    let mut offset = MANIFEST_HEADER_LEN;
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let chunk = &raw[offset..offset + 96];
        entries.push(EntryRecord {
            parent_id: u32::from_le_bytes(chunk[0..4].try_into().unwrap()),
            kind: EntryKind::from_u8(chunk[4])?,
            flags: chunk[5],
            mode: u32::from_le_bytes(chunk[8..12].try_into().unwrap()),
            uid: u32::from_le_bytes(chunk[12..16].try_into().unwrap()),
            gid: u32::from_le_bytes(chunk[16..20].try_into().unwrap()),
            mtime_sec: i64::from_le_bytes(chunk[20..28].try_into().unwrap()),
            mtime_nsec: u32::from_le_bytes(chunk[28..32].try_into().unwrap()),
            size: u64::from_le_bytes(chunk[32..40].try_into().unwrap()),
            name_off: u32::from_le_bytes(chunk[40..44].try_into().unwrap()),
            name_len: u32::from_le_bytes(chunk[44..48].try_into().unwrap()),
            link_off: u32::from_le_bytes(chunk[48..52].try_into().unwrap()),
            link_len: u32::from_le_bytes(chunk[52..56].try_into().unwrap()),
            first_extent: u64::from_le_bytes(chunk[56..64].try_into().unwrap()),
            extent_count: u32::from_le_bytes(chunk[64..68].try_into().unwrap()),
            hardlink_master_entry_id: u32::from_le_bytes(chunk[68..72].try_into().unwrap()),
            dev_major: u32::from_le_bytes(chunk[72..76].try_into().unwrap()),
            dev_minor: u32::from_le_bytes(chunk[76..80].try_into().unwrap()),
            meta_off: u32::from_le_bytes(chunk[80..84].try_into().unwrap()),
            meta_len: u32::from_le_bytes(chunk[84..88].try_into().unwrap()),
        });
        offset += 96;
    }
    let mut extents = Vec::with_capacity(extent_count);
    for _ in 0..extent_count {
        let chunk = &raw[offset..offset + 32];
        extents.push(ExtentRecord {
            bundle_id: u64::from_le_bytes(chunk[0..8].try_into().unwrap()),
            file_offset: u64::from_le_bytes(chunk[8..16].try_into().unwrap()),
            raw_offset_in_bundle: u32::from_le_bytes(chunk[16..20].try_into().unwrap()),
            raw_len: u32::from_le_bytes(chunk[20..24].try_into().unwrap()),
            flags: u32::from_le_bytes(chunk[24..28].try_into().unwrap()),
            entry_id: u32::from_le_bytes(chunk[28..32].try_into().unwrap()),
        });
        offset += 32;
    }
    let mut bundles = Vec::with_capacity(bundle_count);
    for _ in 0..bundle_count {
        let chunk = &raw[offset..offset + 32];
        bundles.push(BundlePlanRecord {
            bundle_id: u64::from_le_bytes(chunk[0..8].try_into().unwrap()),
            raw_len: u32::from_le_bytes(chunk[8..12].try_into().unwrap()),
            file_count: u32::from_le_bytes(chunk[12..16].try_into().unwrap()),
            extent_count: u32::from_le_bytes(chunk[16..20].try_into().unwrap()),
            kind: match chunk[20] {
                0 => BundleKind::Aggregate,
                _ => BundleKind::SingleFile,
            },
            flags: chunk[21],
        });
        offset += 32;
    }
    let name_arena = raw[offset..offset + name_arena_bytes].to_vec();
    offset += name_arena_bytes;
    let meta_blob = raw[offset..offset + meta_blob_bytes].to_vec();
    Ok(Manifest {
        entries,
        extents,
        bundles,
        name_arena,
        meta_blob,
    })
}

#[cfg(test)]
mod tests {
    use crate::config::PackConfig;
    use crate::model::{BundlePlanRecord, EntryRecord, Manifest};

    use super::{HeaderV1, decode_manifest, encode_manifest};

    #[test]
    fn header_roundtrip() {
        let manifest = Manifest {
            entries: vec![EntryRecord {
                parent_id: u32::MAX,
                kind: crate::model::EntryKind::Root,
                flags: 0,
                mode: 0,
                uid: 0,
                gid: 0,
                mtime_sec: 0,
                mtime_nsec: 0,
                size: 0,
                name_off: 0,
                name_len: 0,
                link_off: 0,
                link_len: 0,
                first_extent: 0,
                extent_count: 0,
                hardlink_master_entry_id: u32::MAX,
                dev_major: 0,
                dev_minor: 0,
                meta_off: 0,
                meta_len: 0,
            }],
            bundles: vec![BundlePlanRecord {
                bundle_id: 0,
                raw_len: 0,
                file_count: 0,
                extent_count: 0,
                kind: crate::model::BundleKind::Aggregate,
                flags: 0,
            }],
            ..Manifest::default()
        };
        let (encoded_manifest, manifest_hash) =
            encode_manifest(&manifest, PackConfig::default().manifest_codec).unwrap();
        let header = HeaderV1::from_manifest(
            &manifest,
            &PackConfig::default(),
            encoded_manifest.len(),
            manifest_hash,
        );
        let decoded = HeaderV1::decode(header.encode()).unwrap();
        assert_eq!(decoded.bundle_count, 1);
    }

    #[test]
    fn manifest_roundtrip() {
        let manifest = Manifest {
            entries: vec![EntryRecord {
                parent_id: u32::MAX,
                kind: crate::model::EntryKind::Root,
                flags: 0,
                mode: 0o755,
                uid: 42,
                gid: 7,
                mtime_sec: 1,
                mtime_nsec: 2,
                size: 0,
                name_off: 0,
                name_len: 0,
                link_off: 0,
                link_len: 0,
                first_extent: 0,
                extent_count: 0,
                hardlink_master_entry_id: u32::MAX,
                dev_major: 0,
                dev_minor: 0,
                meta_off: 0,
                meta_len: 0,
            }],
            ..Manifest::default()
        };
        let config = PackConfig::default();
        let (bytes, hash) = encode_manifest(&manifest, config.manifest_codec).unwrap();
        let header = HeaderV1::from_manifest(&manifest, &config, bytes.len(), hash);
        let decoded = decode_manifest(&header, &bytes).unwrap();
        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.entries[0].uid, 42);
    }
}
