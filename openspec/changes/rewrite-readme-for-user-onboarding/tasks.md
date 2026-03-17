## 1. Reframe the top-level README for users first

- [x] 1.1 Rewrite the opening of `README.md` so it leads with SFA's target use case, value proposition, and non-drop-in relationship to `tar`
- [x] 1.2 Reorder the top-level sections so installation, quick start, benchmark evidence, and scope framing appear before verification, repository layout, and contribution details

## 2. Make installation and quick start runnable

- [x] 2.1 Replace the current release-archive wording with installation guidance that matches the repository's actual release state, supported target matrix, and checksum or source-build path
- [x] 2.2 Update Quick Start examples to use the installed CLI path and cover pack, unpack, stdin unpack, and machine-readable stats where supported
- [x] 2.3 Verify the documented commands against the current CLI help and release asset naming so the README does not rely on aspirational or stale examples

## 3. Surface benchmark evidence and supporting links

- [x] 3.1 Add a short benchmark snapshot with concrete `tar + same codec` comparison numbers taken from the committed baseline
- [x] 3.2 Link the README benchmark claims to the committed methodology and baseline assets so readers can audit the scope and environment of the comparison
- [x] 3.3 Review the final README for consistency with release and benchmark documentation, including current pre-release or release-ready wording
