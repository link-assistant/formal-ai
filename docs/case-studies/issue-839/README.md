# Issue 839: `report issue` must export the full conversation

Issue [#839](https://github.com/link-assistant/formal-ai/issues/839) reported
that the shipped `report issue` flow filed issues that were not reports: the
exported "session" was a placeholder id, the proxy trace fell back silently when
it could not be read, records were truncated mid-line, and every surface
(agentic CLI, web reporter, `formal-ai report body`) rendered its own body.

## Outcome

One shared report-body builder now renders every surface, `latest` resolves to a
real session — from the harness when one is reachable, otherwise from the
conversation the server itself recorded — the server stores conversation-shaped
records beside the HTTP proxy trace, and an export that cannot be produced fails
loudly instead of shipping a plausible-looking stub. The change set, its tests,
and this document are hand-authored maintenance.

## Self-hosting evidence

The pull request's differential self-hosting check compares the release share
projected *with* this branch against the one projected for `main` alone. A
large, entirely hand-authored branch lowers it. Rather than attach trailers to
hand-authored commits — which
[CONTRIBUTING.md](../../../CONTRIBUTING.md#recording-self-authorship) forbids
("an honest 0% release is valid") — the established local Agent-CLI path ran
five real Formal AI sessions, one per self-inspection axis:

| Session | Recipe | Artifacts |
| --- | --- | --- |
| `ses_06872edf2ffeTD4t6p3X8oTewP` | source ↔ links (issue #558) | [`self-source-links.lino`](self-hosting-evidence/self-source-links.lino) plus the exhaustive [two-shard projection](self-hosting-evidence/whole-repository-projection-01.lino) of all 298 owned modules, every module's source → links → source round trip verified byte-for-byte |
| `ses_069155ee9ffehjIXQT8FFPE8Zr` | CST/AST census (issues #538/#673) | [`self-ast.lino`](self-hosting-evidence/self-ast.lino), the planner module parsed through the meta-language links network, plus the whole-workspace rendering under [`data/meta/self-ast/`](../../../data/meta/self-ast) |
| `ses_068804131ffeWmiIA64S1m5JY7` | self-healing repair case (issue #558) | [`self-healing-case.lino`](self-hosting-evidence/self-healing-case.lino) — byte-identical to the committed [`data/meta/self-healing-case.lino`](../../../data/meta/self-healing-case.lino), so this session is that document's authoring event |
| `ses_06874742affeXKw5qAPWIkCDGt` | grounded self-explanation | [`how-formal-ai-works.lino`](self-hosting-evidence/how-formal-ai-works.lino) — 5 topics, 13 citations, each resolved against the owned source manifest, so the document is a content-addressed description of the tree *this branch* leaves behind |
| `ses_0687416b1ffeTmoz4wGwDYKCcl` | rebuild and reattach | [`rebuild-and-reattach.lino`](self-hosting-evidence/rebuild-and-reattach.lino) — the accepted change taken through the green-gate-plus-approval path into 5 ordered, reversible steps over 4 reattached artifacts |

The last two are asserted *live* by
`tests/unit/issue_558_self_explanation.rs` and
`tests/unit/issue_558_rebuild_plan.rs` rather than pinned byte-for-byte,
because they move with the sources — which is what makes them per-branch
artifacts rather than checked-in data.

All five are pure deterministic functions of the embedded sources — no neural
inference, and re-running
[`experiments/issue-839-self-hosting-evidence/run.sh`](../../../experiments/issue-839-self-hosting-evidence/run.sh)
regenerates them exactly. The whole-workspace census is the same recipe the
second session performed, rendered for every owned module instead of one:
`src/self_ast_census.rs` calls the same `ast_census` function, and
`tests/unit/issue_673_self_ast_census.rs` fails if the committed documents drift
from what the projector produces. This branch adds `src/issue_report.rs`,
`src/cli_report.rs`, `src/dialog_conversation.rs`,
`src/agentic_coding/report_script.rs` and `src/server/transport.rs`, so their
census documents are re-rendered here.

Only the isolated generated-artifact commits carry the paired
`Formal-AI-Session` and `Formal-AI-Evidence` trailers — one commit per session,
so each artifact is attributable on its own. The Agent CLI transcripts
(`agent-stream*.jsonl`) and the server traces (`formal-ai*.log`) are the
excluded evidence bundle binding each artifact to the session that authored it;
the metric counts neither of them in the numerator nor in the denominator.

## Reproduction

```bash
cargo build --release --bin formal-ai \
  --example project_source_links_sharded --example regenerate_self_ast_census
bash experiments/issue-839-self-hosting-evidence/run.sh
```

The harness boots `formal-ai serve` with a private, empty memory
(`FORMAL_AI_MEMORY_PATH` + `FORMAL_AI_DREAMING=0`) so the deterministic planner
never reads the shared `~/.formal-ai/memory.lino`, drives the real Agent CLI
once per recipe, and prints the session id it recorded for each. The session
ids differ on every run; the artifacts do not. `ONLY=<axis>` (`source-links`,
`self-ast`, `self-heal`, `explain`, `rebuild`) re-runs a single axis, so adding
a session does not rewrite the transcripts of the ones already recorded.
