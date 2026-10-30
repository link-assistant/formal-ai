### Fixed

- Reuse CI's prepared browser assets instead of rebuilding them inside
  Playwright's server-start wait. Keep standalone preparation, bound startup
  to two minutes, and retain both server log streams for diagnosing failures.
