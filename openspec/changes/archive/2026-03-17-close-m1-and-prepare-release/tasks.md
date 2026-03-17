## 1. Restore release-gate compliance

- [x] 1.1 Fix the current formatting drift in the repository so `cargo fmt --all --check` passes cleanly.
- [x] 1.2 Re-run `cargo test --workspace` and the required smoke entrypoints to confirm the documented release checklist is green from a clean workspace.
- [x] 1.3 Re-run the benchmark dry-run command and confirm the release checklist still distinguishes mandatory dry-run from conditional baseline refresh.

## 2. Synchronize release-facing repository artifacts

- [x] 2.1 Update `ROADMAP.md` and `README.md` so they describe the same M1 closeout state and point to the next milestone focus without implying deferred Unix metadata work is already done.
- [x] 2.2 Decide the next release target version and synchronize `[workspace.package].version` plus `CHANGELOG.md` to describe that release consistently.
- [x] 2.3 Update `RELEASING.md` so the authoritative verification checklist, conditional benchmark refresh rules, and milestone-closeout expectations match the current repository behavior.

## 3. Prepare the repository for release review

- [x] 3.1 Review the working tree after the doc and version updates to ensure the repository is cleanly scoped to release-prep and milestone-closeout work.
- [x] 3.2 Summarize the release candidate in repository-traceable form, including the completed verification checklist, compatibility framing, and the remaining post-M1 gaps that stay deferred to later milestones.
