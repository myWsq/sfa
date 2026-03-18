## Why

SFA already has a strong small-file archive story, committed `tar + same codec` benchmark evidence, and a release workflow that packages prebuilt binaries, but the top-level README still reads like a repository status note rather than a user entry point. A new user cannot quickly answer the three adoption questions that matter most: why SFA is worth trying, how to download the right binary, and what to run first after download.

## What Changes

- Rewrite the top-level README so it leads with SFA's target use case, a benchmark-backed value proposition, and a short "when to use / when not to use" framing for Unix directory trees with many small files.
- Replace the current future-tense release-archive wording with actionable installation guidance that matches the repository's actual release assets, platform matrix, checksum flow, and source-build fallback.
- Rework Quick Start around the installed CLI experience so release-download users can pack, unpack, and inspect stats without first reading build-system details.
- Add a benchmark snapshot section that surfaces a small set of concrete `tar` comparison numbers and links users to the reproducible methodology and committed baseline for context.
- Push repository-internal material such as verification checklists, layout notes, and contribution guidance below the user onboarding path instead of letting them dominate the first screen.

## Capabilities

### New Capabilities

- `readme-user-onboarding`: defines the minimum user-facing contract for the repository README, including positioning, installation guidance, quick-start flow, benchmark evidence, and accurate release-state framing.

### Modified Capabilities

None.

## Impact

- The top-level `README.md` structure and messaging
- User-facing installation and usage guidance derived from the existing GitHub release asset naming and platform matrix
- Benchmark-facing documentation references that connect the README snapshot to the committed baseline and methodology
- OpenSpec coverage for repository-level user onboarding expectations
