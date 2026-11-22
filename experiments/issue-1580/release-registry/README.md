# Release registry regressions

This small Cargo harness compiles the actual release scripts and the shared
registry module without compiling Formal AI. The CLI fixture adapts the pinned
Rust template regression (309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb).
It copies script sources and changes only three HTTPS origins to one loopback
server and the retry delay to zero. Production URLs remain fixed.

Hosted native CI runs the real HTTP/module tests and ten CLI binaries. Registry
consumers cover 503, 429, 403, malformed-success, dropped-connection and 404
fixtures. The writer uses a temporary Git repository and local bare origin. Failures must preserve the
manifest and clean tree and must not emit release authorization. Successful
retries and a higher published version floor remain covered.

No local Rust compilation is needed. CI uses the committed Cargo.lock and reuses
one target directory for the unit tests and the copied CLI fixture.

The local command `python3 experiments/issue-1580/release-registry/check_fixture.py`
checks the real loopback server's absent versus wrong-identity responses for
unpublished candidates. It replaces only the setup compilation call and does
not build or run release binaries. The same HTTP fixture regression runs in the
full hosted CLI suite. Its preserved before-control fails with HTTP 404 instead
of the intended mismatched HTTP 200 response.

The version guard also runs against actual temporary Git origins. It checks
package identity after dependency tables, formatting and manifest moves, rejects
failed base fetches and malformed or inherited versions, rejects both increases
and decreases under release-like branch names, and accepts a package version
received through an actual merge of the updated base branch.

The compiled fragment creator also preserves operator text in the physical file
while encoding both Actions command protocols in human logs. The CLI regression
round-trips the logged JSON text and checks the fragment remains byte exact.

The pinned template's output-I/O practice now applies to all five output-writing
counterparts: version reader, bump reader, change detector, publisher and crate
waiter. The real CLI matrix covers a directory, empty path, Linux `/dev/full`,
append behavior, unset local output and Unix non-UTF-8 output paths. Publisher and
waiter use the real template-default skip paths, so this matrix never publishes
or contacts a registry. The maintained smoke test is Bash and has no step-output
protocol; inventing one is not required to apply this practice. The collector's
inventory, fragment-read and successful-consumption cases run as a sixth added
binary and preserve README and unrelated files.

Separate loopback cases require the crate waiter's exact requested identity and
a complete response. Only HTTP 404 can mean absence; malformed success, wrong
identity and exhausted transport/status failures cannot authorize availability.
The change detector preserves case-insensitive `.md` discovery and fails before
any job decision if its relevant main-release inventory cannot be read. Other
workflow contexts do not gain a new fragment-inventory dependency.

These new native cases require hosted compilation and execution. Local Python
syntax checks do not establish CLI success or live publication.
