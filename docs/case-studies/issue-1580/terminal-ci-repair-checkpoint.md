# Terminal CI failure repair checkpoint

The published ff6b097f head finishes all 26 workflows: 23 succeed, one is
explicitly skipped, and Layered CI plus CI/CD Pipeline fail. The retained raw
logs identify ten missing extracted native modules, stale release/TypeScript/
notation assertions, and three JavaScript shards reaching their existing
360-second execution deadline.

Twelve explicit relative module paths repair the native admission failure,
including two additional owners with the same layout. Removing each attribute
or substituting a wrong path fails the new maintained regression. A tiny
rustfmt module-resolution fixture fails before the path and passes afterward;
no local native compilation is claimed. Existing Rust assertion bodies remain
unchanged. The reviewed governed witness retains 221 cases and 968 unsupported
items; only the reviewed fixture/source hashes change.

Offline fixture tests now own a fresh temporary source cache. The regression
first proves that offline mode replays a populated caller cache, then proves
the isolated fixture reads the shipped baseline and restores the caller.
Existing product cache behavior and CI deadlines remain unchanged.

Update stale assertions to the checked artifact adapter, actual ordered
TypeScript cases and the canonical notation parser, with negative controls.
Preserve all generated registry entries in readable lines. Splitting complete
response records preserves the ordered runtime snapshot and every bootstrap
response field, while reducing the current file-size warning count to 64.

All 141 local gates were attempted: 88 passed and 53 explicitly require hosted
execution. The focused assertion/cache/bootstrap checks pass, as do the affected
post-path-change specification/documentation gates and maintained issue entry.
Actual native compilation and the complete 903-file hosted JavaScript suite
must still pass on the next head. The 64 remaining warnings, full eight-template
practice comparison and other original acceptance requirements remain open.
Supplied code and commands receive zero independent-authorship credit.

Evidence: experiments/issue-1580/evidence/checks/terminal-ci-repair/ and
experiments/issue-1580/evidence/ci/native-families-final-ff6b097f/.
PR: https://github.com/link-assistant/formal-ai/pull/1581
