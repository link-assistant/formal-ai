### Fixed

- Start the full Docker image with a private memory directory writable by its `box` runtime user. Docker Compose and Desktop mount shared memory at that path.
- Assign Docker release tags only after the full and slim images pass their runtime checks.
