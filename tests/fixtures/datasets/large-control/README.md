# large-control

Purpose: provide a committed large-file control case that crosses the default small-file threshold and remains large enough to observe streaming pack/unpack behavior.

Construction:

- primary payload is a deterministic NDJSON telemetry stream with 24,000 records
- companion metadata file documents the corpus shape and intended codec hint

Stable summary:

- 2 files under `input/`
- 4,740,516 total input bytes
- dominant payload at `input/traces/telemetry.ndjson`
