use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use tempfile::TempDir;

#[test]
fn unpack_accepts_archive_from_stdin() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    let out = temp.path().join("out");
    let archive = temp.path().join("sample.sfa");
    fs::create_dir_all(src.join("dir")).unwrap();
    fs::write(src.join("dir/hello.txt"), b"hello stdin").unwrap();
    fs::write(src.join("empty.txt"), b"").unwrap();

    sfa_unixfs::pack_directory(&src, &archive, &sfa_core::PackConfig::default()).unwrap();
    let archive_bytes = fs::read(&archive).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_sfa"))
        .args(["unpack", "-", "-C"])
        .arg(&out)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&archive_bytes)
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(out.join("dir/hello.txt")).unwrap(), b"hello stdin");
    assert_eq!(fs::read(out.join("empty.txt")).unwrap(), b"");
}

#[test]
fn unpack_stdin_json_reports_wall_and_phase_breakdowns() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    let out = temp.path().join("out");
    let archive = temp.path().join("sample.sfa");
    fs::create_dir_all(src.join("dir")).unwrap();
    fs::write(src.join("dir/hello.txt"), b"hello stdin").unwrap();
    fs::write(src.join("empty.txt"), b"").unwrap();

    sfa_unixfs::pack_directory(&src, &archive, &sfa_core::PackConfig::default()).unwrap();
    let archive_bytes = fs::read(&archive).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_sfa"))
        .args(["unpack", "-", "-C"])
        .arg(&out)
        .args(["--stats-format", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&archive_bytes)
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("unpack stdin json stats");
    assert_eq!(json["codec"].as_str(), Some("zstd"));
    assert_eq!(
        json["wall_breakdown"]["setup_ms"]["status"].as_str(),
        Some("measured")
    );
    assert_eq!(
        json["wall_breakdown"]["pipeline_ms"]["status"].as_str(),
        Some("measured")
    );
    assert_eq!(
        json["wall_breakdown"]["finalize_ms"]["status"].as_str(),
        Some("measured")
    );
    assert_eq!(
        json["phase_breakdown"]["decode_ms"]["status"].as_str(),
        Some("measured")
    );
    assert_eq!(
        json["phase_breakdown"]["scatter_ms"]["status"].as_str(),
        Some("measured")
    );
}

#[test]
fn unpack_dry_run_rejects_archive_from_stdin() {
    let temp = TempDir::new().unwrap();
    let out = temp.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_sfa"))
        .args(["unpack", "-", "-C"])
        .arg(&out)
        .arg("--dry-run")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("dry-run is not supported"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn unpack_writes_diagnostics_json_when_requested() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    let out = temp.path().join("out");
    let archive = temp.path().join("sample.sfa");
    let diagnostics = temp.path().join("diag/unpack.json");
    fs::create_dir_all(src.join("pkg")).unwrap();
    fs::write(src.join("pkg/a.txt"), b"aaa").unwrap();
    fs::write(src.join("pkg/b.txt"), b"bbb").unwrap();

    sfa_unixfs::pack_directory(&src, &archive, &sfa_core::PackConfig::default()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_sfa"))
        .args(["unpack"])
        .arg(&archive)
        .args(["-C"])
        .arg(&out)
        .env("SFA_UNPACK_DIAGNOSTICS_JSON", &diagnostics)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&fs::read(&diagnostics).unwrap()).expect("valid diagnostics json");
    assert_eq!(json["stats"]["entry_count"].as_u64(), Some(4));
    assert_eq!(
        json["stats"]["wall_breakdown"]["setup_ms"]["status"].as_str(),
        Some("measured")
    );
    assert_eq!(
        json["stats"]["wall_breakdown"]["pipeline_ms"]["status"].as_str(),
        Some("measured")
    );
    assert_eq!(
        json["stats"]["wall_breakdown"]["finalize_ms"]["status"].as_str(),
        Some("measured")
    );
    assert!(
        json["diagnostics"]["scatter"]["write_ns"]
            .as_u64()
            .is_some()
    );
    assert!(
        json["diagnostics"]["pipeline"]["bundles_observed"]
            .as_u64()
            .is_some()
    );
}
