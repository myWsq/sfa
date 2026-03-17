## MODIFIED Requirements

### Requirement: Pack records supported entry types and restore metadata
Pack SHALL encode directory, regular file, symlink, and hardlink entries and MUST record each entry's path, mode, uid, gid, mtime, size, symlink target when applicable, and hardlink master identity when applicable. For the supported v1 Unix metadata contract, `mode` and `mtime` are part of the default restore behavior for regular files and directories, while stored `uid` and `gid` values represent owner metadata that unpack MAY apply only when owner restoration is explicitly requested. The archive header SHALL mark owner-preservation intent only when pack is invoked with owner preservation enabled. v1 pack MUST NOT claim xattrs, ACLs, or special-file metadata as part of this contract.

#### Scenario: Tree includes symlink and hardlink entries
- **WHEN** the source directory contains a symlink and two hardlinked regular-file paths
- **THEN** the archive manifest records the symlink target, emits data extents only for the hardlink master, and records the hardlink follower's master entry identity

#### Scenario: Owner-preservation intent is explicit
- **WHEN** a caller packs the same supported Unix tree once with owner preservation disabled and once with it enabled
- **THEN** both archives record the entry `uid` and `gid` fields, and only the owner-preserving run marks owner-preservation intent in archive metadata
