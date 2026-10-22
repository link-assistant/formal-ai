# Browser dependency installation and unrelated runner repositories

The normal merge of PR #1394 produced public release v0.355.2 and its Docker
images. Its main release workflow passed. The subsequent [browser publication
job](https://github.com/link-assistant/formal-ai/actions/runs/39861900487/job/120114498151)
failed while installing Playwright system dependencies, after selected-source
WASM compilation and locked npm installation had succeeded.

The Ubuntu catalogs were reachable. A preinstalled Microsoft repository returned
HTTP 403 for its Noble InRelease catalog; APT exited 100 and Playwright stopped.
The full retained job log has SHA-256
`c21074f76ab89de9924689dbd6411b16794e843b7f4c038bd998ce4a17dd672d`.
The exact three diagnostic lines are retained as
`rust/tests/fixtures/apt/browser-microsoft-403.log` and used by the regression.
This public CI diagnostic is test evidence, not training data. No user data or
third-party source implementation is included in that fixture.

`scripts/with-ubuntu-apt-sources.sh` prepares the existing official mirror
alternate and copies the runner's distribution-managed `ubuntu.sources` without
changing its contents. A command-scoped APT configuration selects that copy and
an empty legacy source list. Unrelated source files and their cached lists remain
untouched. The APT configuration crosses Playwright's sudo boundary through an
owned temporary sudo wrapper that preserves only `APT_CONFIG`; browser downloads
continue under the invoking user. Temporary configuration is removed on exit.

The same wrapper covers browser publication, public download verification and
the two browser E2E dependency installations. The release engine captures both
APT helpers in its immutable workflow protocol. The existing setup/job deadlines,
Playwright arguments, signature checks, package/Worker checks and E2E oracles
remain in force. There is no additional attempt or ignored signature failure.

`browser-ubuntu-apt-isolation.test.mjs` reproduces the captured failure with the
original privilege boundary, then verifies successful isolated update/install,
byte-exact source preservation, original browser arguments and home directory,
fatal signature/browser failures, unsafe input rejection and temporary cleanup.
The existing mirror, release deadline and historical protocol suites remain
separate controls. Real Linux dependency installation and the released browser
Worker are verified by CI; local tests install no system packages or products.

This is a manual release-blocker fix under the user's release-first instruction.
A separately delegated JavaScript Formal AI attempt to create the changelog leaf
stopped with `unowned-create-request`, made zero tool calls and produced no diff.
No part of this change is credited as Formal AI authored. General agentic
self-coding and the original three-client/all-eleven-scenario acceptance in issue
#1364 remain pending their genuine released-client evidence.
