# archive-unpack Specification

## Purpose
TBD - created by archiving change bootstrap-sfa-v1. Update Purpose after archive.
## Requirements
### Requirement: Unpack consumes sync readers and sequential streams without seek
The unpack operation SHALL accept local file streams, sync `Read` readers, and HTTP-like sequential streams, MUST parse `HeaderV1` and `ManifestSection` before frame decode, MUST honor a caller-provided thread override or `suggested_parallelism` when no override is supplied, and SHALL execute restore work through a bounded `FrameReadQueue -> DecodeQueue -> ScatterQueue -> finalize` pipeline without requiring seek.

#### Scenario: Fragmented sync reader still decodes correctly
- **WHEN** a valid archive is delivered to the unpacker through a fragmented `Read` implementation
- **THEN** the unpacker completes header parsing, manifest parsing, frame consumption, decode, scatter, and finalize without requiring seek

#### Scenario: Path-based unpack delegates to reader-based unpack
- **WHEN** a caller invokes the path-based unpack entry with a local archive file
- **THEN** the implementation opens that file and delegates the real unpack work to the sync `Read` entry rather than maintaining a separate execution path

#### Scenario: Explicit thread override drives worker counts
- **WHEN** a caller unpacks a valid multi-bundle archive with an explicit thread override
- **THEN** the unpacker uses that override as the effective worker count for decode and scatter scheduling instead of only echoing it in output statistics

### Requirement: Unpack restores Unix entries via safe dirfd-style operations
Unpack SHALL create directories before restoring file content, SHALL create/open regular files relative to the output root using dirfd-style operations, SHALL write regular-file extents according to `bundle_id` and `file_offset`, SHALL create symlinks and hardlinks relative to parent directories without using path-string-based escape-prone APIs, and SHALL finalize directory metadata after child restoration completes.

#### Scenario: Malicious symlink in output root does not allow escape
- **WHEN** the output root already contains a symlink on an intermediate path segment that points outside the root
- **THEN** the unpacker rejects restoration through that segment and fails without writing outside the output root

#### Scenario: Mixed tree is restored in safe order
- **WHEN** an archive contains directories, regular files, a symlink, and a hardlink
- **THEN** the unpacker restores directories first, writes regular-file content next, creates symlink and hardlink entries only after prerequisites exist, and finalizes directory metadata last

#### Scenario: Directory owner metadata is restored when policy allows
- **WHEN** a root caller unpacks a valid archive with owner restore enabled
- **THEN** both regular files and directories apply stored uid/gid metadata during finalize, while symlink-own metadata remains unsupported

### Requirement: Unpack rejects path escape and unsafe node creation
Unpack MUST reject absolute paths, `..` segments, empty path segments, NUL bytes, and attempts to traverse through existing symlinks under the output root. It MUST reject special file restoration by default.

#### Scenario: Archive attempts to escape the output root
- **WHEN** an archive entry path is `../../etc/passwd` or otherwise resolves outside the chosen output root
- **THEN** the unpacker fails the archive and does not create or modify files outside the output root

### Requirement: Unpack applies metadata according to policy
Unpack SHALL restore mode and mtime for regular files and directories by default, SHALL treat stored uid and gid values as opt-in owner metadata, MUST attempt uid and gid restoration only when an explicit preserve-owner policy is selected and the effective process uid is root, and MUST leave ownership unchanged on the default and explicit no-owner paths. v1 unpack SHALL NOT restore symlink ownership, xattrs, ACLs, or special files as part of this metadata contract. Unpack MUST expose overwrite, owner-restore, and integrity policies through its public API and CLI.

#### Scenario: Non-root caller unpacks without owner restore
- **WHEN** a non-root caller unpacks a valid archive with default restore settings
- **THEN** file data, directory structure, mode, and mtime are restored while uid and gid ownership changes are skipped safely

#### Scenario: Directory owner metadata is restored when policy allows
- **WHEN** a root caller unpacks a valid archive with owner restore enabled
- **THEN** both regular files and directories apply stored uid/gid metadata during finalize, while symlink-own metadata remains unsupported

#### Scenario: Explicit no-owner policy skips stored owner metadata
- **WHEN** a root caller unpacks a valid archive with an explicit no-owner restore policy
- **THEN** file data, directory structure, mode, and mtime are restored while stored uid and gid values remain unapplied

### Requirement: Integrity failures abort unpack
Unpack MUST fail on invalid header, manifest, frame, or strong trailer validation and MUST report the archive as untrusted instead of silently succeeding.

#### Scenario: Corrupted frame payload fails unpack
- **WHEN** an archive frame payload is corrupted so that the decoded frame hash does not match the recorded frame hash
- **THEN** the unpacker exits with an integrity error and does not report a successful restore

### Requirement: Strong trailer failures mark restored output as untrusted
When strong integrity verification fails after payload restoration has already occurred, the unpacker SHALL fail the command and SHALL leave a stable marker in the output root indicating that the restored contents are not trusted.

#### Scenario: Trailer mismatch writes output marker
- **WHEN** a valid archive reaches trailer verification and the trailer hash does not match
- **THEN** the unpacker returns an integrity error and writes `.sfa-untrusted` in the output root containing a stable failure message

#### Scenario: Successful unpack clears stale untrusted marker
- **WHEN** an output root already contains `.sfa-untrusted` from an earlier failed run and a later unpack succeeds
- **THEN** the unpacker removes the stale marker before reporting success

