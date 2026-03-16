use std::fs::File;
use std::path::PathBuf;

use serde::Serialize;
use sfa_core::ArchiveReader;

#[derive(Debug, Serialize)]
struct FixtureDump {
    version_major: u16,
    version_minor: u16,
    data_codec: String,
    manifest_codec: String,
    integrity_mode: String,
    entry_count: u64,
    extent_count: u64,
    bundle_count: u64,
    manifest_raw_len: u64,
    manifest_encoded_len: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: dump_archive_fixture <archive.sfa> <output.json>");
        std::process::exit(2);
    }

    let archive = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    let reader = File::open(&archive)?;
    let mut archive_reader = ArchiveReader::new(reader);
    let header = archive_reader.read_header()?;
    let manifest = archive_reader.read_manifest()?;

    let dump = FixtureDump {
        version_major: header.version_major,
        version_minor: header.version_minor,
        data_codec: format!("{:?}", header.data_codec).to_lowercase(),
        manifest_codec: format!("{:?}", header.manifest_codec).to_lowercase(),
        integrity_mode: format!("{:?}", header.integrity_mode).to_lowercase(),
        entry_count: manifest.entry_count(),
        extent_count: manifest.extent_count(),
        bundle_count: manifest.bundle_count(),
        manifest_raw_len: header.manifest_raw_len,
        manifest_encoded_len: header.manifest_encoded_len,
    };

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, serde_json::to_string_pretty(&dump)?)?;
    Ok(())
}
