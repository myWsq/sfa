# unpack-setup-performance Specification

## Purpose
TBD - created by archiving change parallelize-unpack-directory-setup. Update Purpose after archive.
## Requirements
### Requirement: Unpack setup reduces directory materialization serialization before worker execution
The unpack implementation SHALL build a directory-setup plan before `run_unpack_pipeline()` begins and SHALL materialize required directories through a bounded setup strategy that can use multiple workers when independent directory targets are available. It MUST preserve parent-before-child creation order, current overwrite semantics, and dirfd-style safety checks while ensuring every directory required for regular-file restore exists before the decode/scatter pipeline starts.

#### Scenario: Independent directory frontiers use bounded setup parallelism
- **WHEN** a caller unpacks a valid archive containing many directories across multiple independent subtrees and provides an unpack thread count greater than one
- **THEN** the implementation may materialize same-depth directory targets concurrently before starting the pipeline, while creating parent directories before their children and preserving the existing safe restore behavior

#### Scenario: Narrow directory trees still restore correctly
- **WHEN** a caller unpacks a valid archive whose directories form only a narrow chain or otherwise provide little setup parallelism
- **THEN** the implementation still completes safe directory materialization before the pipeline starts and does not require setup parallelism to preserve correctness

### Requirement: Setup-side prepared directory handles remain reusable by later restore work
When unpack prepares directories before worker execution, it SHALL retain reusable prepared directory handles for those paths and make them available to the later restore path. Scatter and finalize work MUST NOT need to rediscover already prepared parent directories solely because setup execution became less serial.

#### Scenario: Prepared directory cache survives setup optimization
- **WHEN** unpack finishes preparing directories and begins restoring regular-file content
- **THEN** later restore work can consume the prepared directory handles created during setup instead of reopening those parent directories from scratch

