## Reference example

A real working session from 2026-09-04, uploaded as a public gist:

**https://gist.github.com/konard/f3e509d03be7a159a8a49377fafd2120**

- `2026-09-04-conversation-export-no-thinking.txt` — clean human-readable export (no thinking blocks)
- `2026-09-04-session-daf8b2c5-with-thinking.jsonl` — raw session JSONL with all 34 thinking blocks included

The gist contains the very command that produced it: the session file was uploaded twice, so the second upload captured the upload command itself inside the session data. This is the ground-truth example this issue refers to.

## Goal

Raise the level of thinking in **all** situations — not only in obviously hard ones — to at least the precision demonstrated in the reference dialog, keeping only the best from it, and then push the reasoning **deeper than the reference**.

The end state this issue moves us toward: an AI that works on **formal logic without an LLM**. During the transition the LLM is scaffolding; every judgment it makes must be replaceable by a formal, checkable procedure.

## What the reference demonstrates (adopt)

- **Evidence before claims.** Every statement about the world is backed by an executed command and its observed output, never by impression. When a deletion failed twice (shell glob semantics, then argument-list limits), the approach was changed, the action was re-verified (target folder measured at 0 bytes, free space re-read), and only then success was declared.
- **Adversarial self-check before conclusions.** Before acting on a finding, the dialog asks: "what would have to be true for this problem NOT to exist?" — and searches for that closing mechanism first, instead of collecting confirming evidence.
- **Honest failure reporting.** Partial failures are reported as partial failures (root-owned leftovers deliberately kept, with the reason stated), not silently smoothed over into a green result.
- **Verify-after-act loops.** After every mutating step the state is re-measured and the numbers are shown, so the conclusion is checkable by the reader.

## What to drop from the reference

Project-specific tooling noise, absolute paths, session-management details — everything that is not reusable as a reasoning pattern. The standard is the reasoning discipline, not the particular cleanup task.

## Requirements

1. **Deep reasoning everywhere.** The depth of thinking shown in the reference (and in its 34 thinking blocks) is the floor, not the ceiling — it applies to trivial requests as well as to hard ones.
2. **Information gathering during reasoning.** While working on any task, the system must collect instructions and best practices from the internet on *how to do this class of task well*, and **formalize them** — turn them into explicit, machine-checkable instruction sets. The purpose of formalization: any class of tasks becomes solvable by following such formalized instructions, without relying on the model's intuition.
3. **Documentation and trusted sources by default.** Official documentation and other trustworthy sources are consulted as a default step of reasoning, not as an afterthought.
4. **Source-trust hierarchy, computed not assumed.** Whether something deserves trust must itself be determined through primary sources: government sites, encyclopedias (Wikipedia), reviews and answers to questions, comments in social networks, and so on. Trust is derived from how close a statement is to its primary source, and conflicts are resolved toward the more primary one.
5. **Refutation-first reasoning.** Before leaning toward any conclusion, the system must actively search for refutations — and deliberately seek a **wide variety** of them: different mechanisms, different sources, different assumptions. Only after the refutations are logically refuted, or the alternative viewpoint is positively confirmed/proven, may the system lean toward a conclusion. If neither succeeds, the honest output is "not confirmed and not refuted," together with what exactly blocked the check.
6. **Formal-first pipeline.** Requirements 1–5 must be expressed as formal, checkable procedures rather than stylistic guidance, so the same conclusions remain reachable when the LLM is removed from the loop.

## Contradiction audit (to be done within this issue)

Everything in this repository that contradicts the vision above must be found and updated as part of this issue:

- [ ] Enumerate docs, prompts, handlers and examples that define how reasoning is performed today
- [ ] Flag every place that assumes trust without deriving it from primary sources
- [ ] Flag every place that allows single-source conclusions or confirmation-seeking instead of refutation-seeking
- [ ] Flag every place where reasoning depth is conditional on task difficulty or on prompting
- [ ] Update each flagged place to the standard described here
- [ ] Add a regression test / harness case derived from the reference dialog, so the standard is checkable, not aspirational

