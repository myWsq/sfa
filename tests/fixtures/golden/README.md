# Golden Fixtures

This directory stores canonical protocol fixtures for the frozen SFA v1 format.

Each fixture lives under its own subdirectory:

```text
tests/fixtures/golden/<fixture-name>/
├── input/
├── archive.sfa
├── manifest.json
├── stats.json
└── README.md
```

Required conventions:

- `input/` is the stable source tree used to generate the fixture.
- `archive.sfa` is the committed canonical archive artifact.
- `manifest.json` is the decoded archive dump produced by `dump_archive_fixture`.
- `stats.json` is the committed summary snapshot paired with `manifest.json`.
- `README.md` records the fixed generation parameters and what protocol behavior the fixture is intended to freeze.

Use `tests/scripts/generate_golden_fixture.sh` to regenerate fixture outputs from a committed `input/` tree. A fixture update that changes `archive.sfa`, `manifest.json`, or `stats.json` is a protocol-significant change and should be reviewed against `spec/format-v1.md` and `spec/format-v1-freeze-review.md`.
