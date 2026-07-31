# Self-hosting authorship evidence

Formal AI issue #706 was split into five smallest review leaves:

1. seed-owned language registry;
2. generated round-trip matrix;
3. partial Spanish data proof;
4. explicit `language_gap` policy;
5. code-free Arabic dry run.

Formal AI authored the shared protocol invariant used to review those leaves,
which is one of five leaves (20%). The production-mode server was driven by the
real Agent CLI through `experiments/issue_706_self_authoring/run.sh`.

- Session: `ses_04be5591bffeIoDvic67voo3lE`
- Authored artifact: `language-protocol-invariant.lino`
- Raw client trace: `agent-cli.log`
- Raw server trace: `formal-ai.log`
- Canonical reviewed copy:
  `data/meta/language-protocol-invariant.lino`

The harness proves four chat-completion rounds, a real file write, a read-back
verification, and byte equality between the captured and canonical artifacts.
