### Fixed

- Keep crate availability uncertain when a registry endpoint returns a malformed success response and its peer reports the version absent.
- Require valid SemVer in every sparse index record and reject API responses that identify a different crate.
