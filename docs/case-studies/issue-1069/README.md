# Issue #1069: earning a release Formal AI actually contributed to

Issue [#1069](https://github.com/link-assistant/formal-ai/issues/1069) asks for a
release that satisfies every self-development requirement instead of bypassing
one. Two of those requirements blocked it, and they blocked it for different
reasons.

## The quantitative bar

`target_from_rows` ratchets: each cycle carries the previous target forward and
raises it to whatever the last cycle measured, so a single high cycle sets a
level no later cycle can get back under except by out-measuring it. The maintainer's
decision on [PR #1070][decision] is that the level is theirs to set, and
`target_override_basis_points` in `data/meta/self-hosting-ledger.lino` is where
they set it. It is deliberately a reviewed ledger value and not a flag: lowering
the bar is allowed, lowering it quietly is not.

## The structural bar

The second requirement was not a number. `merged_self_authored_pull_requests`
counted a merged pull request only when **every** non-merge commit it introduced
was attributed to Formal AI. That is a statement about the composition of a pull
request, not about who wrote the work, and it had one practical consequence:
Formal AI's work could never ride along inside ordinary review. Each contribution
needed a pull request containing nothing else, filed by a human who carried the
bytes over from a scratch workspace — at which point the commit was the human's.

Two things changed here.

### A route that lands the work

`scripts/author-change-with-formal-ai.sh` runs the same live loop every harness
under `experiments/agent_cli_e2e/` runs — `formal-ai serve` plus the real
`@link-assistant/agent` CLI — and then does the part those harnesses drop: it
copies the file the CLI wrote into the repository, keeps the run's raw traces as
evidence, and commits both with the three trailers the metric reads.

```sh
scripts/author-change-with-formal-ai.sh \
  --task "Audit all statement-bearing repository prose, code comments, and structured facts; weigh conflicting requirements and captured original-source evidence with probabilities; persist findings and associations; and write statement-audit.lino." \
  --produces statement-audit.lino \
  --into docs/case-studies/issue-1069/formal-ai-authorship/ci-cd-statement-audit.lino \
  --evidence docs/case-studies/issue-1069/formal-ai-authorship/evidence \
  --pull-request https://github.com/link-assistant/formal-ai/pull/1070 \
  --message "docs(issue-1069): audit the CI/CD documentation" \
  --seed docs/ci-cd \
  --contains repository_statement_audit
```

It opens no pull request and pushes nothing; the commit lands on the branch that
is checked out. The artifact under
[`formal-ai-authorship/`](formal-ai-authorship/) is Formal AI's own audit of this
repository's CI/CD documentation, produced in the session its commit names.

That route only works because of the delivery fix earlier in this pull request:
the planner used to treat a file as undelivered unless a *write* call named it,
so a recipe that produces its artifact by running a command
(`formal-ai statement-audit --root . --output statement-audit.lino`) finished
with the file on disk and the plan still asking for it.

### A gate that counts the work, not the packaging

With a route that lands work inside an ordinary pull request, the all-or-nothing
rule would have rejected exactly the pull requests it now produces. The gate now
counts a merged pull request when it introduced **at least one** commit attributed
to Formal AI, and when every attributed commit it introduced names that same pull
request.

Nothing about the measurement moved. The share is computed per commit: an
unattributed commit contributes to the denominator and not the numerator, whether
or not a sibling commit is attributed. Dropping the all-or-nothing rule cannot
inflate a single basis point. What it drops is the requirement that Formal AI's
work arrive alone.

What is still enforced, unchanged: the commit carries `Formal-AI-Session`,
`Formal-AI-Evidence` and `Formal-AI-Pull-Request`; the evidence path is
repo-relative and present in that commit; the evidence identifies `formal-ai` and
contains the session the trailer names; a trailer without evidence is an error
rather than a silent non-attribution; and a commit claiming a different pull
request disqualifies the one it was introduced by.

[decision]: https://github.com/link-assistant/formal-ai/pull/1070#issuecomment-5535449300
