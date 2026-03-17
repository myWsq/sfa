## MODIFIED Requirements

### Requirement: Canonical golden fixtures anchor the frozen wire format
The repository SHALL include a canonical golden fixture set for SFA v1 under `tests/fixtures/golden/` that contains committed `.sfa` archive assets together with stable decoded metadata and fixture documentation. The fixture set MUST collectively cover every data codec supported by v1 writers, every integrity mode exposed by the v1 CLI, at least one archive whose manifest restores more than one data frame or bundle, and the v1 Unix entry semantics already supported for regular files, directories, symlinks, and hardlinks. Each committed fixture MUST identify its input corpus or source tree, fixed generation parameters, the protocol dimensions it is intended to cover, and the decoded archive summary used for protocol regression checks.

#### Scenario: Protocol freeze corpus covers the frozen v1 surface
- **WHEN** a maintainer inspects the committed golden fixture directories in a clean checkout
- **THEN** they can find representative fixtures that collectively exercise codec, integrity, multi-bundle, and supported Unix-entry coverage without relying on undocumented regeneration steps

#### Scenario: New fixture additions remain reviewable
- **WHEN** a contributor updates or adds a committed golden fixture
- **THEN** the fixture directory includes its archive asset, decoded metadata, input tree, and README coverage notes so reviewers can understand why that fixture belongs in the canonical corpus
