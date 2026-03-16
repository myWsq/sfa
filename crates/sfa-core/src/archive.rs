use std::io::{Read, Write};

use crate::codec;
use crate::config::{IntegrityMode, PackConfig};
use crate::format::{
    FrameHeaderV1, HeaderV1, TrailerV1, decode_manifest, encode_manifest, read_header, write_header,
};
use crate::integrity;
use crate::model::{EncodedFrame, Manifest};
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct PreparedArchive {
    pub header: HeaderV1,
    pub manifest_bytes: Vec<u8>,
    pub manifest: Manifest,
}

pub fn prepare_archive(manifest: Manifest, config: &PackConfig) -> Result<PreparedArchive> {
    let (manifest_bytes, manifest_hash) = encode_manifest(&manifest, config.manifest_codec)?;
    let header = HeaderV1::from_manifest(&manifest, config, manifest_bytes.len(), manifest_hash);
    Ok(PreparedArchive {
        header,
        manifest_bytes,
        manifest,
    })
}

pub fn write_archive<W, I>(
    writer: &mut W,
    prepared: &PreparedArchive,
    frames: I,
    integrity_mode: IntegrityMode,
) -> Result<Option<TrailerV1>>
where
    W: Write,
    I: IntoIterator<Item = EncodedFrame>,
{
    write_header(writer, &prepared.header)?;
    writer
        .write_all(&prepared.manifest_bytes)
        .map_err(Error::from)?;

    let mut total_raw_bytes = 0u64;
    let mut total_encoded_bytes = 0u64;
    let mut archive_hash_bytes = Vec::new();

    for frame in frames {
        writer
            .write_all(&frame.header.encode())
            .map_err(Error::from)?;
        writer.write_all(&frame.payload).map_err(Error::from)?;
        total_raw_bytes += frame.header.raw_len as u64;
        total_encoded_bytes += frame.header.encoded_len as u64;
        archive_hash_bytes.extend_from_slice(&frame.header.frame_hash.to_le_bytes());
    }

    if integrity::requires_trailer(integrity_mode) {
        let trailer = TrailerV1 {
            bundle_count: prepared.header.bundle_count,
            total_raw_bytes,
            total_encoded_bytes,
            archive_hash: integrity::trailer_hash(&archive_hash_bytes),
        };
        writer.write_all(&trailer.encode()).map_err(Error::from)?;
        Ok(Some(trailer))
    } else {
        Ok(None)
    }
}

pub struct ArchiveReader<R> {
    reader: R,
    header: Option<HeaderV1>,
    frame_index: u64,
}

impl<R: Read> ArchiveReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            header: None,
            frame_index: 0,
        }
    }

    pub fn read_header(&mut self) -> Result<HeaderV1> {
        let header = read_header(&mut self.reader)?;
        self.header = Some(header.clone());
        Ok(header)
    }

    pub fn read_manifest(&mut self) -> Result<Manifest> {
        let header = self
            .header
            .clone()
            .ok_or(Error::InvalidState("header must be read first"))?;
        let mut manifest_bytes = vec![0u8; header.manifest_encoded_len as usize];
        self.reader
            .read_exact(&mut manifest_bytes)
            .map_err(Error::from)?;
        decode_manifest(&header, &manifest_bytes)
    }

    pub fn next_frame(&mut self) -> Result<Option<EncodedFrame>> {
        let header = self
            .header
            .clone()
            .ok_or(Error::InvalidState("header must be read first"))?;
        if self.frame_index >= header.bundle_count {
            return Ok(None);
        }

        let mut frame_header_bytes = [0u8; crate::format::FRAME_HEADER_LEN];
        self.reader
            .read_exact(&mut frame_header_bytes)
            .map_err(Error::from)?;
        let frame_header = FrameHeaderV1::decode(frame_header_bytes)?;
        let mut payload = vec![0u8; frame_header.encoded_len as usize];
        self.reader.read_exact(&mut payload).map_err(Error::from)?;
        let decoded =
            codec::decode_data(header.data_codec, &payload, frame_header.raw_len as usize)?;
        let expected_hash = integrity::frame_hash(header.frame_hash_algo, &decoded);
        if expected_hash != frame_header.frame_hash {
            return Err(Error::FrameHashMismatch {
                bundle_id: frame_header.bundle_id,
            });
        }
        self.frame_index += 1;
        Ok(Some(EncodedFrame {
            header: frame_header,
            payload,
        }))
    }

    pub fn read_trailer(&mut self) -> Result<Option<TrailerV1>> {
        let header = self
            .header
            .clone()
            .ok_or(Error::InvalidState("header must be read first"))?;
        if !header
            .feature_flags
            .contains(crate::format::FeatureFlags::HAS_TRAILER)
        {
            return Ok(None);
        }
        let mut bytes = [0u8; crate::format::TRAILER_LEN];
        self.reader.read_exact(&mut bytes).map_err(Error::from)?;
        Ok(Some(TrailerV1::decode(bytes)?))
    }
}
