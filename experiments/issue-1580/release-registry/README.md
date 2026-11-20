# Release registry regressions

This small Cargo harness compiles the actual release scripts and the shared
registry module without compiling Formal AI. The CLI fixture adapts the pinned
Rust template regression (309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb).
It copies script sources and changes only three HTTPS origins to one loopback
server and the retry delay to zero. Production URLs remain fixed.

Hosted native CI runs the real HTTP/module tests and the two CLI binaries against
503, 429, 403, malformed-success, dropped-connection and 404 fixtures. The writer
uses a temporary Git repository and local bare origin. Failures must preserve the
manifest and clean tree and must not emit release authorization. Successful
retries and a higher published version floor remain covered.

No local Rust compilation is needed. CI uses the committed Cargo.lock and reuses
one target directory for the unit tests and the copied CLI fixture.
