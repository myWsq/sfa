# SFA Format v1

Status: Frozen v1.0 protocol definition  
Freeze date: 2026-03-16  
Review record: `spec/format-v1-freeze-review.md`

This document is the authoritative wire-format specification for SFA v1 archives. Background discussion and alternative designs may continue to live under `sfa-tech-solution/`, but compatibility decisions for `HeaderV1`, `ManifestSection`, `DataFrame`, and `TrailerV1` are defined here.

## 1. Scope

SFA v1 defines a manifest-first archive format for Unix directory trees with these goals:

- Strict sequential-read decode without `seek`
- Stable bundle planning metadata ahead of frame decode
- Support for regular files, directories, symlinks, and hardlinks
- Optional strong end-of-archive verification without making trailer reads mandatory for normal restore

This document freezes the v1.0 on-disk layout and the minimum reader and writer behavior required for compatibility.

## 2. Physical Layout

```text
Archive
= HeaderV1
| ManifestSection
| DataFrame*
| TrailerV1?   // present only when integrity_mode = strong
```

Writers SHALL emit archive records in that exact order. Readers SHALL be able to consume the archive from start to finish using sequential reads only.

## 3. Common Encoding Rules

- All integer fields SHALL use little-endian encoding.
- Fixed-size structs SHALL occupy exactly the byte length defined in this document.
- Reserved bytes SHALL be written as zero in v1.0.
- Paths and symlink targets are stored as raw byte sequences in `NameArena`; they are not required to be UTF-8.
- Relative paths are reconstructed from `parent_id` plus the entry basename stored in `NameArena`.

## 4. HeaderV1

### 4.1 Size and Magic

- `HeaderV1` size: 128 bytes
- `magic`: `SFA\0\r\n\x1A\n`

### 4.2 Layout

| Offset | Size | Field | Type | Meaning |
|---:|---:|---|---|---|
| 0 | 8 | `magic` | `[u8; 8]` | archive magic |
| 8 | 2 | `header_len` | `u16` | MUST be `128` |
| 10 | 2 | `version_major` | `u16` | v1 major version, frozen as `1` |
| 12 | 2 | `version_minor` | `u16` | v1 minor version, frozen as `0` |
| 14 | 2 | `data_codec` | `u16` | data frame codec |
| 16 | 2 | `manifest_codec` | `u16` | manifest codec |
| 18 | 1 | `integrity_mode` | `u8` | archive integrity mode |
| 19 | 1 | `frame_hash_algo` | `u8` | per-frame raw-bundle hash algorithm |
| 20 | 1 | `manifest_hash_algo` | `u8` | manifest hash algorithm |
| 21 | 1 | `reserved0` | `u8` | MUST be zero |
| 22 | 2 | `suggested_parallelism` | `u16` | writer-suggested worker count |
| 24 | 4 | `bundle_target_bytes` | `u32` | planner bundle target size |
| 28 | 4 | `small_file_threshold` | `u32` | small-file aggregation threshold |
| 32 | 8 | `entry_count` | `u64` | manifest entry count |
| 40 | 8 | `extent_count` | `u64` | manifest extent count |
| 48 | 8 | `bundle_count` | `u64` | manifest bundle count |
| 56 | 8 | `manifest_raw_len` | `u64` | decoded manifest byte length |
| 64 | 8 | `manifest_encoded_len` | `u64` | encoded manifest byte length |
| 72 | 8 | `feature_flags` | `u64` | archive feature bitmap |
| 80 | 32 | `manifest_hash` | `[u8; 32]` | hash of decoded manifest bytes |
| 112 | 4 | `header_crc32` | `u32` | CRC32 over the full header with this field zeroed |
| 116 | 2 | `writer_version_major` | `u16` | writer implementation major version |
| 118 | 2 | `writer_version_minor` | `u16` | writer implementation minor version |
| 120 | 8 | `reserved1` | `[u8; 8]` | MUST be zero |

### 4.3 Enum Values

`data_codec`:

