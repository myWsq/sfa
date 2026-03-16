# small-binary

Purpose: provide a deterministic small-binary corpus with multiple payload sizes for `tar + same codec` and SFA pack/unpack comparison.

Construction:

- generated from deterministic byte patterns with fixed seeds per file
- includes raw frame fragments, cache pages, payload blobs, and firmware-like images

Stable summary:

- 11 files under `input/`
- 344,064 total input bytes
- nested layout covering `assets/raw`, `cache`, `payloads`, and `firmware`
