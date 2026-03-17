# SFA Release Process

This document defines the current release process used by the SFA repository. Git tags remain the source of truth for published versions, while GitHub Releases are normally created and updated by the [release workflow](.github/workflows/release.yml) after a version tag is pushed. Manual GitHub Release creation is only a fallback path.

## Scope

The current process covers:

- Repository-level version releases
- Git tags and GitHub Releases
- Pre-release validation for protocol behavior, test assets, and benchmark baselines

The current process does not cover:

- crates.io publishing
- Multi-platform installer distribution
- macOS notarization or code signing

## Release Principles

- Every public release must have a traceable Git tag
- Release contents must match the repository state, roadmap, and protocol documentation
- Protocol-sensitive changes must update code, specs, fixtures, and verification assets together
- The working tree must be clean before a release
- Every release must pass the repository-defined quality gates

## Release Preconditions

Do not start the release process until all of the following are true:

1. The relevant OpenSpec change is complete, or the release content is otherwise clearly resolved in the repository.
2. [ROADMAP.md](ROADMAP.md) and [README.md](README.md) reflect the current milestone and top-level project status.
3. If the release changes protocol behavior or decode semantics:
   - `spec/format-v1.md` has been updated
   - Golden fixtures have been updated and still cover a representative canonical corpus
   - Compatibility impact is described in the release notes
4. The release content has completed code review.
5. `git status --short` is empty.

## Standard Release Procedure

### 1. Confirm the Version Scope

Start by confirming that the working tree is clean:

```bash
git status --short
```

If the output is not empty, stop and clean up the tree before continuing.

Classify the release as one of:

- `patch`: defect fixes without changing expected interface or protocol behavior
- `minor`: new capability with backward compatibility preserved
- `major`: an incompatible behavior or compatibility change

If `format-v1` is not frozen for the relevant release line, the release notes must explicitly state that protocol compatibility remains in flux even if the version number follows SemVer progression.

### 2. Update Versioned Materials

At minimum, keep the following in sync:

- `[workspace.package].version` in [Cargo.toml](Cargo.toml)
- [CHANGELOG.md](CHANGELOG.md)
- Relevant milestone state in [ROADMAP.md](ROADMAP.md)
- Top-level status wording in [README.md](README.md)
- The in-repo release notes file, usually `release-notes/vX.Y.Z.md`

If the release affects protocol behavior or verification assets, also update:

- [spec/format-v1.md](spec/format-v1.md)
- [spec/verification-and-benchmark.md](spec/verification-and-benchmark.md)
- The affected fixtures under `tests/fixtures/`
- [tests/golden/README.md](tests/golden/README.md) and any relevant fixture README coverage notes

### 3. Run the Quality Gates

Before releasing, run at least:

```bash
cargo fmt --all --check
cargo test --workspace
bash tests/scripts/run_protocol_smoke.sh
bash tests/scripts/run_streaming_smoke.sh
bash tests/scripts/run_safety_smoke.sh
bash tests/scripts/run_roundtrip_smoke.sh
cargo run -p sfa-bench --bin tar_vs_sfa -- --dry-run --output benches/results/latest.json
```

These commands form the repository's authoritative release checklist.

If the release changes benchmark logic, default datasets, planner or pipeline behavior, codec integration, or the benchmark support environment, refresh the committed benchmark baseline as well and describe that refresh in the release notes:

```bash
CARGO_HOME=/tmp/cargo-home cargo build --release -p sfa-cli
./benches/scripts/run_tar_vs_sfa.sh \
  --execute \
  --sfa-bin target/release/sfa-cli \
  --output benches/results/baseline-v0.1.0.json
```

After refreshing the baseline, confirm that `benches/results/baseline-v0.1.0.json` is committed and that `cargo test -p sfa-bench` still validates the asset.

If the release does not change benchmark behavior, datasets, or result schema, the benchmark dry run remains mandatory but a fresh committed baseline is not required.

If benchmark evidence is part of the release claim, also confirm that the committed baseline includes:

