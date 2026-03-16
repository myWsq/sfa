## ADDED Requirements

### Requirement: Unpack consumes sequential streams without seek
The unpack operation SHALL accept local file streams and HTTP-like sequential streams, MUST parse `HeaderV1` and `ManifestSection` before frame decode, and MAY override `suggested_parallelism` with caller-provided thread settings.

#### Scenario: Fragmented input still decodes correctly
- **WHEN** a valid archive is delivered to the unpacker in one-byte or randomly fragmented chunks
- **THEN** the unpacker completes header parsing, manifest parsing, and frame consumption without requiring seek

### Requirement: Unpack restores supported Unix entries in safe order
Unpack SHALL create directories before restoring file content, SHALL write regular-file extents according to `bundle_id` and `file_offset`, SHALL create symlinks only after parent directories exist, SHALL create hardlinks only after the master file has been materialized, and SHALL finalize directory metadata after child restoration completes.

#### Scenario: Mixed tree is restored with correct ordering
- **WHEN** an archive contains directories, regular files, a symlink, and a hardlink
- **THEN** the unpacker materializes the tree in directory-first, data-write, symlink/hardlink, and directory-finalize order so that all supported entries restore successfully

### Requirement: Unpack rejects path escape and unsafe node creation
Unpack MUST reject absolute paths, `..` segments, empty path segments, NUL bytes, and attempts to traverse through existing symlinks under the output root. It MUST reject special file restoration by default.

#### Scenario: Archive attempts to escape the output root
- **WHEN** an archive entry path is `../../etc/passwd` or otherwise resolves outside the chosen output root
- **THEN** the unpacker fails the archive and does not create or modify files outside the output root

### Requirement: Unpack applies metadata according to policy
Unpack SHALL restore mode and mtime by default, MUST ignore owner restoration for non-root callers unless an explicit preserve-owner policy is enabled, and MUST expose overwrite, owner-restore, and integrity policies through its public API and CLI.

#### Scenario: Non-root caller unpacks without owner restore
- **WHEN** a non-root caller unpacks a valid archive with default restore settings
- **THEN** file data, directory structure, mode, and mtime are restored while uid and gid ownership changes are skipped safely

### Requirement: Integrity failures abort unpack
Unpack MUST fail on invalid header, manifest, frame, or strong trailer validation and MUST report the archive as untrusted instead of silently succeeding.

#### Scenario: Strong trailer validation fails
- **WHEN** an archive in `strong` integrity mode has a trailer hash mismatch after all frames are processed
- **THEN** the unpacker exits with an integrity error and does not report a successful restore