| Value | Meaning |
|---:|---|
| 0 | `none` |
| 1 | `lz4` |
| 2 | `zstd` |

`manifest_codec`:

| Value | Meaning |
|---:|---|
| 0 | `none` |
| 1 | `zstd` |

`integrity_mode`:

| Value | Meaning |
|---:|---|
| 0 | `off` |
| 1 | `fast` |
| 2 | `strong` |

`frame_hash_algo`:

| Value | Meaning |
|---:|---|
| 0 | `none` |
| 1 | `xxh3_64` |

`manifest_hash_algo`:

| Value | Meaning |
|---:|---|
| 0 | `none` |
| 1 | `blake3_256` |

### 4.4 Feature Flags

| Bit | Meaning |
|---:|---|
| 0 | archive contains at least one symlink |
| 1 | archive contains at least one hardlink |
| 2 | archive contains special files |
| 3 | archive contains metadata bytes in `MetaBlob` |
| 4 | archive contains `TrailerV1` |
| 5 | archive preserves owner metadata |
| 6-63 | reserved |

### 4.5 Required Behavior

- Readers SHALL reject headers with invalid magic, invalid `header_len`, or invalid `header_crc32`.
- v1.0 writers SHALL emit `version_major = 1` and `version_minor = 0`.
- `manifest_hash` SHALL be computed over decoded manifest bytes, not over encoded manifest bytes.
- `suggested_parallelism`, `bundle_target_bytes`, and `small_file_threshold` are diagnostic and tuning metadata; they do not alter parsing rules.

## 5. ManifestSection

### 5.1 Layout

```text
ManifestSection
= ManifestHeaderV1
| EntryRecordV1[entry_count]
| ExtentRecordV1[extent_count]
| BundlePlanRecordV1[bundle_count]
| NameArena[name_arena_bytes]
| MetaBlob[meta_blob_bytes]
```

The manifest SHALL fully describe the restore plan without storing compressed frame offsets.

### 5.2 ManifestHeaderV1

- Size: 64 bytes
- `magic`: `MFST`

| Offset | Size | Field | Type | Meaning |
|---:|---:|---|---|---|
| 0 | 4 | `magic` | `[u8; 4]` | manifest magic |
| 4 | 2 | `header_len` | `u16` | MUST be `64` |
| 6 | 2 | `flags` | `u16` | reserved for future manifest-level flags; v1.0 writers SHALL emit zero |
| 8 | 8 | `entry_count` | `u64` | number of entry records |
| 16 | 8 | `extent_count` | `u64` | number of extent records |
| 24 | 8 | `bundle_count` | `u64` | number of bundle plan records |
| 32 | 8 | `name_arena_bytes` | `u64` | byte length of `NameArena` |
| 40 | 8 | `meta_blob_bytes` | `u64` | byte length of `MetaBlob` |
| 48 | 16 | `reserved` | `[u8; 16]` | MUST be zero |

Readers SHALL reject manifests whose byte length does not match the counts recorded in `ManifestHeaderV1`.

### 5.3 EntryRecordV1

- Size: 96 bytes
- `entry_id` is implicit: the zero-based array index of the entry record
- root entry SHALL occupy `entry_id = 0`

