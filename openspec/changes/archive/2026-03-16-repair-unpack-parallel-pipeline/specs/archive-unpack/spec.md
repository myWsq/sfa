## MODIFIED Requirements

### Requirement: Unpack consumes sequential streams without seek
The unpack operation SHALL accept local file streams and HTTP-like sequential streams, MUST parse `HeaderV1` and `ManifestSection` before frame decode, MUST honor a caller-provided thread override or `suggested_parallelism` when no override is supplied, and SHALL execute multi-bundle restore work through a bounded parallel pipeline without requiring seek.

#### Scenario: Fragmented input still decodes correctly
- **WHEN** a valid archive is delivered to the unpacker in one-byte or randomly fragmented chunks
- **THEN** the unpacker completes header parsing, manifest parsing, frame consumption, and restore without requiring seek

#### Scenario: Thread override changes effective unpack worker count
- **WHEN** a caller unpacks a valid multi-bundle archive with an explicit thread override
- **THEN** the unpacker uses that override as the effective worker count for bundle decode and restore scheduling instead of only echoing it in output statistics

### Requirement: Unpack restores supported Unix entries in safe order
Unpack SHALL create directories before restoring file content, SHALL write regular-file extents according to `bundle_id` and `file_offset`, MAY decode and scatter multiple bundles concurrently, SHALL create symlinks only after parent directories exist, SHALL create hardlinks only after the master file has been materialized, and SHALL finalize directory metadata after child restoration completes.

#### Scenario: Mixed tree is restored with correct ordering
- **WHEN** an archive contains directories, regular files, a symlink, and a hardlink
- **THEN** the unpacker materializes the tree in directory-first, data-write, symlink/hardlink, and directory-finalize order so that all supported entries restore successfully

#### Scenario: Concurrent bundle work preserves restore semantics
- **WHEN** multiple bundles are decoded and scattered concurrently during unpack
- **THEN** the restored files remain byte-for-byte correct and no symlink, hardlink, or directory-finalize step runs before its required data or master entry exists
