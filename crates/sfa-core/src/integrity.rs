use crate::config::{FrameHashAlgo, IntegrityMode, ManifestHashAlgo};

pub fn manifest_hash(algo: ManifestHashAlgo, input: &[u8]) -> [u8; 32] {
    match algo {
        ManifestHashAlgo::None => [0; 32],
        ManifestHashAlgo::Blake3_256 => *blake3::hash(input).as_bytes(),
    }
}

pub fn frame_hash(algo: FrameHashAlgo, input: &[u8]) -> u64 {
    match algo {
        FrameHashAlgo::None => 0,
        FrameHashAlgo::Xxh3_64 => xxhash_rust::xxh3::xxh3_64(input),
    }
}

pub fn trailer_hash(input: &[u8]) -> [u8; 32] {
    *blake3::hash(input).as_bytes()
}

pub fn requires_trailer(mode: IntegrityMode) -> bool {
    matches!(mode, IntegrityMode::Strong)
}
