# Required documentation ignores false closing fences

The complete aggregate review found that `check-required-documentation.mjs`
accepted every JavaScript whitespace character after a closing Markdown fence.
A non-breaking space, em space, narrow non-breaking space, byte-order mark,
vertical tab or form feed therefore closed an example and let its fake
`## Quick Start` heading satisfy the required documentation gate.

Reproduce with a README containing a three-backtick opening line, three
backticks followed by U+00A0, then `## Quick Start`. The heading remains inside
the code example, so the gate must report the missing required section.
The maintained `documentation-only-ci-policy.test.mjs` now exercises both
backtick and tilde delimiters with all six invalid suffixes. The test failed
against the original checker and passes with ASCII space/tab-only closure.
Eight valid closing controls preserve ordinary spaces and tabs.

All 37 tests in the existing CI test file pass, as do the documentation CLI,
workflow policy, file-size, abbreviation and readable-code gates. Complete
before/after output and hashes are retained in
`experiments/issue-1580/evidence/checks/document-fence-whitespace/`.
These observations are local JavaScript execution; hosted execution is pending
the next complete CI round. Supplied edits receive zero independent-authorship
credit and do not complete the wider issue qualification.