| Offset | Size | Field | Type | Meaning |
|---:|---:|---|---|---|
| 0 | 4 | `parent_id` | `u32` | parent entry id, or `u32::MAX` for root |
| 4 | 1 | `kind` | `u8` | entry kind |
| 5 | 1 | `flags` | `u8` | entry flags |
| 6 | 2 | `reserved0` | `u16` | MUST be zero |
| 8 | 4 | `mode` | `u32` | Unix mode bits |
| 12 | 4 | `uid` | `u32` | owning user id |
| 16 | 4 | `gid` | `u32` | owning group id |
| 20 | 8 | `mtime_sec` | `i64` | mtime seconds |
| 28 | 4 | `mtime_nsec` | `u32` | mtime nanoseconds |
| 32 | 8 | `size` | `u64` | regular-file raw size, otherwise type-dependent |
| 40 | 4 | `name_off` | `u32` | basename offset in `NameArena` |
| 44 | 4 | `name_len` | `u32` | basename length |
| 48 | 4 | `link_off` | `u32` | symlink target offset in `NameArena`, or `0` |
| 52 | 4 | `link_len` | `u32` | symlink target length |
| 56 | 8 | `first_extent` | `u64` | index of first extent for this entry |
| 64 | 4 | `extent_count` | `u32` | number of extents for this entry |
| 68 | 4 | `hardlink_master_entry_id` | `u32` | hardlink master entry id, or `u32::MAX` |
| 72 | 4 | `dev_major` | `u32` | reserved for special files |
| 76 | 4 | `dev_minor` | `u32` | reserved for special files |
| 80 | 4 | `meta_off` | `u32` | metadata offset in `MetaBlob` |
| 84 | 4 | `meta_len` | `u32` | metadata length |
| 88 | 8 | `reserved1` | `[u8; 8]` | MUST be zero |

`kind`:

| Value | Meaning |
|---:|---|
| 0 | `root` |
| 1 | `directory` |
| 2 | `regular` |
| 3 | `symlink` |
| 4 | `hardlink` |
| 5 | `char_device` |
| 6 | `block_device` |
| 7 | `fifo` |

`flags`:

| Bit | Meaning |
|---:|---|
| 0 | explicit empty regular file |
| 1 | path validated |
| 2 | metadata present in `MetaBlob` |
| 3-7 | reserved |

### 5.4 ExtentRecordV1

- Size: 32 bytes

| Offset | Size | Field | Type | Meaning |
|---:|---:|---|---|---|
| 0 | 8 | `bundle_id` | `u64` | bundle containing this file segment |
| 8 | 8 | `file_offset` | `u64` | write offset within the target file |
| 16 | 4 | `raw_offset_in_bundle` | `u32` | offset within the decoded raw bundle |
| 20 | 4 | `raw_len` | `u32` | extent byte length |
| 24 | 4 | `flags` | `u32` | extent flags |
| 28 | 4 | `entry_id` | `u32` | owning entry id |

`flags`:

| Bit | Meaning |
|---:|---|
| 0 | this extent is the final extent for its file |
| 1 | this extent belongs to an aggregate bundle |
| 2 | this extent belongs to a single-file chunked bundle |
| 3-31 | reserved |

### 5.5 BundlePlanRecordV1

- Size: 32 bytes

| Offset | Size | Field | Type | Meaning |
|---:|---:|---|---|---|
| 0 | 8 | `bundle_id` | `u64` | bundle id |
| 8 | 4 | `raw_len` | `u32` | decoded bundle byte length |
| 12 | 4 | `file_count` | `u32` | number of files contributing data to this bundle |
| 16 | 4 | `extent_count` | `u32` | number of extents expected from this bundle |
| 20 | 1 | `kind` | `u8` | bundle kind |
| 21 | 1 | `flags` | `u8` | bundle flags |
| 22 | 10 | `reserved` | `[u8; 10]` | MUST be zero |

`kind`:

| Value | Meaning |
|---:|---|
| 0 | `aggregate` |
| 1 | `single_file` |

Writers SHALL emit bundle records in ascending `bundle_id` order. Readers SHALL use `bundle_id` order as the expected `DataFrame` order.

### 5.6 NameArena

`NameArena` stores raw byte sequences for:

- entry basenames
- symlink targets
- future path-adjacent metadata values

For v1.0:

- basenames SHALL NOT contain `/`
- basenames SHALL NOT be empty except for the root entry
- basenames and symlink targets SHALL NOT contain NUL
- symlink targets are stored as raw bytes and SHALL NOT be canonicalized during archive creation

### 5.7 MetaBlob

`MetaBlob` is a byte region reserved for extension metadata. v1.0 writers MAY leave it empty. When populated, entry-local metadata is addressed by `meta_off` plus `meta_len`.

## 6. DataFrame

### 6.1 Layout

