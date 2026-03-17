## MODIFIED Requirements

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
