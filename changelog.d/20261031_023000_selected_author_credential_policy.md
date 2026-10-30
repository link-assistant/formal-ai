### Fixed

- Configure the selected author credential explicitly for Git before source
  fetches and writes. Verify that path in the Rust credential policy instead
  of requiring checkout's obsolete default-token header.
