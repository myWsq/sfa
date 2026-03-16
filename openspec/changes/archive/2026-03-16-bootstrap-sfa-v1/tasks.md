## 1. Workspace Bootstrap

- [x] 1.1 Create the Rust workspace root and initialize `sfa-core`, `sfa-unixfs`, `sfa-cli`, and `sfa-bench` crates with shared lint, test, and dependency configuration.
- [x] 1.2 Define shared configuration, error, codec, integrity, and stats types used across pack, unpack, CLI, and benchmark flows.
- [x] 1.3 Add repository skeleton directories for `spec/`, `tests/`, `benches/`, and golden/corruption/streaming fixtures so later tasks have stable locations.

## 2. Wire Format And Planning

- [x] 2.1 Implement `HeaderV1`, frame header, trailer, and related validation logic for the `.sfa` wire format.
- [x] 2.2 Implement `ManifestSection` encoding and decoding for entries, extents, bundle plans, name arena, and metadata blob placeholders.
- [x] 2.3 Build the deterministic Unix scanner with `lstat` semantics, stable entry ordering, and hardlink master detection.
- [x] 2.4 Implement the stable linear bundle planner for aggregate bundles, chunked large files, and empty-file handling.
- [x] 2.5 Add fixture dump and golden archive generation helpers for protocol regression coverage.

## 3. Pack Pipeline

- [x] 3.1 Implement bundle readers that assemble raw bundle buffers from the scan and plan output.
- [x] 3.2 Add codec adapters for `lz4` and `zstd`, plus fast and strong integrity hashing support.
- [x] 3.3 Implement the parallel pack pipeline with bounded queues and an ordered writer that emits header, manifest, frames, and optional trailer.
- [x] 3.4 Expose a library-level pack API that returns execution statistics required by the CLI and benchmark harness.

## 4. Unpack Pipeline And Safe Restore

- [x] 4.1 Implement the sequential archive reader state machine for sync `Read` inputs, including header and manifest validation before frame decode.
- [x] 4.2 Implement decoded bundle scatter with `write_at`-style writes, extent tracking, and bounded file-handle caching.
- [x] 4.3 Implement safe Unix restore operations for directories, regular files, symlinks, and hardlinks using dirfd-style path handling and policy-driven metadata application.
- [x] 4.4 Implement corruption handling and strong trailer validation so unpack aborts cleanly on invalid archives.

## 5. CLI Surface

- [x] 5.1 Implement `sfa pack` and `sfa unpack` commands with argument parsing for codec, threads, bundle parameters, integrity mode, output paths, overwrite policy, and owner restore policy.
- [x] 5.2 Add stable human-readable summaries, structured stats output, and non-zero exit behavior for parse, integrity, IO, and safety failures.
- [x] 5.3 Wire the CLI to the library pack and unpack APIs without leaking internal module details into the command layer.

## 6. Verification And Benchmarking

- [x] 6.1 Add roundtrip integration tests for empty trees, nested directories, mixed small and large files, symlinks, hardlinks, and empty files.
- [x] 6.2 Add fragmented-stream, corruption, and path-safety tests that cover sequential input, invalid headers/manifests/frames, and output-root escape attempts.
- [x] 6.3 Implement the `tar + same codec` benchmark harness for the small-text, small-binary, and large-file datasets with comparable metrics output.
- [x] 6.4 Add CI or project scripts that run the protocol tests, integration suite, and benchmark smoke checks needed before the first v1 release.
