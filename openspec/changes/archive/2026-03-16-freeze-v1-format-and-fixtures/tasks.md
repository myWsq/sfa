## 1. Freeze The Authoritative Protocol Text

- [x] 1.1 Replace the placeholder `spec/format-v1.md` with the normative SFA v1 protocol text covering header, manifest, frames, trailer, integrity modes, and sequential-read constraints.
- [x] 1.2 Reconcile the frozen spec wording with the current technical-solution docs and spec index so the repository has one clear protocol source of truth.

## 2. Commit Canonical Golden Fixtures

- [x] 2.1 Define the canonical directory layout and README conventions for frozen protocol fixtures under `tests/fixtures/golden/`.
- [x] 2.2 Generate and commit the first canonical golden fixture set, including a `.sfa` archive, decoded metadata dump, and fixture summary from a stable input corpus.
- [x] 2.3 Update or extend the existing fixture-generation helper so maintainers can reproduce the committed golden assets with fixed parameters.

## 3. Add Freeze Review Traceability

- [x] 3.1 Add a repository review record that references the frozen `spec/format-v1.md`, the committed golden fixtures, the freeze date, and the explicitly deferred benchmark follow-up.
- [x] 3.2 Update adjacent documentation such as `README.md`, `ROADMAP.md`, or `spec/README.md` so the repository advertises the frozen-format status and points readers to the review record.

## 4. Guard The Frozen Assets In Smoke Checks

- [x] 4.1 Extend `tests/scripts/run_protocol_smoke.sh` to validate that the committed golden fixture files exist and can be parsed by the current reader.
- [x] 4.2 Add smoke-level consistency checks that compare the decoded fixture metadata against the committed golden summaries and fail on protocol drift.
