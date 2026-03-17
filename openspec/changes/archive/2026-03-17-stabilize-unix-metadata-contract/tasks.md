## 1. Align pack and unpack behavior with the clarified metadata contract

- [x] 1.1 Audit the current pack/unpack owner and metadata paths against the new `archive-pack` and `archive-unpack` spec deltas, then adjust any mismatches in `sfa-core`, `sfa-unixfs`, and `sfa-cli`.
- [x] 1.2 Clarify CLI-facing and API-facing owner-policy behavior so the implemented default path, explicit preserve-owner path, and deferred metadata scope are documented consistently in code-adjacent surfaces.

## 2. Expand repository verification for Unix metadata behavior

- [x] 2.1 Add or extend automated tests that verify supported restores preserve `mode` and `mtime` for regular files and directories under the default non-privileged path.
- [x] 2.2 Add or extend verification coverage for owner-policy behavior so the default / explicit no-owner paths remain non-restoring and the privileged preserve-owner branch stays repository-traceable without becoming a mandatory root-only checklist item.
- [x] 2.3 Update roundtrip, integration, or smoke assets so metadata-focused restore cases are auditable alongside the existing link and safety coverage.

## 3. Synchronize repository-facing milestone and scope documentation

- [x] 3.1 Update `README.md`, `ROADMAP.md`, and the relevant `sfa-tech-solution/` documents so M3 reflects the narrowed metadata-contract scope and keeps xattrs / ACL explicitly deferred.
- [x] 3.2 Run the relevant workspace tests and smoke checks for the metadata-contract change set, then review the final diff to confirm the change is limited to contract, verification, and documentation work.
