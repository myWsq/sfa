# SFA

Small-file archives that leave `tar` behind.

```text
+--------------------------------+    +--------------------------------+
| tar | zstd --fast=3            |    | SFA                            |
| long file-by-file stream       |    | manifest + bundles + frames    |
|                                |    |                                |
| many tiny files                |    | many tiny files                |
| [f][f][f][f][f][f]             |    | [f][f][f][f][f][f]            |
|             |                  |    |             |                  |
|             v                  |    |         scan once              |
|      one long stream           |    |             |                  |
|             |                  |    |  manifest + bundle plan        |
|             v                  |    |             |                  |
| restore plan emerges late      |    | [bundle A] [bundle B] [bundle C]
|             |                  |    |             |                  |
|           unpack               |    |       ordered frames           |
|                                |    |             |                  |
|                                |    |     sequential unpack          |
+--------------------------------+    +--------------------------------+
```

On the committed macOS `aarch64` `node-modules-100k` baseline, SFA packs about `34.9x` faster than `tar`, unpacks about `15.9x` faster, and produces an archive about `2.2x` smaller than `tar | zstd --fast=3`.

SFA is a CLI and archive format for Unix directory trees with many small files.

## Why SFA

- Much faster than `tar` on Unix trees dominated by small files
- Built for `node_modules`, dependency caches, vendored source trees, and generated outputs
- Turns many small files into bundle-sized work units
- Unpacks with strict sequential reads and supports `stdin` input
- Supports machine-readable stats, integrity validation, and safe restore behavior

## Install

Published binaries are currently available for:

- Linux `x86_64`
- macOS `x86_64`
- macOS `arm64`

### Homebrew

```bash
brew tap myWsq/sfa
brew install sfa
```

### Install script

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
sfa --version
```

### Build from source

Requirements:

- Rust `1.85` or newer
- A Unix-like environment

```bash
cargo build --release -p sfa-cli
export PATH="$PWD/target/release:$PATH"
sfa --version
```

## Quick Start

Create an archive:

```bash
sfa pack ./input ./archive.sfa --integrity strong
```

Extract an archive:

```bash
sfa unpack ./archive.sfa -C ./restore
```

Extract from standard input:

```bash
cat ./archive.sfa | sfa unpack - -C ./restore
```

Emit machine-readable stats:

```bash
sfa pack ./input ./archive.sfa --stats-format json
```

## Best For

Use SFA when you need:

- Fast local archive and restore workflows for Unix directory trees with many small files
- Deterministic scanning and stable bundle planning
- Sequential archive reads without seek-dependent unpack logic
- Benchmarkable comparisons against `tar | zstd --fast=3`
- Explicit integrity and restore-policy controls

SFA is not yet the right fit when you need:

- Full Unix extended metadata coverage such as xattrs, ACLs, or device file restore
- Full `tar` format compatibility
- Fully equivalent behavior on non-Unix platforms
- Signed or notarized macOS binaries

SFA is not a drop-in replacement for `tar`. It is optimized for reliable local pack/unpack workflows where small-file throughput, ordered restore, and safety matter more than `tar` byte compatibility.

## Benchmark Snapshot

The repository baseline uses the generated `node-modules-100k` workload under `benches/workloads/node-modules-100k/`. That workload materializes a deterministic nested dependency tree with `10,560` generated packages and `105,601` regular files, then compares default `sfa pack` / `sfa unpack` commands against `tar | zstd --fast=3` on the same tree.

| Workload | Measurement | SFA | `tar` | Relative Result |
| --- | --- | ---: | ---: | --- |
| `node-modules-100k` | `pack` | `9.8 s` | `340.6 s` | SFA about `34.9x` faster |
| `node-modules-100k` | `unpack` | `15.3 s` | `242.2 s` | SFA about `15.9x` faster |
| `node-modules-100k` | archive size | `5.5 MiB` | `12.3 MiB` | SFA about `2.2x` smaller |

These numbers are repository baseline evidence, not universal guarantees.

See [benches/README.md](benches/README.md), [benches/results/README.md](benches/results/README.md), and [spec/verification-and-benchmark.md](spec/verification-and-benchmark.md) for workload details, regeneration workflow, and interpretation guidance.

## Why It Is Fast

SFA keeps the user path simple with two commands, `pack` and `unpack`, but the format is built around a different execution model than `tar`:

- Scan once and build a manifest plus bundle plan
- Write archive records in order: header, manifest, frames, optional trailer
- Decode and restore sequentially without seek-dependent archive traversal
- Restore paths with `dirfd` / `openat`-style safe object creation

This is the main design trade: SFA keeps Unix tree semantics, but uses a bundle-oriented internal structure instead of `tar`'s file-by-file layout.

## Status

SFA is preparing its first stable `v1.0.0` release.

- `format-v1` is frozen and is the current compatibility boundary
- Milestones M0 through M3 for the stable-v1 scope are complete
- xattrs, ACLs, special-file restore, and broader Unix extensions remain deferred to post-v1 work

## Documentation

- [ROADMAP.md](ROADMAP.md): release status and short-term priorities
- [spec/README.md](spec/README.md): protocol and verification entry point
- [RELEASING.md](RELEASING.md): release checklist and GitHub release process
- [CHANGELOG.md](CHANGELOG.md): repository change history
- [sfa-tech-solution/README.md](sfa-tech-solution/README.md): deeper technical design background

## Contributing

Contributions are welcome.

For larger changes, start from the roadmap and spec boundary:

- [ROADMAP.md](ROADMAP.md)
- [spec/README.md](spec/README.md)
- [sfa-tech-solution/README.md](sfa-tech-solution/README.md)

If your change affects the archive format, fixtures, benchmark baselines, or restore semantics, keep implementation, specs, and verification assets in sync.

## License

SFA is released under the MIT License. See [LICENSE](LICENSE).
