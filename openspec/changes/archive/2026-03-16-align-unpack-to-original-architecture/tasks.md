## 1. Reader Entry And CLI Surface

- [x] 1.1 Add `unpack_reader_to_dir<R: Read>` in `sfa-unixfs` and make the path-based unpack entry delegate to it.
- [x] 1.2 Extend CLI/service handling so `sfa unpack -` reads from stdin, while `stdin + --dry-run` fails with a usage error.

## 2. Pipeline Realignment

- [x] 2.1 Split the current unpack worker path into bounded reader, decode, and scatter stages without changing the existing unpack stats schema.
- [x] 2.2 Preserve frame-hash rejection, strong trailer verification, and thread-count observability after the pipeline split.

## 3. Dirfd-Style Restore

- [x] 3.1 Replace path-string-based restore operations with dirfd/openat-style safe IO for directory creation, regular-file open/create, symlink creation, and hardlink creation.
- [x] 3.2 Keep bounded lazy regular-file handle caching and apply `mode/mtime/uid/gid` finalize semantics for regular files and directories according to policy.
- [x] 3.3 Add `.sfa-untrusted` marker handling for strong trailer failures and clear stale markers on successful unpack starts.

## 4. Verification And Documentation

- [x] 4.1 Add regression coverage for fragmented reader unpack, CLI stdin unpack, dirfd path-escape rejection, strong trailer marker behavior, and the existing multi-bundle probe guarantees.
- [x] 4.2 Update unpack-facing docs and benchmark notes to describe stdin support, dirfd/openat restore, and `.sfa-untrusted`.
- [x] 4.3 Re-run `cargo test --workspace`, the existing smoke scripts, benchmark report refresh, and the real `node_modules` unpack thread sweep to confirm the new pipeline is not worse than the current baseline.
