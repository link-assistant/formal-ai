### Fixed

- Split isolated native coding, reasoning, Hive Mind, benchmark and interface contracts into independent jobs after serial execution exhausted the 20-minute job budget. Preserve the full target union, single Rust test threads, failure propagation, read-only permissions and existing job limits. Keep the default complete native runner available.
