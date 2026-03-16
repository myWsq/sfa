use crate::model::{
    BundleInput, BundleKind, BundlePart, BundlePlanRecord, EntryKind, EntryRecord, ExtentRecord,
    Manifest, PlannerInputEntry,
};
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct PlannedArchive {
    pub manifest: Manifest,
    pub bundles: Vec<BundleInput>,
}

pub fn plan_archive(
    entries: &[PlannerInputEntry],
    bundle_target_bytes: u32,
    small_file_threshold: u32,
) -> Result<PlannedArchive> {
    let mut manifest = Manifest::default();
    let mut bundles = Vec::new();
    let mut name_arena = Vec::new();
    let mut meta_blob = Vec::new();
    let mut extents = Vec::new();
    let mut bundle_records = Vec::new();
    let mut bundle_id = 0u64;
    let mut current = BundleInput {
        bundle_id,
        kind: BundleKind::Aggregate,
        raw_len: 0,
        parts: Vec::new(),
    };

    for entry in entries {
        let name_off = name_arena.len() as u32;
        name_arena.extend_from_slice(&entry.name);
        let link_off = if let Some(target) = &entry.link_target {
            let off = name_arena.len() as u32;
            name_arena.extend_from_slice(target);
            off
        } else {
            0
        };
        let meta_off = meta_blob.len() as u32;
        meta_blob.extend_from_slice(&entry.metadata);
        let first_extent = extents.len() as u64;
        let mut extent_count = 0u32;

        if entry.kind == EntryKind::Regular
            && entry.hardlink_master_entry_id.is_none()
            && entry.size > 0
        {
            let source_path = entry
                .source_path
                .clone()
                .ok_or(Error::MissingSourcePath(entry.entry_id))?;
            if entry.size < small_file_threshold as u64 {
                if current.raw_len > 0
                    && current.raw_len as u64 + entry.size > bundle_target_bytes as u64
                {
                    finalize_bundle(&mut bundles, &mut bundle_records, &mut current);
                    bundle_id += 1;
                    current = BundleInput {
                        bundle_id,
                        kind: BundleKind::Aggregate,
                        raw_len: 0,
                        parts: Vec::new(),
                    };
                }
                let raw_offset = current.raw_len;
                let raw_len = entry.size as u32;
                current.parts.push(BundlePart {
                    entry_id: entry.entry_id,
                    source_path,
                    file_offset: 0,
                    raw_len,
                    raw_offset_in_bundle: raw_offset,
                });
                current.raw_len += raw_len;
                extents.push(ExtentRecord {
                    bundle_id: current.bundle_id,
                    entry_id: entry.entry_id,
                    file_offset: 0,
                    raw_offset_in_bundle: raw_offset,
                    raw_len,
                    flags: 0b11,
                });
                extent_count = 1;
            } else {
                if current.raw_len > 0 {
                    finalize_bundle(&mut bundles, &mut bundle_records, &mut current);
                    bundle_id += 1;
                    current = BundleInput {
                        bundle_id,
                        kind: BundleKind::Aggregate,
                        raw_len: 0,
                        parts: Vec::new(),
                    };
                }
                let mut file_offset = 0u64;
                while file_offset < entry.size {
                    let chunk_len =
                        ((entry.size - file_offset).min(bundle_target_bytes as u64)) as u32;
                    let single_bundle_id = bundle_id;
                    bundles.push(BundleInput {
                        bundle_id: single_bundle_id,
                        kind: BundleKind::SingleFile,
                        raw_len: chunk_len,
                        parts: vec![BundlePart {
                            entry_id: entry.entry_id,
                            source_path: source_path.clone(),
                            file_offset,
                            raw_len: chunk_len,
                            raw_offset_in_bundle: 0,
                        }],
                    });
                    bundle_records.push(BundlePlanRecord {
                        bundle_id: single_bundle_id,
                        raw_len: chunk_len,
                        file_count: 1,
                        extent_count: 1,
                        kind: BundleKind::SingleFile,
                        flags: 0,
                    });
                    extents.push(ExtentRecord {
                        bundle_id: single_bundle_id,
                        entry_id: entry.entry_id,
                        file_offset,
                        raw_offset_in_bundle: 0,
                        raw_len: chunk_len,
                        flags: if file_offset + chunk_len as u64 >= entry.size {
                            0b101
                        } else {
                            0b100
                        },
                    });
                    extent_count += 1;
                    file_offset += chunk_len as u64;
                    bundle_id += 1;
                }
                current.bundle_id = bundle_id;
            }
        }

        manifest.entries.push(EntryRecord {
            parent_id: entry.parent_id,
            kind: entry.kind,
            flags: if entry.size == 0 && entry.kind == EntryKind::Regular {
                0b1
            } else {
                0
            },
            mode: entry.mode,
            uid: entry.uid,
            gid: entry.gid,
            mtime_sec: entry.mtime_sec,
            mtime_nsec: entry.mtime_nsec,
            size: entry.size,
            name_off,
            name_len: entry.name.len() as u32,
            link_off,
            link_len: entry
                .link_target
                .as_ref()
                .map(|target| target.len() as u32)
                .unwrap_or(0),
            first_extent,
            extent_count,
            hardlink_master_entry_id: entry.hardlink_master_entry_id.unwrap_or(u32::MAX),
            dev_major: entry.dev_major,
            dev_minor: entry.dev_minor,
            meta_off,
            meta_len: entry.metadata.len() as u32,
        });
    }

    if current.raw_len > 0 {
        finalize_bundle(&mut bundles, &mut bundle_records, &mut current);
    }

    manifest.extents = extents;
    manifest.bundles = bundle_records;
    manifest.name_arena = name_arena;
    manifest.meta_blob = meta_blob;
    Ok(PlannedArchive { manifest, bundles })
}

