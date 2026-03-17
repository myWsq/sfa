# unpack-performance Specification

## Purpose
TBD - created by archiving change optimize-unpack-setup-and-small-file-scatter. Update Purpose after archive.
## Requirements
### Requirement: Unpack defers non-empty regular-file preparation until worker execution
The unpack implementation SHALL limit pre-pipeline setup for non-empty regular files to manifest-derived restore planning and any directory creation required for safe restore ordering. It MUST NOT require eager parent-directory handle opening, per-file descriptor preparation, or equivalent restore work for every non-empty regular file before the bounded decode/scatter pipeline starts.

#### Scenario: Multi-bundle small-file archive enters the pipeline without eager regular-file preparation
- **WHEN** a caller unpacks a valid archive containing many non-empty regular files spread across multiple bundles
- **THEN** the implementation completes header parsing, manifest parsing, restore-target planning, and required directory creation before starting the decode/scatter pipeline without first opening or preparing every regular file under the output root

### Requirement: Small-file scatter restores regular files through lazy syscall-efficient paths
For non-empty single-extent regular files, unpack SHALL restore file content and apply required regular-file metadata through a worker-owned create/write/finalize path that does not depend on an earlier serial preparation pass. For non-empty multi-extent regular files, unpack SHALL lazily acquire reusable file handles during extent writes, MUST preserve bounded descriptor usage, and SHALL defer final metadata application until data restoration completes.

#### Scenario: Single-extent regular file avoids a separate serial prepare phase
- **WHEN** a valid archive contains a non-empty regular file represented by exactly one extent
- **THEN** scatter workers create or open that file on demand, write its content, apply required mode and mtime metadata, and complete restore without requiring an earlier serial descriptor-preparation phase

#### Scenario: Multi-extent regular file keeps bounded lazy handle reuse
- **WHEN** a valid archive contains a non-empty regular file represented by multiple extents
- **THEN** scatter workers acquire the file handle lazily on the first extent write, reuse bounded cached handles during the remaining writes, and defer final metadata application until the file's data restoration is complete

