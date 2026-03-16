## ADDED Requirements

### Requirement: SFA archives are manifest-first and sequentially readable
An SFA v1 archive SHALL consist of a fixed-length `HeaderV1`, exactly one `ManifestSection`, zero or more `DataFrame` records, and an optional `TrailerV1` only when strong integrity or explicit trailer output is enabled. A compliant reader MUST be able to parse the archive from start to end using sequential reads and MUST NOT require seek to locate the manifest or any frame payload.

#### Scenario: Reader consumes an archive from an HTTP-like stream
- **WHEN** an unpacker reads a valid `.sfa` archive from a source that only supports sequential `Read`
- **THEN** it parses `HeaderV1` first, `ManifestSection` second, and each `DataFrame` in order without seeking

### Requirement: HeaderV1 records wire-level execution metadata
`HeaderV1` SHALL be 128 bytes, encoded in little-endian form, and MUST record the archive version, data codec, manifest codec, integrity mode, hash algorithms, suggested parallelism, bundle planning parameters, entry/extent/bundle counts, manifest lengths, feature flags, manifest hash, and header CRC32.

#### Scenario: Reader validates the header before consuming the manifest
- **WHEN** an unpacker receives a header with an unsupported version or an invalid header CRC32
- **THEN** it fails the archive before reading `ManifestSection` and reports an incompatible or corrupted archive

### Requirement: ManifestSection defines the restore plan without compressed offsets
`ManifestSection` SHALL encode entries, extents, bundle plans, name arena data, and optional metadata blobs needed to reconstruct the directory tree. It MUST map file content to `bundle_id`, `file_offset`, `raw_offset_in_bundle`, and `raw_len`, and MUST NOT depend on compressed frame offsets or post-write backfilling.

#### Scenario: Unpacker builds a restore plan before frame decode
- **WHEN** a valid manifest is decoded
- **THEN** the unpacker can determine directory creation order, extent-to-file mappings, symlink targets, hardlink masters, and the expected bundle sequence before decoding any `DataFrame` payload

### Requirement: Integrity behavior is mode-aware
In `fast` integrity mode, the archive MUST validate header CRC32, manifest hash, and per-frame payload hashes. In `strong` integrity mode, the archive MUST additionally validate trailer-level archive totals and final archive hash before reporting success.

#### Scenario: Corrupted frame stops unpack early
- **WHEN** a `DataFrame` payload hash does not match the frame metadata
- **THEN** the unpacker aborts the archive and reports corruption without continuing to later frames
