# Evidence-weighted repository statement audit

This case study is the executable acceptance record for [issue #661](https://github.com/link-assistant/formal-ai/issues/661). It extends the original conversational requirement-conflict warning into a repository audit that can be run by a person, CI, or the external Formal AI Agent CLI.

## What is audited

`formal-ai statement-audit` snapshots Git-tracked files and assigns a stable source location and probability to every statement it recognizes in:

- prose lines outside Markdown headings and fenced code;
- line and block comments in supported source-file extensions;
- scalar TOML, YAML, and JSON key/value facts.

The scanner records binary, unreadable, and non-UTF-8 paths under `skipped_paths` instead of silently treating them as text. An initialized repository with no tracked files falls back to a filtered tree snapshot so a first audit is still useful.

This scope is deliberately precise. The audit extracts statement-bearing prose, comments, and structured facts; it does not claim to prove arbitrary executable semantics from every programming language. It currently verifies repository path-existence claims against the complete tracked-path index and weighs explicitly captured external facts. Additional semantic checkers can attach evidence to the same claim and provenance model without changing the audit format.

## Probability and provenance model

Every extracted statement starts with the existing assumed-true prior of `0.6`. Relative Meta-Logic evidence then changes the posterior according to stance, strength, and source tier. Original or first-party sources contribute their asserted mass. An unoriginal source remains visible in the output for provenance but contributes an `effective_mass` of `0`; repeated copies therefore cannot overpower an original source.

Exclusive alternatives, including `always`/`never` requirement pairs, are grouped by subject and predicate. Their evidence-adjusted posteriors are temperature-ranked with softmax, so `relative_weight` sums to one inside a conflicting group. Each conflict proposes an append-only resolution: retract the lower-weight statement by appending a replacement, or split/scope meanings when they are not actually exclusive.

The output is deterministic Links Notation. It contains the posterior and relative weight of every recognized statement, source locations, evidence URLs and capture hashes, contradictions, issue candidates, associations, and usage-derived retention scores. Findings are marked `disposition "issue_candidate"`; the command does not mutate GitHub or file issues automatically.

## Replayable fixture

The fixture at [`examples/issue-661-statement-audit`](../../../examples/issue-661-statement-audit) covers prose, a real and a missing repository path, a code comment, structured facts, and contradictory requirements. Run it directly:

```bash
formal-ai statement-audit \
  --root examples/issue-661-statement-audit \
  --evidence examples/issue-661-statement-audit-evidence.json \
  --output statement-audit.lino
```

The committed [evidence replay](replay-evidence/statement-audit-with-evidence.lino) contains 8 statements, 1 contradiction, and 2 issue candidates. Its W3C capture records:

- primary source: `https://www.w3.org/TR/prov-o/`;
- capture time: `2026-07-19T15:10:43Z`;
- SHA-256: `6b96671ab84faf12ce3f041aca12c3f93a6df2ed242348810743179a68e69555`;
- asserted and effective mass: `1.0`;
- synthetic unoriginal contradictory repost: asserted mass `1.0`, effective mass `0.0`.

The repository-index evidence similarly raises the existing `src/runtime.rs` claim to probability `1.0` and lowers the missing `src/missing_runtime.rs` claim to `0.0`.

## External Agent CLI execution

The task was also executed through the real `@link-assistant/agent` package, not a Rust-only substitute. The release workflow installs the package, and the harness at [`experiments/agent_cli_e2e/run_issue_661_statement_audit.sh`](../../../experiments/agent_cli_e2e/run_issue_661_statement_audit.sh) starts Formal AI's OpenAI-compatible endpoint, gives the installed Agent CLI an isolated temporary repository, and asks it to write `statement-audit.lino`. The recorded run used `@link-assistant/agent@0.25.0`. The planner emits exactly:

```text
formal-ai statement-audit --root . --output statement-audit.lino
```

The recorded run completed in two chat rounds and reported 8 statements, 1 contradiction, and 2 findings. Evidence is preserved as the [agent stream](agent-cli-evidence/agent-stream.jsonl), [agent diagnostics](agent-cli-evidence/agent-stderr.log), [Formal AI server trace](agent-cli-evidence/formal-ai.log), and [resulting audit](agent-cli-evidence/statement-audit.lino). Compatibility warnings in the diagnostics are preserved rather than hidden; they are the known AI SDK compatibility behavior tracked separately by issue #279 and did not change the result.

The release workflow runs the same harness, while unit tests establish that ordinary truth questions do not accidentally route to this repository-wide tool.

## Full-repository scale check

The final implementation was also run against the complete 7,482-path Git snapshot at commit `dcfd8810b61a9b33d4e356c193542b26e971159a`. It audited 181,885 recognized statements, exposed 4,069 contradictions and 18,523 issue candidates, and explicitly listed 218 skipped paths. The resulting 131,592,919-byte Links Notation network contains 3,481,890 lines, so it remains in temporary test storage rather than bloating the repository; the compact [measurement record](full-repository-summary.json) is committed.

On this runner, the release binary completed the warm-cache scan in 13.59 seconds with a 706,232 KiB process high-water mark. The original point-score renderer remained CPU-bound beyond 27 minutes on the pre-optimization working tree. The bulk implementation removes that statement-by-association cross-product, while the fixture replay test proves the optimized renderer is byte-identical for the same audit.

## Learning and follow-up work

Statements, evidence captures, conflicts, and findings are persisted as an associative network. Reads, writes, incoming links, and outgoing links produce deterministic retention scores; bulk export computes all scores in one association pass, growing with expressions plus links instead of their cross-product. The multilingual polarity vocabulary is data-driven, so adding learned markers widens requirement recognition without adding another language-specific control-flow branch.

Contradictions and improbable claims remain linked to their exact source statements as issue candidates. This preserves evidence for human review and makes future automation possible without granting a read-only audit command authority to create external issues.

## Known boundaries

- External evidence capture is explicit input. The deterministic core never performs implicit network requests, so an audit can be replayed byte-for-byte.
- Unrecognized executable semantics retain the `0.6` prior until a checker or captured source contributes evidence.
- Text extraction is source-type-aware at the supported prose, comment, and structured-data boundaries, not a full theorem prover for every language.
- Skipped paths and command errors are reported; they are not silently converted into negative evidence.

These boundaries leave the format open to richer analyzers: new extractors emit located claims, new verifiers emit provenance-bearing evidence, and the probability, contradiction, association, and replay layers remain unchanged.
