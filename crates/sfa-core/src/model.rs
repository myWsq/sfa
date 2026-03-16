use std::path::PathBuf;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum EntryKind {
    #[default]
    Root = 0,
    Directory = 1,
    Regular = 2,
    Symlink = 3,
    Hardlink = 4,
    CharDevice = 5,
    BlockDevice = 6,
    Fifo = 7,
}

impl EntryKind {
    pub fn from_u8(value: u8) -> crate::Result<Self> {
        match value {
            0 => Ok(Self::Root),
            1 => Ok(Self::Directory),
            2 => Ok(Self::Regular),
            3 => Ok(Self::Symlink),
            4 => Ok(Self::Hardlink),
            5 => Ok(Self::CharDevice),
            6 => Ok(Self::BlockDevice),
            7 => Ok(Self::Fifo),
            other => Err(crate::Error::UnsupportedEntryKind(other)),
        }
    }

    pub fn carries_data(self) -> bool {
        matches!(self, Self::Regular)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct EntryFlags: u8 {
        const EMPTY_FILE = 1 << 0;
        const PATH_VALIDATED = 1 << 1;
        const HAS_METADATA = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum BundleKind {
    #[default]
    Aggregate = 0,
    SingleFile = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerInputEntry {
    pub entry_id: u32,
    pub parent_id: u32,
    pub kind: EntryKind,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
    pub size: u64,
    pub name: Vec<u8>,
    pub link_target: Option<Vec<u8>>,
    pub source_path: Option<PathBuf>,
    pub hardlink_master_entry_id: Option<u32>,
    pub dev_major: u32,
    pub dev_minor: u32,
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlePart {
    pub entry_id: u32,
    pub source_path: PathBuf,
    pub file_offset: u64,
    pub raw_len: u32,
    pub raw_offset_in_bundle: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInput {
    pub bundle_id: u64,
    pub kind: BundleKind,
    pub raw_len: u32,
    pub parts: Vec<BundlePart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryRecord {
    pub parent_id: u32,
    pub kind: EntryKind,
    pub flags: u8,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
    pub size: u64,
    pub name_off: u32,
    pub name_len: u32,
    pub link_off: u32,
    pub link_len: u32,
    pub first_extent: u64,
    pub extent_count: u32,
    pub hardlink_master_entry_id: u32,
    pub dev_major: u32,
    pub dev_minor: u32,
    pub meta_off: u32,
    pub meta_len: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtentRecord {
    pub bundle_id: u64,
    pub entry_id: u32,
    pub file_offset: u64,
    pub raw_offset_in_bundle: u32,
    pub raw_len: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlePlanRecord {
    pub bundle_id: u64,
    pub raw_len: u32,
    pub file_count: u32,
    pub extent_count: u32,
    pub kind: BundleKind,
    pub flags: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: Vec<EntryRecord>,
    pub extents: Vec<ExtentRecord>,
    pub bundles: Vec<BundlePlanRecord>,
    pub name_arena: Vec<u8>,
    pub meta_blob: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub header: crate::format::FrameHeaderV1,
    pub payload: Vec<u8>,
}

impl Manifest {
    pub fn raw_len(&self) -> usize {
        crate::format::MANIFEST_HEADER_LEN
            + self.entries.len() * 96
            + self.extents.len() * 32
            + self.bundles.len() * 32
            + self.name_arena.len()
            + self.meta_blob.len()
    }

    pub fn entry_count(&self) -> u64 {
        self.entries.len() as u64
    }

    pub fn extent_count(&self) -> u64 {
        self.extents.len() as u64
    }

    pub fn bundle_count(&self) -> u64 {
        self.bundles.len() as u64
    }
}
