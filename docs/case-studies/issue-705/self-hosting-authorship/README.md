# Same-task self-hosting evidence

Formal AI served its symbolic `formal-ai` model to the real external Agent CLI
for one reviewed declarative leaf of issue #705. Final session
`ses_04262d1beffeStZQyvlKMA8qCg` used the client-owned write tool, read the file
back through its shell tool, and completed four chat-completion rounds.

- `anticipation-invariant.lino`: exact Agent-authored artifact;
- `agent-cli.log`: raw client transcript;
- `formal-ai.log`: raw server/tool projection;
- `decomposition.lino`: five-leaf authorship accounting;
- `failed-literal-wording/`: the first attempt, retained because the literal
  parser copied the task word `exactly` into the artifact.

Only the clean second session is counted. One of five reviewed leaves is
Agent-CLI-authored, or 20%. The implementation, tests, recipe, and case-study
analysis were written with Codex assistance and are not claimed as Formal AI
authorship. The artifact bytes are pinned by the issue #705 unit suite.

Reproduce with:

```bash
cargo build --bin formal-ai
experiments/issue_705_self_authoring.sh
```
