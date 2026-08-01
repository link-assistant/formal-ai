# Repository statement-audit summary

Run on 2026-08-01 with the issue implementation at commit `bf246935` and the
staged documentation tree `a8752cc4fcbc350b64b411aa8d4dfc6bf4d3b1c6`:

```bash
cargo build --bin formal-ai
target/debug/formal-ai statement-audit \
  --root . \
  --output /tmp/issue885-final-staged-tree-audit.lino
```

Formal AI reported:

| Measurement | Result |
| --- | ---: |
| Statements | 215,945 |
| Contradiction candidates | 4,660 |
| Findings | 20,884 |
| Skipped paths | 243 |
| Supplied evidence captures | 0 |
| Serialized `resolved_text` fields | 147 |
| Serialized antecedent ids | 144 |
| Issue-885 case-study statements | 201 |
| Four new legal-guide statements | 188 |

The generated Links Notation graph had 4,085,030 lines and SHA-256
`bccbcdc8b91676a406786466706a2e17e8a8f6cb898da71dbf0b11b780ab1ff6`.
It is intentionally not committed: the hash identifies the reviewed output,
and the command plus Git tree reproduces it without adding a roughly 152 MiB
generated file to every checkout. This summary was added after that run, so it
is the only case-study file outside the identified tree; including its own
output digest would create a self-reference.

## Finding from the run

The first staged-tree run exposed two false resolutions where Markdown
soft-wrapped continuation lines began with “its” and “that.” They followed
unterminated lines rather than completed statements. A failing regression now
covers both shapes, and commit `bf246935` requires a completed preceding
sentence before resolving a leading reference. The final run contains neither
false resolution.

The 4,660 contradiction candidates and 20,884 findings are triage signals, not
4,660 established repository defects. No external evidence file was supplied
to this broad run, so unsupported claims remain visible at their priors and
mechanically exclusive structured values can appear as contradiction
candidates. Current legal and model facts were reviewed separately against the
primary-source ledger in [`raw-data/online-research.md`](raw-data/online-research.md).
The URL probe reached 25 sources with HTTP 200; the three official OpenAI pages
returned HTTP 403 to command-line retrieval and were inspected through the
official documentation/browser path instead.

The smaller real-Agent-CLI audit in
[`agent-cli-evidence/statement-audit/`](agent-cli-evidence/statement-audit/)
supplies an exact evidence capture and proves that the public command persists
resolved evidence, a dependency link, and contextual probability across a
whole two-document workspace.
