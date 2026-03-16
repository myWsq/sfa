# archive-format-v1 Specification

## Purpose
TBD - created by archiving change bootstrap-sfa-v1. Update Purpose after archive.
## Requirements
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

### Requirement: Frozen format specification is authoritative for SFA v1
Before the first SFA v1 release, the repository SHALL provide a normative `spec/format-v1.md` that defines the frozen v1 wire-format semantics for `HeaderV1`, `ManifestSection`, `DataFrame`, `TrailerV1`, integrity behavior, and sequential-read constraints. Once this document is declared frozen, protocol-affecting changes MUST be proposed through a new OpenSpec change instead of being updated ad hoc in implementation notes or placeholder files.

#### Scenario: Maintainer checks the frozen protocol source of truth
- **WHEN** a maintainer needs to determine whether a reader or writer change alters SFA v1 compatibility
- **THEN** `spec/format-v1.md` is the authoritative protocol reference and no placeholder document is treated as equally normative

### Requirement: Canonical golden fixtures anchor the frozen wire format
The repository SHALL include a canonical golden fixture set for SFA v1 under `tests/fixtures/golden/` that contains committed `.sfa` archive assets together with stable decoded metadata and fixture documentation. Each committed fixture MUST identify its input corpus or source tree, fixed generation parameters, and the decoded archive summary used for protocol regression checks.

#### Scenario: Protocol freeze includes a reproducible fixture corpus
- **WHEN** the first protocol freeze is reviewed
- **THEN** reviewers can inspect at least one committed golden fixture and its paired decoded metadata without regenerating assets from undocumented inputs

### Requirement: Protocol smoke validates frozen fixture assets
The protocol smoke entrypoint SHALL consume the committed golden fixture set and MUST fail if a required archive asset is missing, if a committed archive cannot be parsed by the current reader, or if the decoded archive summary no longer matches the committed fixture metadata.

#### Scenario: Protocol regression is detected by smoke checks
- **WHEN** a code change causes the reader or manifest decoder to diverge from a committed golden fixture
- **THEN** the protocol smoke check fails before the change can be treated as preserving the frozen v1 format

### Requirement: Protocol freeze is review-traceable in the repository
The repository SHALL contain a protocol freeze review record that references the frozen `spec/format-v1.md`, the committed golden fixture set used for the freeze decision, the freeze date, and the deferred follow-up items that remain outside the protocol-freeze scope. The review record MUST make clear that benchmark baselines and real dataset population are tracked separately from the protocol-freeze gate.

#### Scenario: Contributor audits why the protocol was frozen
- **WHEN** a contributor needs to understand what was frozen for SFA v1 and what was intentionally deferred
- **THEN** the repository contains a review record that links the frozen spec, canonical fixtures, and deferred benchmark work in one traceable place

