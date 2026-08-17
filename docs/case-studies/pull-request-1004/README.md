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

The branch also closes the two defects the gate proved nothing about — a
working transport carrying no work, reported in
[hive-mind#2158](https://github.com/link-assistant/hive-mind/issues/2158) with
evidence in [PR #2159](https://github.com/link-assistant/hive-mind/pull/2159).
Reviewers should additionally verify that:

- an unmarked caller preamble no longer outranks the objective stated after a
  line-anchored delimiter, so `Your prepared working directory: …` no longer
  plans `pwd` (issue #907, rung `R916-09`);
- a policy clause naming a privileged command no longer authorizes it, so
  `When running sudo commands, …` no longer plans bare `sudo`, while an
  imperative naming the same command still selects it (rung `R916-10`);
- a dispatched repository work item is read before its execution is judged
  impossible, and `planned_not_executed` survives only where the capability is
  genuinely unavailable or the work item names no artifact — which is never
  invented into one (issue #904);
- the policy leads and the read-step wording are seed data, so the next
  conditional form or language is added by editing a `.lino` file;
- the Hive Mind CI pin does not fall back below 2.12.5, the first release
  carrying the #2159 boundary this gate runs against.

No screenshots are applicable because Issue #921 changes a CLI integration
gate and committed text/session evidence, not a visual interface.
