## Context

SFA already has a tagged-release workflow that verifies the repository, builds platform archives, attaches those archives to GitHub Releases, and emits matching `.sha256` files. What it does not have is a managed installation story on top of those assets. Users still need to select the correct archive manually, verify it themselves, unpack it, and place the binary on `PATH`.

This change is cross-cutting because it touches release automation, public install assets, release documentation, and the repository's user-onboarding contract. It also introduces one external system boundary: a project-owned Homebrew tap repository. For the current canonical remote, that tap should live under the same GitHub owner as the main repository, for example `myWsq/homebrew-sfa`.

The current platform matrix is limited to Linux `x86_64`, macOS `x86_64`, and macOS `arm64`. Windows package managers, crates.io, and macOS signing or notarization remain explicitly out of scope for this change.

## Goals / Non-Goals

**Goals:**
- Make released SFA binaries installable through a one-command managed path on the current macOS and Linux target matrix.
- Keep GitHub Release archives and checksums as the only binary source of truth.
- Add a public install script that can fetch, verify, and install the correct release asset for a supported host.
- Add a project-owned Homebrew tap that tracks released versions using the same archive URLs and checksums.
- Update README and release documentation so managed channels are the default recommendation and fallback paths stay accurate.

**Non-Goals:**
- Adding Windows distribution through WinGet, Scoop, or MSI packages
- Publishing to crates.io or requiring `cargo install` as a supported default path
- Renaming the shipped binary from `sfa-cli` to `sfa`
- Adding macOS Developer ID signing, notarization, or other installer packaging such as `.pkg` or `.dmg`
- Creating native `apt`, `rpm`, or Nix packaging in this change

## Decisions

### Decision: GitHub Release assets remain the canonical distribution source

The release workflow will continue to build the platform archives and `.sha256` files that are attached to each GitHub Release. The install script and the Homebrew formula will derive their URLs and checksums from those published assets instead of rebuilding binaries or maintaining separate package payloads.

Alternative considered: build separate binaries for the install script or for Homebrew distribution.
Why not: it creates multiple binary supply chains, increases drift risk, and makes release verification harder because users on different channels would not necessarily receive the same artifact.

### Decision: Support two managed install channels now, not a broader package matrix

This change will add exactly two first-class installation channels above raw release archives: a public shell install script and a Homebrew tap. Together they cover the current supported macOS and Linux targets with relatively low maintenance burden.

Alternative considered: add `apt`, `rpm`, `winget`, `crates.io`, or installer bundles in the same change.
Why not: each adds a distinct publishing surface, operational process, and support burden. Homebrew plus an install script captures most of the immediate user-friction reduction without expanding the release matrix beyond what the repository already builds.

### Decision: The install script is versioned, asset-driven, and intentionally small

The repository will publish a shell install script that supports installing either the latest release or an explicit version. The script will:
- detect the host OS and architecture
- map that host to the existing release asset naming convention
- download the matching archive and checksum file from GitHub Releases
- verify the archive with an available SHA-256 tool
- unpack `sfa-cli` and install it into a configurable destination directory
- fail with actionable errors for unsupported targets, missing host tools, or checksum mismatches

The script should be written to run under a plain POSIX-style shell and depend only on common host utilities such as `uname`, `tar`, `mktemp`, `curl` or `wget`, and one SHA-256 verification tool.

Alternative considered: require users to run a Rust bootstrap command or a Bash-only installer.
Why not: Rust bootstrapping defeats the purpose of binary distribution, and Bash-specific scripts reduce portability on minimal environments.

### Decision: Homebrew publication is automated through a dedicated tap repository

The release workflow will generate a Homebrew formula from the just-published release metadata and push it to a project-owned tap repository such as `myWsq/homebrew-sfa`. The formula will reference the release archives and checksums for the supported target matrix and will install the existing `sfa-cli` binary without renaming it.

Publishing to a separate tap repository requires a dedicated credential because the default GitHub Actions token for `myWsq/sfa` cannot assume cross-repository write access. The workflow should therefore use a repository secret dedicated to tap publication. If the tap update fails, the release workflow should fail rather than silently claiming that the managed Homebrew channel is current.

Alternative considered: keep a formula file only inside the main repository or update the tap repository manually.
Why not: formula-in-repo still leaves users on a raw-URL workflow instead of a normal `brew install` path, while manual tap updates create an obvious drift vector between the release assets and the advertised installation channel.

### Decision: Managed-channel documentation becomes the default user path, but archives stay visible as fallback

`README.md` will recommend `brew install` and the install script first when a release is available. Direct archive download and build-from-source remain documented fallback paths for unsupported environments, security-sensitive users who want manual artifact handling, and contributors working from source.

Alternative considered: replace archive and source-build instructions entirely.
Why not: fallback paths are still necessary for unsupported hosts, for debugging, and for users who do not want to execute a network install script.

## Risks / Trade-offs

- [A release can succeed partially if the tap repository update breaks after GitHub assets are uploaded] -> Make the workflow fail loudly on tap publication errors, document a manual tap-update fallback, and allow rerunning the release workflow for the same tag.
- [The supported matrix remains narrow because current binaries cover only Linux `x86_64` and macOS `x86_64` / `arm64`] -> Keep unsupported-host messaging explicit in the install script and docs rather than implying broader support.
- [Unsigned macOS binaries can still trigger trust friction even when installation is easier] -> Keep the installer and README explicit that signing and notarization are not yet part of the supported distribution contract.
- [The Homebrew package name may feel slightly awkward while the installed executable remains `sfa-cli`] -> Keep formula and docs aligned with the actual binary name in this change, and treat any command rename as a separate product decision.
- [Installer scripts are easy to let drift from asset naming changes] -> Centralize asset-name construction and release-version handling, and add release-time validation that the script and formula both resolve the exact uploaded asset names.

## Migration Plan

1. Add the install script and any supporting metadata or helper logic needed to map release tags to platform archives.
2. Extend the release workflow so it collects release asset checksums and generates the Homebrew formula for the same version.
3. Create and seed the dedicated Homebrew tap repository, then wire the release workflow to push formula updates using a dedicated credential.
4. Update README and release documentation to recommend `brew install` and the install script ahead of manual archive download.
5. Add release-time verification that the generated formula and install script inputs match the actual uploaded GitHub Release assets.
6. Roll back by removing managed-channel docs, disabling tap publication in the workflow, and leaving GitHub Release archives as the only published path if the new channels prove unreliable.

## Open Questions

- Should the install documentation prefer `curl | sh` examples, or should it always show a two-step download-then-run flow even if the script itself supports both?
- Should the tap publication workflow update only stable tags such as `v1.2.3`, or should it also have an explicit policy for pre-release tags once the repository starts publishing them?
