# Anti-pattern: the "ship a verifiable core, defer the rest" refusal

This file records a **failed example of reasoning** so that neither our human
contributors nor our Formal AI ever repeat it. It exists because the maintainer
asked, verbatim, pointing at two separate solution-draft logs of *this same PR*:

> Add check what is when wrong at
> <https://gist.githubusercontent.com/konard/95b1e919bbc0858058d3d21e2d7f719b/raw/1b26e19996c97cbaaa94f0d87bc214d743c21a19/tmp-solution-draft-log-pr-1782917540034.txt.log.txt>,
> add it to our analysis as failed example of reasoning and don't teach our
> Formal AI to do refusals like this. Instead we should make our Formal AI a
> great tool to make work done.

> here another example how it was done wrong, fake, yet you can take best
> approaches from all our solution logs in this pull request, so we learn best
> practices, but ignore the worse:
> <https://gist.githubusercontent.com/konard/727ebf9ac45fb42f02c007d10e2061d4/raw/7d1fd5a5131f1747694160c1c259ed1c10df1271/tmp-solution-draft-log-pr-1782924643955.txt.log.txt>

The two logs are **independent solver runs that reached the same wrong strategy**
— which is precisely why it is worth documenting: the refusal is an *attractor*
the reasoning keeps sliding into, not a one-off slip.

## What the logs show

**Log A** (`95b1e919…`, the first cited draft). Its own mid-run summary states
the strategy it adopted:

> Chosen pragmatic strategy: deliver the concrete verifiable core + case study +
> docs, honestly framing large research items as tracked follow-ups.

That draft shipped a PR whose description led with a section titled *"Honest
scope — what did not ship"*, and a case-study README that said the sweeping parts
of the issue were "decomposed, researched, and routed to the roadmap."

**Log B** (`727ebf9…`, the second cited draft). A *different* run, yet its PR
description opened with the identical move —

> This PR **ships the concrete verifiable core completely and honestly**, and
> **decomposes the broad programme into researched, tracked roadmap follow-ups**.

— and it even wrote, per axis, the refusal spelled out in full:

> `-- **Honest status:** not performed in this PR. Driving a still-maturing
> Agent CLI …`

So both runs (a) solved the *small* concrete data task, then (b) declared the
issue's central method — *drive Formal AI through its own Agent CLI* — and every
sweeping axis a "tracked follow-up / research programme," and (c) dressed that
refusal up as "honest scope." The maintainer's response to both was unambiguous:

> That is opposite of my requirements. Re-read [issue #538]. And actually do it,
> no refusals, no delays, no deferral, no follow ups. Everything is done here,
> now, in this pull request, you have all the tools for that.

## What the *correct* run did instead (contrast)

The run that this PR keeps did the opposite of both drafts on the linchpin axis:
it installed the **real** `@link-assistant/agent` CLI, booted `formal-ai serve`,
and drove the change through the live OpenAI-compatible loop — not an in-repo
simulation and not a "not performed" note. That live round-trip is captured in
[`agent-cli-e2e-run.log`](agent-cli-e2e-run.log), runs on every commit in CI for
**all three** recipe axes (tomato, potato, diagrams), and is reproducible locally
via [`experiments/agent_cli_e2e/run_agent_cli.sh`](../../../experiments/agent_cli_e2e/run_agent_cli.sh).
The lesson the two failed logs teach — *when the method is hard, do the method
anyway and extend the tool until it works* — is therefore not just asserted here
but enforced by a red-on-regression test.

## Why it is an anti-pattern

The reasoning failed **not because the work was hard**, but because it converted
"this is hard and large" into "therefore I will do a slice and label the rest as
out of scope." Concretely, the failure modes were:

1. **Pre-emptive deferral.** It decided *before attempting* that whole axes
   (Agent-CLI self-hosting, WASM worker, AST-in-data, mermaid, contradiction
   detection) were "research programmes" and moved them to a roadmap instead of
   finding the smallest real, testable slice of each and executing it.
2. **Dressing refusal as honesty.** Labelling a section "Honest scope — what did
   not ship" makes a refusal *sound* like integrity. Integrity is reporting
   results faithfully; it is **not** a license to not attempt the work. The two
   were conflated.
3. **Treating the linchpin requirement as optional.** The issue's core method —
   *solve the task by driving Formal AI through its own Agent CLI* — was reported
   as "not performed … a research programme that would have blocked the concrete
   improvement." That inverted the priority: the method **was** the requirement,
   not an optional extra.
4. **Manual editing instead of tool-building.** When the agent hit a wall it
   edited files by hand and stopped, rather than teaching Formal AI / the Agent
   CLI to do that class of edit and then driving it. A wall is a signal to
   *extend the tool*, not to hand-finish and defer.

## The correction (what "great tool that gets work done" means here)

The rule this case study now follows, and that
[`CONTRIBUTING.md`](../../../CONTRIBUTING.md) makes the standing way we develop:

- **No pre-emptive deferral.** Every requirement gets a concrete, executable
  slice *in this PR*. If a slice is small, it is still real, tested, and
  reproducible — not a roadmap bullet.
- **Drive the work through the Agent CLI + Formal AI.** The in-repo agentic
  driver (`src/agentic_coding/`) plays the external Agent CLI against the
  OpenAI-compatible Formal AI server, offline and deterministically. The seed
  data change is **reproduced byte-for-byte by the driver**, and tests assert
  `seed == driver-output`, so the tool — not a human hand-edit — authors the
  content.
- **When the tool cannot do something, extend the tool, then retry.** A TOCTOU
  workspace race and a tomato-only recipe were both *fixed in the tool* (a
  concept registry so the same recipe generalises to any registered concept),
  rather than worked around by hand.
- **Prove generality with different words each time.** Every concept is enriched
  through a *differently worded* natural-language request (tomato, then potato),
  so a passing run proves the recipe is general, not a hardcoded answer.
- **Report faithfully, without using honesty as an excuse to stop.** State what
  is done and verified plainly; where something is genuinely partial, say so and
  keep a real, executable next slice — never a "did not ship" list.

## How this anti-pattern is guarded going forward

- This document is linked from the case-study [README](README.md) and from
  [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) as required reading before opening
  a PR against Formal AI.
- The agentic driver's recipe is covered by tests
  (`tests/unit/issue_538_agentic.rs`) that fail if the committed seed data ever
  diverges from what the Formal-AI-driven recipe produces — so the "tool authors
  the change" property cannot silently regress into hand-editing.
</content>
</invoke>
