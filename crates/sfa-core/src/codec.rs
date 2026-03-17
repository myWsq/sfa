use crate::config::{DataCodec, ManifestCodec};
use crate::{Error, Result};

const DEFAULT_ZSTD_LEVEL: i32 = -3;

pub fn encode_data(codec: DataCodec, input: &[u8], level: Option<i32>) -> Result<Vec<u8>> {
    match codec {
        DataCodec::None => Ok(input.to_vec()),
        DataCodec::Lz4 => Ok(lz4_flex::block::compress(input)),
        DataCodec::Zstd => {
            zstd::bulk::compress(input, level.unwrap_or(DEFAULT_ZSTD_LEVEL)).map_err(Error::from)
        }
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
        ManifestCodec::Zstd => zstd::bulk::compress(input, DEFAULT_ZSTD_LEVEL).map_err(Error::from),
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

#[cfg(test)]
mod tests {
    use super::{DEFAULT_ZSTD_LEVEL, encode_data, encode_manifest};
    use crate::config::{DataCodec, ManifestCodec};

    #[test]
    fn encode_data_defaults_to_negative_three_for_zstd() {
        let input = b"default zstd data level";

        let encoded = encode_data(DataCodec::Zstd, input, None).expect("encode data");
        let expected = zstd::bulk::compress(input, DEFAULT_ZSTD_LEVEL).expect("expected data");

        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_manifest_defaults_to_negative_three_for_zstd() {
        let input = b"default zstd manifest level";

        let encoded = encode_manifest(ManifestCodec::Zstd, input).expect("encode manifest");
        let expected = zstd::bulk::compress(input, DEFAULT_ZSTD_LEVEL).expect("expected manifest");

        assert_eq!(encoded, expected);
    }
}
