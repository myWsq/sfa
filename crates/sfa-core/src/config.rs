use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u16)]
pub enum DataCodec {
    #[default]
    None = 0,
    Lz4 = 1,
    Zstd = 2,
}

impl DataCodec {
    pub fn from_u16(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Lz4),
            2 => Ok(Self::Zstd),
            other => Err(Error::UnsupportedDataCodec(other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u16)]
pub enum ManifestCodec {
    None = 0,
    #[default]
    Zstd = 1,
}

impl ManifestCodec {
    pub fn from_u16(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Zstd),
            other => Err(Error::UnsupportedManifestCodec(other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum IntegrityMode {
    Off = 0,
    #[default]
    Fast = 1,
    Strong = 2,
}

impl IntegrityMode {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Fast),
            2 => Ok(Self::Strong),
            other => Err(Error::UnsupportedIntegrityMode(other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum FrameHashAlgo {
    None = 0,
    #[default]
    Xxh3_64 = 1,
}

impl FrameHashAlgo {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Xxh3_64),
            other => Err(Error::UnsupportedFrameHashAlgo(other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ManifestHashAlgo {
    None = 0,
    #[default]
    Blake3_256 = 1,
}

impl ManifestHashAlgo {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Blake3_256),
            other => Err(Error::UnsupportedManifestHashAlgo(other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OverwritePolicy {
    #[default]
    Error,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RestoreOwnerPolicy {
    #[default]
    Skip,
    Preserve,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackConfig {
    pub codec: DataCodec,
    pub manifest_codec: ManifestCodec,
    pub compression_level: Option<i32>,
    pub threads: usize,
    pub bundle_target_bytes: u32,
    pub small_file_threshold: u32,
    pub integrity: IntegrityMode,
    pub preserve_owner: bool,
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            codec: DataCodec::Lz4,
            manifest_codec: ManifestCodec::Zstd,
            compression_level: None,
            threads: std::thread::available_parallelism()
                .map(|value| value.get())
                .unwrap_or(1),
            bundle_target_bytes: 4 * 1024 * 1024,
            small_file_threshold: 256 * 1024,
            integrity: IntegrityMode::Fast,
            preserve_owner: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnpackConfig {
    pub threads: Option<usize>,
    pub overwrite: OverwritePolicy,
    pub restore_owner: RestoreOwnerPolicy,
    pub integrity: IntegrityMode,
}

impl Default for UnpackConfig {
    fn default() -> Self {
        Self {
            threads: None,
            overwrite: OverwritePolicy::Error,
            restore_owner: RestoreOwnerPolicy::Skip,
            integrity: IntegrityMode::Fast,
        }
    }
}