fn finalize_bundle(
    bundles: &mut Vec<BundleInput>,
    bundle_records: &mut Vec<BundlePlanRecord>,
    current: &mut BundleInput,
) {
    if current.raw_len == 0 {
        return;
    }
    bundle_records.push(BundlePlanRecord {
        bundle_id: current.bundle_id,
        raw_len: current.raw_len,
        file_count: current.parts.len() as u32,
        extent_count: current.parts.len() as u32,
        kind: current.kind,
        flags: 0,
    });
    bundles.push(current.clone());
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::model::{EntryKind, PlannerInputEntry};

    use super::plan_archive;

    #[test]
    fn aggregates_small_files_and_chunks_large_files() {
        let entries = vec![
            PlannerInputEntry {
                entry_id: 0,
                parent_id: u32::MAX,
                kind: EntryKind::Root,
                mode: 0,
                uid: 0,
                gid: 0,
                mtime_sec: 0,
                mtime_nsec: 0,
                size: 0,
                name: Vec::new(),
                link_target: None,
                source_path: None,
                hardlink_master_entry_id: None,
                dev_major: 0,
                dev_minor: 0,
                metadata: Vec::new(),
            },
            PlannerInputEntry {
                entry_id: 1,
                parent_id: 0,
                kind: EntryKind::Regular,
                mode: 0,
                uid: 0,
                gid: 0,
                mtime_sec: 0,
                mtime_nsec: 0,
                size: 3,
                name: b"a".to_vec(),
                link_target: None,
                source_path: Some(PathBuf::from("a")),
                hardlink_master_entry_id: None,
                dev_major: 0,
                dev_minor: 0,
                metadata: Vec::new(),
            },
            PlannerInputEntry {
                entry_id: 2,
                parent_id: 0,
                kind: EntryKind::Regular,
                mode: 0,
                uid: 0,
                gid: 0,
                mtime_sec: 0,
                mtime_nsec: 0,
                size: 20,
                name: b"b".to_vec(),
                link_target: None,
                source_path: Some(PathBuf::from("b")),
                hardlink_master_entry_id: None,
                dev_major: 0,
                dev_minor: 0,
                metadata: Vec::new(),
            },
        ];
        let planned = plan_archive(&entries, 8, 8).unwrap();
        assert_eq!(planned.bundles.len(), 4);
        assert_eq!(planned.manifest.extents.len(), 4);
    }
}
