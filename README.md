# SFA

SFA, short for Small File Archive, is a manifest-first archive format and CLI for Unix directory trees with many small files. It is designed for deterministic scanning, sequential reads, integrity validation, and ordered restore behavior without treating `tar` compatibility as the primary goal.

On the current committed macOS `aarch64` benchmark baseline for the generated `node-modules-100k` workload, SFA packs about `34.9x` faster than `tar`, unpacks about `15.9x` faster, and produces an archive about `2.2x` smaller.

SFA is not intended to be a drop-in replacement for `tar`. It focuses on a narrower problem: reliable local archiving and restore workflows for Unix-like directory trees where metadata, ordering, and small-file behavior matter.

## When To Use SFA

SFA is a good fit when you need:

- Fast pack and unpack workflows for Unix directory trees with many small files
- Deterministic directory scanning and stable bundle planning
- Sequential archive reads without seek-dependent restore logic
- Benchmarkable comparisons against a canonical TAR baseline that matches the default SFA compression profile
- Path safety checks, integrity validation, and explicit restore-policy controls

SFA is not yet the right fit when you need:

- Full Unix extended metadata coverage such as xattrs, ACLs, or device file restore
- Fully equivalent behavior on non-Unix platforms
- crates.io distribution
- Signed or notarized macOS binaries

## Project Status

SFA is under active development and is currently preparing its first stable `v1.0.0` release.

- `format-v1` is frozen and serves as the current compatibility boundary.
- Milestones M0 through M3 for the current stable-v1 scope are complete.
- The current release train keeps xattrs, ACLs, special-file restore, and broader Unix extensions deferred to post-v1 work.

For milestone details and current priorities, see [ROADMAP.md](ROADMAP.md).

## Installation

Published releases support two managed installation paths on the current binary
matrix:

- Linux `x86_64`: `sfa-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- macOS `x86_64`: `sfa-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- macOS `arm64`: `sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz`

The Homebrew tap and install script both resolve these same GitHub Release
archives and checksum files. macOS binaries remain unsigned and not notarized.

### Homebrew Tap

```bash
brew install myWsq/sfa/sfa-cli
brew upgrade myWsq/sfa/sfa-cli
```

Homebrew installs the packaged `sfa-cli` binary from the project-owned tap and
chooses the matching release archive for the current supported host.

### Install Script

Download and run the public installer script. It resolves the current host,
downloads the matching archive plus checksum, verifies the archive, and installs
`sfa-cli` into `"$HOME/.local/bin"` by default.

```bash
curl -fsSLo install-sfa.sh https://raw.githubusercontent.com/myWsq/sfa/main/install.sh
sh install-sfa.sh
```

Install a specific release or choose a different destination directory:

```bash
sh install-sfa.sh --version v1.0.0 --bin-dir "$HOME/.local/bin"
```

If your shell does not already include `"$HOME/.local/bin"` on `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
sfa-cli --version
```

### Direct Release Archives

If you prefer to inspect the archive manually, download the release asset that
matches your platform together with its checksum file, then verify and extract
it:

```bash
shasum -a 256 -c sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz.sha256
# or: sha256sum -c sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz.sha256
tar -xzf sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz
```

Each archive contains the `sfa-cli` binary together with `README.md` and `LICENSE`.

### Build From Source

Build from source if you are testing an unreleased revision, targeting an
unsupported host, or you prefer a local Rust toolchain flow.

Requirements:

- Rust `1.85` or newer
- A Unix-like environment

```bash
cargo build --release -p sfa-cli
```

The binary is produced at `target/release/sfa-cli`.

If you want the Quick Start examples below to work unchanged from the repository
root, add that directory to your shell `PATH` first:

```bash
export PATH="$PWD/target/release:$PATH"
sfa-cli --version
```

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

The repository benchmark baseline uses the generated `node-modules-100k` workload under `benches/workloads/node-modules-100k/`. That workload materializes a deterministic nested dependency tree with `10,560` generated packages and `105,601` regular files, then compares default `sfa pack` / `sfa unpack` commands against `tar | zstd --fast=3` on the same tree.

The committed JSON baseline was recorded on macOS `aarch64` with `/usr/bin/tar` (`bsdtar 3.5.3`) and Homebrew `zstd` `1.5.7`. It reports wall time, files/s, MiB/s, archive size, and SFA pack/unpack observability for both pack and unpack.

| Workload | Measurement | SFA | `tar` | Relative Result |
| --- | --- | ---: | ---: | --- |
| `node-modules-100k` | `pack` | `9.8 s` | `340.6 s` | SFA about `34.9x` faster |
| `node-modules-100k` | `unpack` | `15.3 s` | `242.2 s` | SFA about `15.9x` faster |
| `node-modules-100k` | archive size | `5.5 MiB` | `12.3 MiB` | SFA about `2.2x` smaller |

These numbers are repository baseline evidence, not universal guarantees.

See [benches/README.md](benches/README.md), [benches/results/README.md](benches/results/README.md), and [spec/verification-and-benchmark.md](spec/verification-and-benchmark.md) for the workload contract, regeneration workflow, cache-warming caveats, and interpretation guidance.

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
bash tests/scripts/run_distribution_smoke.sh
cargo run -p sfa-bench --bin tar_vs_sfa -- --dry-run --output benches/results/latest.json
```

See [RELEASING.md](RELEASING.md) for the release process and quality gates.

## Repository Layout

- `crates/sfa-core`: archive format, codecs, integrity, planning, and ordered readers
- `crates/sfa-unixfs`: Unix filesystem scan, archive, and restore implementation
- `crates/sfa-cli`: command-line entry point
- `crates/sfa-bench`: benchmark runner and fixture dump tooling
- `install.sh`: public managed installer for published release binaries
- `spec/`: protocol and verification specifications
- `scripts/release`: release-distribution helpers for formula generation, validation, and tap publication
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
