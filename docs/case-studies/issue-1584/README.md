# Controlled output: acceptance remains pending

Work in progress in [PR 1585](https://github.com/link-assistant/formal-ai/pull/1585).
The [implementation and acceptance inventory](../../../experiments/issue-1584/README.md)
records every unfinished criterion. Configuration and finite lexical projection
are available experimentally; strict CNL rendering is not established.

## Reproduction and evidence

Run the commands in the experiment README. The final local gate run recorded
86 passes, 52 CI-only skips and no failures. Focused tests recorded 10 output,
3 request-local HTTP and 7 existing server test passes. Rust execution remains
unverified pending CI. Generated registry formatting differs from rustfmt by
generator design; handwritten changed Rust passed the focused format check.

The [bounded raw evidence archive](raw-data/development.tar.gz) includes exact
requests, full captured tool responses, failed attempts, diffs, commands, logs,
and 880 finite projection records. The manifest records the archive cutoff.
Only 2 projection cases have all requested constraints checked and passing.
The other cases report violations or missing validators; lexical identity
retention is not independent semantic validation. Supplied edits and commands
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
