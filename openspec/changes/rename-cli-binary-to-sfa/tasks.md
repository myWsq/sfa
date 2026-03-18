## 1. Docs and Spec Alignment

- [x] 1.1 Update the OpenSpec artifacts for the `sfa` managed-install rename and README onboarding changes.
- [x] 1.2 Update `README.md` and `RELEASING.md` so installation, quick start, and release instructions use `sfa`.

## 2. Public Binary and Distribution Plumbing

- [x] 2.1 Rename the released CLI binary target to `sfa` and update CLI integration tests to execute the renamed binary.
- [x] 2.2 Update the install script, release workflow, and Homebrew formula generation/publication to package and install `sfa` via `Formula/sfa.rb`.

## 3. Verification Tooling

- [x] 3.1 Update distribution smoke tests and benchmark binary discovery so `sfa` is the only supported installed command name.
- [x] 3.2 Run focused verification for the renamed CLI binary and managed-distribution flow.
