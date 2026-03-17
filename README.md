# SFA

SFA, short for Small File Archive, is a manifest-first archive format and Rust toolchain for Unix directory trees with many small files. It is designed for deterministic scanning, sequential reads, integrity validation, and benchmarkable comparisons against `tar` with the same codec settings.

SFA is not intended to be a drop-in replacement for `tar`. The project focuses on a narrower problem: reliable local archiving and restore workflows for Unix-like directory trees where metadata, ordering, and small-file behavior matter.

## Project Status

SFA is under active development.

- `format-v1` is frozen and serves as the current compatibility boundary.
- Milestones M0, M1, and M2 are complete.
- Current work is focused on M3: tightening the Unix metadata contract and its repository-level verification assets.

At the repository level, SFA already provides a runnable `pack` / `unpack` chain, canonical golden fixtures, CLI regression coverage, committed benchmark datasets, and machine-readable benchmark baselines.

For milestone details and current priorities, see [ROADMAP.md](ROADMAP.md).

## Features

- End-to-end `sfa pack` and `sfa unpack` workflows for Unix-like directory trees
- Manifest-first `.sfa` layout with header, manifest, frames, and an optional trailer
- `lz4` and `zstd` data codec support
- Deterministic directory scanning and stable bundle planning
- Sequential archive read/write path without seek-dependent restore logic
- Support for regular files, directories, symlinks, and hardlinks
- Default restore of `mode` and `mtime` for regular files and directories
- Archive-side recording of `uid` and `gid`, with explicit opt-in owner restore policy
- Path safety checks, integrity validation, and basic corruption detection
- Machine-readable command stats via `--stats-format json`
- Benchmarks against `tar` with the same codec configuration
- Repository-traceable benchmark datasets and committed baseline results
- Safe restore path built around `dirfd` / `openat`-style object creation
- `.sfa-untrusted` marker emission when `strong` trailer verification fails during restore

## Current Scope and Non-Goals

SFA currently does not promise:

- Full Unix extended metadata coverage such as xattrs, ACLs, or device file restore
- Fully equivalent behavior on non-Unix platforms
- crates.io distribution
- Installer-style distribution, macOS notarization, or code signing

## Installation

### Build From Source

Requirements:

- Rust `1.85` or newer
- A Unix-like environment

Build the CLI:

```bash
cargo build --release -p sfa-cli
```

The release binary is produced at `target/release/sfa-cli`.

### GitHub Release Archives

GitHub Releases are intended to publish prebuilt CLI archives for:

- Linux `x86_64`
- macOS `x86_64`
- macOS `arm64`

Each archive contains the `sfa-cli` binary together with `README.md` and `LICENSE`.

## Quick Start

Create an archive:

```bash
./target/release/sfa-cli pack ./input ./archive.sfa --codec zstd --integrity strong
```

Extract an archive:

```bash
./target/release/sfa-cli unpack ./archive.sfa -C ./restore
```

Extract from standard input:

```bash
cat ./archive.sfa | ./target/release/sfa-cli unpack - -C ./restore
```

Emit machine-readable stats:

```bash
./target/release/sfa-cli pack ./input ./archive.sfa --stats-format json
```

## Verification

The repository keeps a release-grade verification checklist. The current authoritative checks are:

```bash
cargo fmt --all --check
cargo test --workspace
bash tests/scripts/run_protocol_smoke.sh
bash tests/scripts/run_streaming_smoke.sh
bash tests/scripts/run_safety_smoke.sh
bash tests/scripts/run_roundtrip_smoke.sh
cargo run -p sfa-bench --bin tar_vs_sfa -- --dry-run --output benches/results/latest.json
```

See [RELEASING.md](RELEASING.md) for the release process and quality gates.

## Repository Layout

- `crates/sfa-core`: archive format, codecs, integrity, planning, and ordered readers
- `crates/sfa-unixfs`: Unix filesystem scan, archive, and restore implementation
- `crates/sfa-cli`: command-line entry point
- `crates/sfa-bench`: benchmark runner and fixture dump tooling
- `spec/`: protocol and verification specifications
- `tests/`: fixtures, smoke scripts, and repository-level regression assets
- `release-notes/`: release note drafts kept in-repo
- `sfa-tech-solution/`: implementation and design background documents
- `openspec/`: change proposals, design notes, and task breakdowns

## Documentation

- [ROADMAP.md](ROADMAP.md): milestone status and short-term priorities
- [RELEASING.md](RELEASING.md): release checklist and GitHub release process
- [CHANGELOG.md](CHANGELOG.md): repository-level change history
- [spec/README.md](spec/README.md): protocol and verification entry point
- [spec/format-v1-freeze-review.md](spec/format-v1-freeze-review.md): protocol freeze review record
- [sfa-tech-solution/README.md](sfa-tech-solution/README.md): broader technical solution overview

## Contributing

Contributions are welcome, but larger changes should start from the repository's current roadmap and specification boundary.

Before opening a substantial change, review:

- [ROADMAP.md](ROADMAP.md)
- [spec/README.md](spec/README.md)
- [sfa-tech-solution/README.md](sfa-tech-solution/README.md)

If your change affects the archive format, fixtures, benchmark baselines, or restore semantics, keep the implementation, specs, and verification assets in sync.

## License

SFA is released under the MIT License. See [LICENSE](LICENSE).
