# Pull Request #1004 — Issue #921 Full-Circle Gate

Pull request: <https://github.com/link-assistant/formal-ai/pull/1004>

Issue: <https://github.com/link-assistant/formal-ai/issues/921>

## Provenance

`raw-data/` preserves the prepared PR metadata and all conversation, inline
review-comment, and review surfaces as captured before implementation. The
issue-side requirements, upstream evidence, live sessions, and root-cause
analysis live in `docs/case-studies/issue-921/`.

## Review Scope

Reviewers should verify that:

- the exact Hive Mind public invocation selects its shipped
  `formalai/formal-ai` Agent path;
- the live effect is produced by Hive Mind's production executor, the installed
  Agent CLI, and the candidate Formal AI server;
- the reverse leg enters through public `formal-ai agent run`, records an
  observed effect, and replays the committed hash chain;
- both exit-23 probes remain nonzero and cannot leave a success commit;
- CI uses a clean evidence directory and uploads complete traces on failure;
- the prepare-only permission shim intercepts only Hive Mind's premature
  permission query and delegates every actual GitHub read;
- the documentation explicitly distinguishes safe public command preparation
  from the production executor call, without claiming that CI mutates GitHub.

No screenshots are applicable because Issue #921 changes a CLI integration
gate and committed text/session evidence, not a visual interface.