- `environment.resource_sampler` aligned with the documented support environment
- Command wall-time for each execution record
- Pack phase-level `sfa_stats` for SFA runs, plus unpack additive `wall_breakdown` and diagnostic `phase_breakdown`
- `user_cpu_ms`, `system_cpu_ms`, and `max_rss_kib` where the support environment provides them
- Unpack additive `sfa_stats.wall_breakdown` fields named `setup`, `pipeline`, and `finalize`
- Unpack diagnostic `sfa_stats.phase_breakdown` fields named `header`, `manifest`, `frame_read`, `decode`, `scatter`, and `restore_finalize`, rather than the older `decode_and_scatter`
- A preserved effective thread count when the release affects unpack pipeline behavior or `--threads` semantics

When documenting unpack timings, treat `wall_breakdown` as the additive wall-time accounting view and `phase_breakdown` as the pipelined diagnostic view rather than as values that must all sum directly to total wall-time.

### 4. Prepare Release Notes

Each release notes file should cover at least:

- A short release summary
- Major additions or fixes
- Whether protocol behavior changed
- Verification status
- Known limitations or follow-up work

Recommended structure:

```text
Highlights
Compatibility
Verification
Known Gaps
```

Prepare `release-notes/vX.Y.Z.md` in the repository first, verify that the summary and validation statements match the repository state, and then publish that file as the GitHub Release body.

### 5. Create and Push the Tag

Once the main branch content is confirmed, create the version tag:

```bash
git tag -a vX.Y.Z -m "sfa vX.Y.Z"
git push origin vX.Y.Z
```

If the release also requires pushing the branch tip:

```bash
git push origin main
git push origin vX.Y.Z
```

After `vX.Y.Z` is pushed, the release workflow will:

- Re-run the authoritative release checklist on the tagged revision
- Read `release-notes/vX.Y.Z.md`
- Build CLI archives for Linux `x86_64`, macOS `x86_64`, and macOS `arm64`
- Create or update the GitHub Release and attach the generated archives and checksum files

If the release workflow is not yet available, or if a Release must be backfilled for an existing tag, manually trigger `.github/workflows/release.yml` with `workflow_dispatch` and pass the tag name.

### 6. Confirm the GitHub Release

Under normal conditions, the tag-triggered release workflow will publish the GitHub Release for `vX.Y.Z` using the contents of `release-notes/vX.Y.Z.md`.

The same workflow also uploads:

- `sfa-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- `sfa-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- `sfa-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- Matching `.sha256` checksum files

If the workflow cannot be used, fall back to:

```bash
gh release create vX.Y.Z --verify-tag --title "sfa vX.Y.Z" --notes-file release-notes/vX.Y.Z.md
```

The public release should make clear:

- Whether the protocol is frozen
- Recommended use cases
- Compatibility changes relative to the previous version
- The roadmap stage represented by the release

### 7. Post-Release Follow-Up

After the release is published, check:

- Whether [ROADMAP.md](ROADMAP.md) and [README.md](README.md) need status updates
- Whether [CHANGELOG.md](CHANGELOG.md) should reopen the next `Unreleased` section
- Whether protocol evolution requires a new OpenSpec change

## Additional Requirements for Protocol-Sensitive Releases

Treat the following as protocol-sensitive changes:

- Any change to header, manifest, frame, or trailer structure
- Any change to codec or integrity field semantics
- Any change to decoder tolerance or validation behavior
- Any change that affects golden fixtures

These releases must additionally satisfy:

- `spec/format-v1.md` matches the implementation
- Golden fixtures, dump outputs, and fixture README coverage notes are updated
- The release notes explicitly describe compatibility impact

## Minimal Release Checklist

Before publishing, confirm:

- [ ] Version metadata has been updated
- [ ] [CHANGELOG.md](CHANGELOG.md) has been updated
- [ ] The release notes file matches the version and repository state
- [ ] `git status --short` is empty
- [ ] The authoritative release checklist has passed
- [ ] Specs and fixtures are synchronized when the release affects protocol behavior
- [ ] The Git tag has been created and pushed
- [ ] The GitHub Release has been created by the workflow or by manual fallback
- [ ] Linux and macOS release assets have been uploaded
