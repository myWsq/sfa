## 1. Close the current v1-scope milestone state

- [x] 1.1 Update `ROADMAP.md` and `README.md` so the current M3 metadata-contract hardening slice is treated as complete for the stable v1 scope and post-v1 Unix extensions remain explicitly deferred.
- [x] 1.2 Update the relevant `sfa-tech-solution/` roadmap material and any directly related repository status wording so the stable v1 boundary, deferred scope, and next milestone or release-train framing are consistent.
- [x] 1.3 Apply the `release-readiness` spec delta to the repository's release-facing expectations so M3 closeout and first-stable-release preparation are reviewable from the repo alone.

## 2. Synchronize first-stable-release materials

- [x] 2.1 Set the first stable release target to `1.0.0` across workspace version metadata, `CHANGELOG.md`, and the in-repo release notes file `release-notes/v1.0.0.md`.
- [x] 2.2 Update `RELEASING.md` and related release guidance so they describe the selected candidate revision, the included post-`v0.3.0` unpack setup optimization, and the resulting benchmark-baseline refresh expectation.
- [x] 2.3 Ensure roadmap, changelog, and release notes all distinguish shipped `v1.0.0` scope from deferred post-v1 follow-up work such as xattrs, ACLs, and broader Unix extensions.

## 3. Restore release-gate compliance on the candidate revision

- [x] 3.1 Fix the current candidate-revision release blockers, including the formatting drift in `crates/sfa-unixfs/src/restore.rs`, without expanding runtime scope beyond stable-release preparation.
- [x] 3.2 Run the authoritative release checklist on the selected candidate revision and confirm `cargo fmt --all --check`, `cargo test --workspace`, the smoke scripts, and the benchmark dry run all pass cleanly.
- [x] 3.3 Refresh `benches/results/baseline-v0.1.0.json` and validate it with `cargo test -p sfa-bench` because the selected candidate includes benchmark-affecting unpack setup changes beyond `v0.3.0`.

## 4. Review and hand off the release candidate

- [x] 4.1 Review the final diff and `git status --short` to confirm the change is cleanly scoped to M3 closeout and first-stable-release preparation.
- [x] 4.2 Verify that the release workflow inputs for `v1.0.0` are present and consistent, then record the exact tagging and publishing commands for the stable release handoff.
