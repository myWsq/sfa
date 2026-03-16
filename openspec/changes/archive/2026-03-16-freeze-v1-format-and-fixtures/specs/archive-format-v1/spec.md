## ADDED Requirements

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
