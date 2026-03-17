## 1. Expand the canonical golden corpus

- [x] 1.1 Define the representative fixture matrix and add fixture directories plus README coverage notes for codec, integrity, multi-bundle, and supported Unix-entry coverage.
- [x] 1.2 Generate and commit the archive, manifest dump, and stats snapshot assets for each new canonical fixture using the existing golden fixture flow.
- [x] 1.3 Update protocol-smoke expectations only where needed so every committed golden fixture directory is validated with the same required asset checks.

## 2. Add CLI regression coverage

- [x] 2.1 Add pack CLI regression tests for default dry-run JSON stats, default option reporting, and usage-error exit behavior.
- [x] 2.2 Add unpack CLI regression tests for missing-input failures and the supported `stdin` plus `--dry-run` behavior matrix.
- [x] 2.3 Add unpack CLI regression tests for overwrite-protected restores and the explicit overwrite-enabled success path.

## 3. Align verification guidance

- [x] 3.1 Update fixture, test, and release-facing documentation to describe the expanded canonical corpus and when fixture-backed regression assets must be refreshed.
- [x] 3.2 Run the relevant Rust tests and smoke entrypoints to verify the expanded golden corpus and CLI regression matrix end to end.
