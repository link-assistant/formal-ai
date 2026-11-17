# Controlled output: acceptance remains pending

Work in progress in [PR 1585](https://github.com/link-assistant/formal-ai/pull/1585).
The [implementation and acceptance inventory](../../../experiments/issue-1584/README.md)
records every unfinished criterion. Configuration and finite lexical projection
are available experimentally; strict CNL rendering is not established.

## Reproduction and evidence

Run the commands in the experiment README. Local verification covered 86 passing gates and 52 CI-only skips, including
a targeted retry after restoring ShellCheck on PATH. The PR case study retains
the failed full inventory and successful retry. Focused tests recorded 11 output,
3 request-local HTTP and 7 existing server test passes. Native behavior and WASM parity remain
unverified; prior repair head `778a3a00c` passed the native type-and-lint job. Generated registry formatting differs from rustfmt by
generator design; handwritten changed Rust passed the focused format check.

The [bounded raw evidence archive](raw-data/development.tar.gz) includes exact
requests, full captured tool responses, failed attempts, diffs, commands, logs,
and 880 finite projection records. The manifest records the archive cutoff.
The initial capture reported two E-Prime cases as compliant. Those reports
are withdrawn: they lacked independent evidence. The lexical filter also did
not cover archaic contractions. A
reproducing regression and the repair are recorded in the PR case study. The
new 880-case capture reports no compliant cases without independent checkers.
Lexical identity retention is not independent semantic validation. Supplied edits and commands
earn zero autonomous authorship credit: 8 frozen units, 0 independently accepted.

## Requirement conflict

Arbitrary top-N includes top-1. Restricting natural-language output to a single
word cannot express arbitrary propositions with exact meaning through ordinary
natural-language composition. A policy choice for unsatisfiable limits is
required; no answer has been received. This conflict does not excuse the other
implementation gaps listed in the acceptance inventory.

## Primary references

- [ACE 6.6 construction rules](https://attempto.ifi.uzh.ch/site/docs/ace/6.6/ace_constructionrules.html) restrict sentence forms; the current fixtures do not validate ACE.
- [ISO 24495-1](https://www.iso.org/standard/78907.html) describes reader-oriented plain-language principles; the preview does not certify them.
- [NSM resources](https://nsm-approach.net/resources) include language tables and syntax frames. NSM is not a flat token whitelist.
- Official Russian and Chinese NSM tables were retrieved and type-checked locally; their hashes and URLs are in the evidence. The files themselves are excluded. A bounded STE PDF retrieval exceeded the download limit; that does not establish source unavailability.

The [PR case study](../pull-request-1585/README.md) retains the first current-head
CI failures, their repair evidence, and later verification boundaries.
