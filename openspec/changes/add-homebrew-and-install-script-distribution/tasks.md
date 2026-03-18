## 1. Add managed installer assets

- [ ] 1.1 Add a public install script that maps supported macOS and Linux hosts to the existing GitHub Release asset names, downloads the selected release archive and checksum, verifies the archive, and installs `sfa-cli` into a configurable destination directory.
- [ ] 1.2 Add any supporting metadata or helper logic needed to keep release asset naming, version resolution, and checksum handling shared between installer-facing outputs instead of duplicating that logic in multiple places.
- [ ] 1.3 Add verifiable coverage for the installer path, including supported-host resolution, explicit-version installation, unsupported-host failure, and checksum-mismatch handling.

## 2. Automate Homebrew publication from release assets

- [ ] 2.1 Add Homebrew formula generation inputs and rendering logic that produce a formula for the released version using the canonical GitHub Release archive URLs and SHA-256 values for the supported target matrix.
- [ ] 2.2 Extend the release workflow so tagged releases gather the published asset metadata, generate the Homebrew formula, and publish it to the project-owned tap repository using dedicated release credentials.
- [ ] 2.3 Document or script the Homebrew tap bootstrap and manual fallback process so maintainers can recover if automated tap publication fails for an already-created tag.

## 3. Update release and onboarding documentation

- [ ] 3.1 Update `README.md` so published releases recommend `brew install` and the install script first, while keeping direct archive download and source build as explicit fallback paths.
- [ ] 3.2 Update `RELEASING.md` and any release-note guidance so managed distribution publication, required secrets, and post-release verification steps are part of the documented release process.
- [ ] 3.3 Add a release-time validation or smoke-check step that confirms the generated installer and Homebrew metadata resolve the exact uploaded release asset names and checksums.
