use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn cli_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sfa"))
}

fn write_sample_source(root: &Path) {
    fs::create_dir_all(root.join("dir")).unwrap();
    fs::write(root.join("dir/hello.txt"), b"hello from archive").unwrap();
    fs::write(root.join("notes.txt"), b"fixture notes").unwrap();
}

fn pack_sample_archive(src: &Path, archive: &Path) {
    sfa_unixfs::pack_directory(src, archive, &sfa_core::PackConfig::default()).unwrap();
}

#[test]
fn pack_dry_run_json_reports_default_options() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    let archive = temp.path().join("dry-run.sfa");
    write_sample_source(&src);

    let output = cli_command()
        .args(["pack"])
        .arg(&src)
        .arg(&archive)
        .args(["--dry-run", "--stats-format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!archive.exists(), "dry-run should not create an archive");

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("pack dry-run json stats");
    assert_eq!(json["codec"].as_str(), Some("zstd"));
    assert_eq!(json["bundle_target_bytes"].as_u64(), Some(4 * 1024 * 1024));
    assert_eq!(json["small_file_threshold"].as_u64(), Some(256 * 1024));
    assert_eq!(json["entry_count"].as_u64(), Some(4));
    assert!(json["threads"].as_u64().unwrap_or(0) >= 1);
}

#[test]
fn pack_without_codec_flag_writes_zstd_archive_header() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    let archive = temp.path().join("packed.sfa");
    write_sample_source(&src);

    let output = cli_command()
        .args(["pack"])
        .arg(&src)
        .arg(&archive)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut reader = BufReader::new(fs::File::open(&archive).unwrap());
    let header = sfa_core::format::read_header(&mut reader).expect("read archive header");
    assert_eq!(header.data_codec, sfa_core::DataCodec::Zstd);
}

#[test]
fn pack_missing_arguments_exits_with_usage_error() {
    let output = cli_command().arg("pack").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Usage:"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn unpack_dry_run_with_file_archive_reports_json_stats() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    let archive = temp.path().join("sample.sfa");
    let out = temp.path().join("out");
    write_sample_source(&src);
    pack_sample_archive(&src, &archive);
    let archive_size = fs::metadata(&archive).unwrap().len();

    let output = cli_command()
        .args(["unpack"])
        .arg(&archive)
        .args(["-C"])
        .arg(&out)
        .args(["--dry-run", "--stats-format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !out.exists(),
        "dry-run should not create an output directory"
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("unpack dry-run json stats");
    assert_eq!(json["codec"].as_str(), Some("zstd"));
    assert_eq!(json["raw_bytes"].as_u64(), Some(archive_size));
    assert_eq!(json["encoded_bytes"].as_u64(), Some(archive_size));
    assert!(json["threads"].as_u64().unwrap_or(0) >= 1);
    assert!(json["bundle_count"].as_u64().unwrap_or(0) >= 1);
    assert_eq!(
        json["wall_breakdown"]["setup_ms"]["status"].as_str(),
        Some("unavailable")
    );
    assert_eq!(
        json["wall_breakdown"]["pipeline_ms"]["status"].as_str(),
        Some("unavailable")
    );
    assert_eq!(
        json["wall_breakdown"]["finalize_ms"]["note"].as_str(),
        Some("dry-run does not measure execution phases")
    );
    assert_eq!(
        json["phase_breakdown"]["decode_ms"]["status"].as_str(),
        Some("unavailable")
    );
}

#[test]
fn unpack_missing_archive_reports_io_error() {
    let temp = TempDir::new().unwrap();
    let out = temp.path().join("out");
    let missing = temp.path().join("missing.sfa");

    let output = cli_command()
        .args(["unpack"])
        .arg(&missing)
        .args(["-C"])
        .arg(&out)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(20));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("input archive does not exist"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn unpack_without_overwrite_preserves_existing_output() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    let archive = temp.path().join("sample.sfa");
    let out = temp.path().join("out");
    write_sample_source(&src);
    pack_sample_archive(&src, &archive);

    fs::create_dir_all(out.join("dir")).unwrap();
    fs::write(out.join("dir/hello.txt"), b"stale content").unwrap();

    let output = cli_command()
        .args(["unpack"])
        .arg(&archive)
        .args(["-C"])
        .arg(&out)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(20));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("path already exists"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(out.join("dir/hello.txt")).unwrap(),
        b"stale content"
    );
}

#[test]
fn unpack_with_overwrite_replaces_existing_output() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    let archive = temp.path().join("sample.sfa");
    let out = temp.path().join("out");
    write_sample_source(&src);
    pack_sample_archive(&src, &archive);

    fs::create_dir_all(out.join("dir")).unwrap();
    fs::write(out.join("dir/hello.txt"), b"stale content").unwrap();

    let output = cli_command()
        .args(["unpack"])
        .arg(&archive)
        .args(["-C"])
        .arg(&out)
        .arg("--overwrite")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(out.join("dir/hello.txt")).unwrap(),
        b"hello from archive"
    );
}
