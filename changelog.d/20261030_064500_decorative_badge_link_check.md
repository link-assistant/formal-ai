### Fixed

- Prevent the exact decorative crates.io version badge's transient HTTP 408
  from blocking CI, while continuing to check its crate destination and other
  links. Run real-HTTP timeout and missing-page regressions in the link workflow.
