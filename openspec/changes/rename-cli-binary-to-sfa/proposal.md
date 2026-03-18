## Why

Managed distribution still exposes `sfa-cli` even though the documented command
surface is already `sfa`. That leaves Homebrew install commands awkward, makes
release artifacts inconsistent with the CLI help text, and forces users to
remember two names for the same tool.

## What Changes

- **BREAKING** Rename the shipped executable in release archives, the public
  install script, and the Homebrew formula from `sfa-cli` to `sfa`, with no
  managed-channel compatibility alias.
- Publish the Homebrew formula as `sfa` and update release automation to write
  `Formula/sfa.rb` instead of `Formula/sfa-cli.rb`.
- Update onboarding docs, verification docs, tests, and benchmark/tooling
  defaults so `sfa` is the only documented installed command name.

## Capabilities

### New Capabilities
- `cli-distribution`: Managed installation channels publish and install the
  `sfa` executable under the `sfa` Homebrew formula name.

### Modified Capabilities
- `readme-user-onboarding`: Installation and quick-start guidance use the
  `sfa` executable and the tap flow that matches the renamed Homebrew formula.

## Impact

- `crates/sfa-cli` binary target and CLI integration tests
- Release packaging workflow and managed-distribution scripts
- `install.sh`, Homebrew formula generation, and tap publication
- `README.md`, `RELEASING.md`, and distribution verification scripts
- Benchmark binary discovery and related user-facing error messages
