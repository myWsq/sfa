# archive-pack Specification

## Purpose
TBD - created by archiving change bootstrap-sfa-v1. Update Purpose after archive.
## Requirements
### Requirement: Pack scans Unix directory trees deterministically
The pack operation SHALL scan the input tree with `lstat` semantics, MUST preserve relative path hierarchy, MUST order siblings by basename byte order with directories emitted before their contents, and MUST detect hardlink groups by `(st_dev, st_ino)` so that repeated packs of the same tree and configuration produce the same logical entry order.

#### Scenario: Repeated pack uses stable manifest ordering
- **WHEN** the same directory tree is packed twice with the same parameters
- **THEN** both runs produce the same entry ordering and the same bundle assignment plan

### Requirement: Pack plans bundles with stable linear aggregation
Pack SHALL exclude directories, symlinks, hardlinks, and empty files from raw bundle payloads, SHALL aggregate regular files smaller than `small_file_threshold` into bundle payloads until `bundle_target_bytes` would be exceeded, and SHALL split larger files into one or more single-file bundle chunks.

#### Scenario: Mixed small and large files are planned predictably
- **WHEN** the input tree contains multiple small regular files and one regular file larger than `small_file_threshold`
- **THEN** the small files are placed into aggregate bundles and the large file is emitted as one or more chunked bundle plans

### Requirement: Pack writes archives in manifest-first bundle order
After scan and planning complete, pack SHALL write `HeaderV1`, then `ManifestSection`, then `DataFrame` records in strictly increasing `bundle_id` order, and SHALL append `TrailerV1` only when strong integrity or explicit trailer output is enabled.

#### Scenario: Compression workers finish out of order
- **WHEN** encoded bundle results arrive at the writer out of `bundle_id` order
- **THEN** the archive writer buffers pending results and emits frames in ascending `bundle_id` order

### Requirement: Pack records supported entry types and restore metadata
Pack SHALL encode directory, regular file, symlink, and hardlink entries and MUST record each entry's path, mode, uid, gid, mtime, size, symlink target when applicable, and hardlink master identity when applicable.

#### Scenario: Tree includes symlink and hardlink entries
- **WHEN** the source directory contains a symlink and two hardlinked regular-file paths
- **THEN** the archive manifest records the symlink target, emits data extents only for the hardlink master, and records the hardlink follower's master entry identity

### Requirement: Pack exposes throughput-oriented configuration
The public pack API and CLI SHALL support selecting `lz4` or `zstd`, thread count, `bundle_target_bytes`, `small_file_threshold`, integrity mode, and metadata preservation policies required by v1.

#### Scenario: Caller selects custom codec and concurrency
- **WHEN** a caller runs pack with `zstd`, a custom thread count, and non-default bundle planning parameters
- **THEN** the resulting archive header records those execution parameters and the pack pipeline uses them for planning and encoding

