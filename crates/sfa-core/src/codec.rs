use crate::config::{DataCodec, ManifestCodec};
use crate::{Error, Result};

pub fn encode_data(codec: DataCodec, input: &[u8], level: Option<i32>) -> Result<Vec<u8>> {
    match codec {
        DataCodec::None => Ok(input.to_vec()),
        DataCodec::Lz4 => Ok(lz4_flex::block::compress(input)),
        DataCodec::Zstd => zstd::bulk::compress(input, level.unwrap_or(3)).map_err(Error::from),
    }
}

pub fn decode_data(codec: DataCodec, input: &[u8], expected_raw_len: usize) -> Result<Vec<u8>> {
    match codec {
        DataCodec::None => Ok(input.to_vec()),
        DataCodec::Lz4 => lz4_flex::block::decompress(input, expected_raw_len)
            .map_err(|error| Error::Message(format!("lz4 decode failed: {error}"))),
        DataCodec::Zstd => zstd::bulk::decompress(input, expected_raw_len).map_err(Error::from),
    }
}

pub fn encode_manifest(codec: ManifestCodec, input: &[u8]) -> Result<Vec<u8>> {
    match codec {
        ManifestCodec::None => Ok(input.to_vec()),
        ManifestCodec::Zstd => zstd::bulk::compress(input, 3).map_err(Error::from),
    }
}

pub fn decode_manifest(
    codec: ManifestCodec,
    input: &[u8],
    expected_raw_len: usize,
) -> Result<Vec<u8>> {
    match codec {
        ManifestCodec::None => Ok(input.to_vec()),
        ManifestCodec::Zstd => zstd::bulk::decompress(input, expected_raw_len).map_err(Error::from),
    }
}
