# Template picomatch lock investigation

Four actual npm audits reproduce GHSA-fc5q-39xc-vjpf in current default-branch
locks: JavaScript root (13 high affected dependency nodes), JavaScript universal
app (10), Python artifact-action development tooling (3), and C++ root tooling
(3). Every advisory chain leads to the same newly reviewed advisory; these
counts are not distinct vulnerabilities. Exact current source/blob identities,
raw audit responses, complete issue inventories and hashes are preserved here.

The Python observation does not establish reachability in the shipped artifact
runtime bundle. The npm consumer-downgrade suggestions do not identify a
patched picomatch release. No advisory exclusion or severity change is proposed.
The associated bounded regression, maintained local backport and installed
consumer controls live in ../picomatch-single-branch/ and
rust/tests/web/picomatch-security-backport.test.mjs. Reports are prepared for the
three repositories and will use this checkpoint's immutable commit URL after
publication. Reporting is not an upstream repair or wider issue completion.

Development execution uses the JavaScript Formal AI command driver. All supplied
commands and source changes receive zero autonomous-authorship credit.
