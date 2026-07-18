# Issue 745: generalized multilingual intent routing

The regression suite in `tests/issue_745.rs` sends more than 300 routing requests through the production `plan_chat_step` entry point. Each primary capability has at least 15 variations in English, Russian, Hindi, and Chinese, plus explicit object-type collision cases.

Evidence:

- `red-regression.log` records five failing intent groups before the fix.
- `green-regression.log` records all six groups passing after the fix.
- `agent-create-regression-test.jsonl` records the repository's own Agent CLI creating the test.
- `agent-apply-fix.jsonl` records the Agent CLI applying `experiments/issue_745_intent_routing.patch`.
- `agent-general-change-plan.lino` preserves the plan emitted by Formal AI for the test artifact.

The first attempt through `examples/self-coding/run.sh --live` failed before reaching Formal AI because the installed solve wrapper rejected the `formal-ai` model name. The successful evidence therefore uses the contributing guide's direct local-server and Agent CLI runbook.
