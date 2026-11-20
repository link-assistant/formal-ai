### Fixed

- Fail the Rust dependency audit when Git cannot complete the committed lockfile
  inventory, including failures after partial output. Preserve strict primary
  and experiment-lock audit policies and reject missing sparse-checkout inputs.
