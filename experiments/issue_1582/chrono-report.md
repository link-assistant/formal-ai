The published crate currently introduces `chrono` as a required dependency. RustSec advisory [RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543) marks every chrono version unmaintained (`patched = []`) and recommends `jiff` or `time`.

This surfaced in downstream Formal AI's [Cargo audit run](https://github.com/link-assistant/formal-ai/actions/runs/44096923794), lines 741–748, while repairing https://github.com/link-assistant/formal-ai/issues/1582. The downstream lockfile contains `command-stream 1.5.3 -> chrono 0.4.45` and `link-calculator 0.22.0 -> chrono 0.4.45`. The latest published versions, command-stream 1.13.0 and link-calculator 0.24.0, still declare chrono as a nonoptional normal dependency, so a downstream upgrade cannot remove it.

Reproduction without compiling:

1. Create a minimal Cargo package with this crate as a dependency (use the latest published version).
2. Run `cargo generate-lockfile`.
3. Run `cargo audit --file Cargo.lock --deny warnings` with cargo-audit 0.22.2 and a current advisory database.
4. Observe RUSTSEC-2026-0543 and a nonzero audit status. `cargo tree --invert chrono` identifies the introducing dependency.

Suggested code change: replace chrono usage with maintained `jiff` or `time` equivalents, preserve timestamp precision/UTC formatting and date arithmetic semantics, and publish a release without chrono in its dependency graph. Add focused tests for formatted timestamps and boundary cases; for the calculator, preserve timezone, parsing and date-difference behavior. Audit the resulting lockfile to confirm chrono is gone.

Temporary downstream workaround: narrowly acknowledge this informational unmaintained advisory with its exact ID and a recorded upstream report. This does not fix the maintenance issue. Keep other vulnerabilities, unsoundness and yanked findings fatal, and automatically reject a stale exception once chrono leaves the build graph. Disabling an unrelated feature cannot remove an unconditional dependency.
