# Fixtures

Fixtures are grouped by intent:

- `datasets/`: benchmark input datasets
- `golden/`: protocol golden artifacts and decoded snapshots
- `corruption/`: malformed archive blobs
- `streaming/`: chunk pattern definitions
- `safety/`: path escape and unsafe node examples

The committed benchmark datasets under `datasets/` are intended to be runnable from a clean checkout without any external download step. Each dataset directory includes a short README with its purpose, construction notes, and stable size summary so benchmark baselines remain reviewable.