```text
DataFrame
= FrameHeaderV1
| payload[encoded_len]
```

Each bundle SHALL produce exactly one `DataFrame`.

### 6.2 FrameHeaderV1

- Size: 48 bytes
- `magic`: `FRME`

| Offset | Size | Field | Type | Meaning |
|---:|---:|---|---|---|
| 0 | 4 | `magic` | `[u8; 4]` | frame magic |
| 4 | 2 | `header_len` | `u16` | MUST be `48` |
| 6 | 2 | `flags` | `u16` | reserved for future frame flags; v1.0 writers SHALL emit zero |
| 8 | 8 | `bundle_id` | `u64` | bundle id for this frame |
| 16 | 4 | `raw_len` | `u32` | decoded bundle byte length |
| 20 | 4 | `encoded_len` | `u32` | payload byte length |
| 24 | 8 | `frame_hash` | `u64` | hash of decoded raw bundle bytes |
| 32 | 16 | `reserved` | `[u8; 16]` | MUST be zero |

### 6.3 Required Behavior

- Frames SHALL appear in strictly increasing `bundle_id` order.
- `frame_hash` SHALL be computed over decoded raw bundle bytes, not over the encoded payload.
- Readers in `fast` or `strong` integrity mode SHALL reject frames whose decoded raw-bundle hash does not match `frame_hash`.

## 7. TrailerV1

### 7.1 Presence

`TrailerV1` SHALL be present when `integrity_mode = strong`. It MAY be omitted for `off` and `fast`.

Readers SHALL be able to restore archive contents without consuming the trailer. Trailer validation is an end-of-archive integrity step, not a prerequisite for reading `ManifestSection` or `DataFrame`.

### 7.2 Size and Magic

- Size: 64 bytes
- `magic`: `TRLR`

### 7.3 Layout

| Offset | Size | Field | Type | Meaning |
|---:|---:|---|---|---|
| 0 | 4 | `magic` | `[u8; 4]` | trailer magic |
| 4 | 2 | `header_len` | `u16` | MUST be `64` |
| 6 | 2 | `flags` | `u16` | reserved; v1.0 writers SHALL emit zero |
| 8 | 8 | `bundle_count` | `u64` | bundle count observed by the writer |
| 16 | 8 | `total_raw_bytes` | `u64` | total decoded frame bytes |
| 24 | 8 | `total_encoded_bytes` | `u64` | total encoded payload bytes |
| 32 | 32 | `archive_hash` | `[u8; 32]` | BLAKE3-256 over the concatenated frame-hash values in frame order |

### 7.4 Required Behavior

- Writers in `strong` mode SHALL emit exactly one trailer after the last `DataFrame`.
- Strong readers SHALL verify `bundle_count`, `total_raw_bytes`, `total_encoded_bytes`, and `archive_hash` before reporting success.

## 8. Sequential-Read Constraints

To remain compatible with SFA v1:

- readers MUST parse `HeaderV1` before reading any manifest bytes
- readers MUST parse `ManifestSection` before decoding `DataFrame` payloads
- readers MUST NOT rely on `seek` to locate manifest bytes or frame payloads
- writers MUST NOT require backfilled compressed offsets inside `ManifestSection`

The manifest therefore links files to bundles by `bundle_id`, `file_offset`, `raw_offset_in_bundle`, and `raw_len`, not by encoded archive offsets.

## 9. Path and Restore Constraints

An unpacker that reconstructs filesystem paths from `EntryRecordV1` and `NameArena` SHALL reject:

- absolute paths
- empty non-root basenames
- path segments equal to `.` or `..`
- names or link targets containing NUL
- path resolution strategies that allow an already-existing symlink to redirect parent traversal outside the output root

Symlink targets are restore data, not path-resolution inputs for parent lookup.

## 10. Compatibility Contract

This frozen document defines SFA v1.0 compatibility. Protocol-affecting changes to field layout, enum values, required validation, or fixture interpretation MUST be proposed through a new OpenSpec change and MUST update the canonical golden fixture set and freeze review record together.
