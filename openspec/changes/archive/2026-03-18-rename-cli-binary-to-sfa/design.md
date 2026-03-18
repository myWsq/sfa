## Context

The repository already presents the command surface as `sfa` in Clap help text
and most user-facing examples, but the managed distribution path still ships a
binary named `sfa-cli`. Release archives, the install script, Homebrew formula
generation, smoke tests, and some benchmark fallback logic still assume the old
name. This is a cross-cutting change because the public install contract spans
Cargo build output, release packaging, tap publication, documentation, and test
fixtures.

## Goals / Non-Goals

**Goals:**
- Make `sfa` the only supported installed binary name for managed distribution.
- Rename the Homebrew formula to `sfa` so tapped users can run `brew install sfa`.
- Keep release automation, docs, and verification aligned with the renamed
  binary and formula.

**Non-Goals:**
- Publishing the formula to `homebrew/core` so a fresh machine can install with
  `brew install sfa` without tapping the project repository first.
- Renaming the internal workspace path or Cargo package id `sfa-cli` in this
  slice.
- Changing archive file names, release tags, or the tap repository slug.

## Decisions

### Decision: Rename the public binary target to `sfa` without renaming the package id

The distributed executable should become `sfa`, but the Cargo package id and
workspace path can remain `sfa-cli` for now. This keeps internal workspace
references stable while producing the correct public binary name via an
explicit bin target.

Alternative considered: rename the package id and workspace path as well.
Why not: it broadens the change into repository-wide crate churn without
improving the user-facing install experience that motivated this work.

### Decision: Managed distribution drops `sfa-cli` fallback entirely

Installer extraction, Homebrew formula generation, release packaging, smoke
tests, and benchmark binary discovery should all treat `sfa` as the only
supported installed command. This matches the requested breaking change and
avoids carrying two names forward in docs and tooling.

Alternative considered: ship both `sfa` and `sfa-cli` temporarily.
Why not: compatibility aliases keep the naming ambiguity alive and would force
the release docs and tests to support two public commands.

### Decision: README and release docs must distinguish formula renaming from tap distribution

The repository can rename the formula to `sfa`, but a third-party tap still
requires either a prior `brew tap myWsq/sfa` or a fully qualified install
command on a fresh machine. The docs should therefore promote the shorter tapped
flow while staying accurate about the project-owned tap boundary.

Alternative considered: document `brew install sfa` as universally sufficient.
Why not: that would be incorrect unless the formula is also accepted into
`homebrew/core`, which is out of scope for this change.

## Risks / Trade-offs

- [Existing user scripts still call `sfa-cli`] -> Treat the rename as a
  breaking change, update release notes/docs, and avoid silently claiming
  compatibility that no longer exists.
- [Public binary name diverges from Cargo package id] -> Keep the package id
  internal-only and document the maintainer build command separately from the
  installed command.
- [Users may still expect untapped `brew install sfa` to work] -> Make the
  README explicit that the short command assumes the project tap has already
  been added.

## Migration Plan

1. Add an explicit `sfa` bin target to the CLI package and update release
   packaging to copy that binary into the published archives.
2. Switch installer logic, Homebrew formula generation/publication, docs, and
   smoke tests from `sfa-cli` to `sfa`.
3. Update benchmark binary lookup and release guidance so automated tooling no
   longer falls back to `sfa-cli`.

## Open Questions

- None.
