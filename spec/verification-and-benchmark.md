# Verification And Benchmark Baseline

## Verification baseline

- Roundtrip correctness for supported Unix entry types
- Sequential stream decoding with fragmented input
- Corruption handling for header/manifest/frame/trailer
- Path safety enforcement for restore roots

## Benchmark baseline

- Compare SFA and TAR using the same codec (`lz4`, `zstd`)
- Run on `small-text`, `small-binary`, and `large-control` datasets
- Record wall time, files/sec, MiB/sec, output size, and resource metrics

## Runner

`crates/sfa-bench/src/bin/tar_vs_sfa.rs` is the current benchmark runner entrypoint.
