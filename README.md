# SFA

SFA, short for Small File Archive, is a manifest-first archive format and CLI for Unix directory trees with many small files. It is designed for deterministic scanning, sequential reads, integrity validation, and ordered restore behavior without treating `tar` compatibility as the primary goal.

On the current committed macOS `aarch64` benchmark baseline, SFA is about `5x` faster than `tar` on the default small-text dataset and about `10x` faster on one current unpack control case when both use the same codec.

SFA is not intended to be a drop-in replacement for `tar`. It focuses on a narrower problem: reliable local archiving and restore workflows for Unix-like directory trees where metadata, ordering, and small-file behavior matter.

## When To Use SFA

SFA is a good fit when you need:

- Fast pack and unpack workflows for Unix directory trees with many small files
- Deterministic directory scanning and stable bundle planning
- Sequential archive reads without seek-dependent restore logic
- Benchmarkable comparisons against `tar` with the same codec settings
- Path safety checks, integrity validation, and explicit restore-policy controls

SFA is not yet the right fit when you need:

- Full Unix extended metadata coverage such as xattrs, ACLs, or device file restore
- Fully equivalent behavior on non-Unix platforms
- crates.io distribution
- Installer-style distribution, macOS notarization, or code signing

## Project Status

SFA is under active development and is currently preparing its first stable `v1.0.0` release.

- `format-v1` is frozen and serves as the current compatibility boundary.
- Milestones M0 through M3 for the current stable-v1 scope are complete.
- The current release train keeps xattrs, ACLs, special-file restore, and broader Unix extensions deferred to post-v1 work.

For milestone details and current priorities, see [ROADMAP.md](ROADMAP.md).

## Installation

### Current Supported Path: Build From Source

At the current repository state, build-from-source is the active installation path. The README does not assume that a published GitHub Release is already available for the revision you are reading.

Requirements:

- Rust `1.85` or newer
- A Unix-like environment

Build the CLI:

```bash
cargo build --release -p sfa-cli
```

The binary is produced at `target/release/sfa-cli`.

If you want the Quick Start examples below to work unchanged from the repository root, add that directory to your shell `PATH` first:

```bash
export PATH="$PWD/target/release:$PATH"
sfa-cli --version
```

### GitHub Release Archives

When a version tag is published, the release workflow uploads prebuilt archives for:

- Linux `x86_64`: `sfa-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- macOS `x86_64`: `sfa-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- macOS `arm64`: `sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz`

Each release also includes a matching `.sha256` file for every archive. Download the archive that matches your platform, download the corresponding checksum file, verify it with a SHA-256 tool available on your host, and then extract the archive:

```bash
shasum -a 256 -c sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz.sha256
# or: sha256sum -c sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz.sha256
tar -xzf sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz
```

Each archive contains the `sfa-cli` binary together with `README.md` and `LICENSE`.

## Quick Start

The commands below assume `sfa-cli` is already on your `PATH`. If you built from source and did not export `target/release` into `PATH`, replace `sfa-cli` with `./target/release/sfa-cli`.

Create an archive:

```bash
sfa-cli pack ./input ./archive.sfa --integrity strong
```

By default, `sfa-cli pack` uses `zstd` data frames at level `-3`.

Extract an archive:

```bash
sfa-cli unpack ./archive.sfa -C ./restore
```

Extract from standard input:

```bash
cat ./archive.sfa | sfa-cli unpack - -C ./restore
```

Emit machine-readable stats:

```bash
sfa-cli pack ./input ./archive.sfa --stats-format json
```

## Benchmark Snapshot

The repository benchmark baseline compares SFA and `tar` using the same codec settings on committed datasets. The current committed baseline was recorded on macOS `aarch64` with `/usr/bin/tar` (`bsdtar 3.5.3`), Homebrew `lz4` `1.10.0`, and Homebrew `zstd` `1.5.7`.

| Dataset | Codec | Command | SFA | `tar` | Relative Result |
| --- | --- | --- | ---: | ---: | --- |
| `small-text` | `lz4` | `pack` | `19 ms` | `95 ms` | SFA about `5.0x` faster |
| `small-text` | `lz4` | `unpack` | `15 ms` | `76 ms` | SFA about `5.1x` faster |
| `large-control` | `lz4` | `unpack` | `8 ms` | `81 ms` | SFA about `10.1x` faster |
| `large-control` | `zstd` | `pack` | `9 ms` | `133 ms` | SFA about `14.8x` faster |

These numbers are repository baseline evidence, not universal guarantees. See [benches/README.md](benches/README.md), [benches/results/README.md](benches/results/README.md), and [spec/verification-and-benchmark.md](spec/verification-and-benchmark.md) for the full matrix, methodology, and interpretation guidance.

## Features

- End-to-end `sfa-cli pack` and `sfa-cli unpack` workflows for Unix-like directory trees
- Manifest-first `.sfa` layout with header, manifest, frames, and an optional trailer
- `lz4` and `zstd` data codec support
- Support for regular files, directories, symlinks, and hardlinks
- Default restore of `mode` and `mtime` for regular files and directories
- Archive-side recording of `uid` and `gid`, with explicit opt-in owner restore policy
- Machine-readable command stats via `--stats-format json`
- Safe restore path built around `dirfd` / `openat`-style object creation
- `.sfa-untrusted` marker emission when `strong` trailer verification fails during restore

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
