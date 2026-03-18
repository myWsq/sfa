## Context

The repository already contains the ingredients for a strong user-facing README: a benchmark baseline committed under `benches/results/`, documented benchmark methodology, and a release workflow that packages platform-specific archives with checksums. The current top-level README does not expose those assets along a normal user journey. It leads with repository state, mixes user and maintainer concerns, and uses source-tree-relative commands that only help readers who already know how to build the project locally.

This change is documentation-only, but it is cross-cutting because the README must accurately summarize release state, installation paths, quick-start commands, benchmark evidence, and project scope without drifting away from the underlying release and benchmark documents.

## Goals / Non-Goals

**Goals:**
- Make the root README answer the first-visit questions in order: what SFA is for, why it beats `tar` in its target niche, how to install it, and how to use it immediately.
- Align installation text with the actual release asset naming and platform matrix already defined by the repository.
- Surface a small benchmark snapshot that is concrete, reproducible, and explicitly scoped to the committed baseline environment.
- Keep maintainer-oriented sections available, but move them below the main user onboarding flow.

**Non-Goals:**
- Renaming the executable, changing release asset packaging, or introducing a new installer channel
- Reworking benchmark generation, datasets, or measurement methodology
- Rewriting deeper protocol, architecture, or contributing documentation outside the top-level README

## Decisions

### Decision: Organize the README around the user adoption path

The README will be restructured so the first screen flows from value proposition to installation to quick start. This is the shortest path from landing on the repository to running the tool successfully.

Alternative considered: keep the current repository-status-first structure and only patch missing sections.
Why not: the existing structure is maintainer-centric and would continue to bury the adoption path under roadmap, verification, and repo-layout material.

### Decision: Use benchmark claims only when backed by the committed baseline and scope them tightly

The README will surface a small set of comparison numbers drawn from the committed `tar + same codec` baseline and will explicitly point readers to the methodology and environment details. This keeps the headline persuasive without making unbounded performance claims.

Alternative considered: avoid concrete numbers and only state that SFA is benchmarked against `tar`.
Why not: the repository already has strong evidence; omitting it wastes the most user-visible differentiator.

### Decision: Make installation guidance state-aware rather than aspirational

The README installation section will describe the real active path for the repository state it documents. When release archives are available, it should show the actual archive naming and checksum flow. When the repository is still pre-release, it should say so plainly and keep source build as the active path instead of implying published binaries exist already.

Alternative considered: continue using future-tense wording such as "intended to publish".
Why not: it forces users to guess whether binaries are actually available and breaks trust.

### Decision: Keep source-build and release-download users on separate tracks

The README will distinguish "install from release assets" from "build from source" and will ensure Quick Start uses the installed CLI path rather than `./target/release/...`. Source-build details remain available as a fallback path, not the default mental model.

Alternative considered: use only source-build examples everywhere.
Why not: that makes the release workflow effectively invisible to end users and turns Quick Start into a maintainer workflow.

## Risks / Trade-offs

- [Benchmark snapshot ages as the baseline changes] -> Point the README at the committed baseline and refresh the snapshot whenever benchmark-affecting releases refresh that asset.
- [Release-state wording can drift from reality] -> Tie installation text to the current repository state and keep it consistent with release docs and workflow-defined asset names.
- [Executable naming remains slightly awkward because the shipped binary is `sfa-cli`] -> Document the real binary name consistently for now; treat any rename to `sfa` as a separate product decision.
- [A user-focused README gives less space to maintainer details above the fold] -> Preserve those sections lower in the document and link to deeper docs rather than removing them.

## Migration Plan

1. Rewrite the top-level README sections in the new order.
2. Validate installation wording against the documented release assets and source-build fallback.
3. Verify every Quick Start command against the current CLI surface.
4. Cross-check benchmark claims against the committed baseline and methodology docs.

## Open Questions

- Should the public-facing docs continue to present the executable as `sfa-cli`, or should a later change rename the shipped binary to `sfa`?
- If the repository remains pre-release for some time, should the README include templated future release commands, or only the currently valid source-build path?
