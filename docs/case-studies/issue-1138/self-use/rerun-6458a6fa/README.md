# Re-run at commit `6458a6fa`

The wave F loop requires that previously failing tasks are re-run against a
rebuilt binary as the siblings' work lands, and that a task which now succeeds is
recorded with the commit that made it pass.

**Rebuilt.** `formal-ai 0.350.0` from `6458a6faba5e0b3b7f15d75cfea44bacb4bd6e14`,
which carries five sibling commits landed during this session (plan 09 leaves
1–9 and the shared `Need` / `Evidence` records of plan 00 C1).

**Re-run.** The plan 01 held-out words (10 prompts), the plan 06 toolchain
families (15 prompts) and the plan 03 foreign-tree family (5 prompts), through
`formal-ai chat`.

**Result.** Every answer is **byte-identical** to the first pass:

```
$ for each of the 30 prompts: cmp <first-pass>/chat-answer.txt <rerun>/chat-answer.txt
(no answer changed between dc9b0574 / 74875c1b and 6458a6fa)
```

and all 21 wave F tests are still red at `6458a6fa`, with the one green guard
still green.

**Nothing to report as fixed.** Not one previously failing task passes at any
commit reached during this session. That is the expected shape at this point: the
sibling waves that landed are plan 09's handler-migration ratchet and the shared
record types, neither of which touches the routes these tasks exercise. The
re-run is recorded because the absence of movement is itself a measurement, and
because a later session can re-run the same three case files and compare against
these files rather than against a memory of them.
