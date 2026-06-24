### Fixed
- Deploy GitHub Pages from the resolved release commit so the website and API docs advertise the same version as the latest release.
- Retry `rust-script` installation in CI so transient crates.io HTTP failures do not fail unrelated workflow jobs.
