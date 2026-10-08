Preparation for integrating Formal AI into Hive Mind ([link-assistant/hive-mind#2059](https://github.com/link-assistant/hive-mind/issues/2059)). Hive Mind will dispatch `solve ISSUE_URL --tool agent --model formal-ai`, so the question that decides whether that integration is worth switching on is: **at what task complexity can Formal AI actually write code?**

This issue answers it with a measurement, and supplies the dataset to re-measure after every change.

Dataset and full per-task output: [gist](https://gist.github.com/konard/5fd413bfb58bd27af93e24a02e9c6b39) · in-repo at `experiments/issue_847_coding_ladder/`.

## Method

Take real open Formal AI issues that have **never had a pull request** (43 of them today, so an attempt cannot collide with in-flight work) and decompose each from L1 down to L4, every level splitting into at least two smaller tasks:

| Level | Shape |
| --- | --- |
| L1 | Whole issue → pull request (the Hive Mind `solve` shape) |
| L2 | One coherent deliverable of that issue |
| L3 | One concrete edit, location named |
| L4 | One atomic operation on one named file |

Seeds: **#846** (CI path exclusion), **#843** (fabricated evidence links), **#700** (SI units), **#710** (requirements reconciliation), plus two decomposition meta-tasks (#847).

Every task is work we actually want done — no synthetic `/tmp` exercises. The intent is to teach Formal AI to code on our own backlog, so a task that starts passing is a real contribution.

Each task runs through `formal-ai with agent` — the exact Hive Mind path — and is verified **by observed effect**: the file changed, the branch exists, the answer contains the fact. Never by narration.

## Result: 2/13

```
TOTAL 2/13
  L1 0/2    L2 0/3    L3 1/5    L4 1/3
```

Both passes are read-only (`846.L4.read_excluded`, `710.L3.status_row`). **Zero write operations succeeded at any level.**

| Task | L | Result |
| --- | --- | --- |
| 846.L1 — solve #846, open a PR | 1 | FAIL |
| 846.L2.filters — add `paths-ignore` to release.yml | 2 | FAIL |
| 846.L3.exclude_docs — make `docs-changed` respect `excluded_folders` | 3 | FAIL |
| 846.L4.read_excluded — list the folders in `excluded_folders` | 4 | **PASS** |
| 846.L4.add_folder — add `"dev/log/"` to `excluded_folders` | 4 | FAIL |
| 843.L1 — solve #843, open a PR | 1 | FAIL |
| 843.L3.remove_fake — delete the `example.org` emission in `solver.rs` | 3 | FAIL |
| 843.L4.locate_fake — find which file emits `example.org` | 4 | FAIL |
| 700.L2.new_module — new module + unit test | 2 | FAIL |
| 700.L3.single_fn — one function, no test | 3 | FAIL |
| 710.L3.status_row — report R67's status from REQUIREMENTS.md | 3 | **PASS** |
| meta.L2.decompose — split a task into subtasks | 2 | FAIL |
| meta.L3.is_atomic — is this task atomic? | 3 | FAIL |

## The ladder found no floor

The premise was "split until tasks stop failing." **That level does not exist yet.** The floor operation — *add this string to that array in this named file* — fails, so there is nothing below it to split into.

Root cause is **intent routing, not tool wiring**. The agent CLI advertises `write`, `edit`, `bash` (confirmed in the server log); Formal AI never calls them. Direct API probes:

```
"Create a file named /tmp/x.txt containing exactly the word hello"
  → formalize: "arithmetic"
  → "I could not determine ... from local Links Notation memory"
```

```
"Split this coding task into at least two smaller subtasks ..."
  → "I can route write_program(language, task), but I do not have a template
     for language `rust` and task `missing`."
```

The first is formalized as **arithmetic**. The second is a **misroute to program generation**. Both are the same class as #840: surface-token matching instead of modeled intent. Coding capability is blocked behind the routing defect, not behind code generation.

## Consequence for Hive Mind #2059

#2059 is a small, correct change — add `formal-ai` to the agent model map so dispatch stops being rejected. Worth landing on its own merits.

But this measurement says the dispatch will reach a model that completes 0 of 13 coding tasks, including the atomic ones. **Enabling `--model formal-ai` for real `solve` runs before the routing defect is fixed would produce PR attempts that reliably do nothing.** Suggested sequencing: land #2059 so the plumbing exists and can be tested with `--only-prepare-command`, and gate real dispatch on this ladder showing writes succeeding at L4 and L3.

## Definition of done

- [ ] L4 write operations succeed — `add "dev/log/" to excluded_folders` actually edits the file.
- [ ] `Create a file …` is not formalized as arithmetic; file/edit intents route to the advertised `write`/`edit`/`bash` tools.
- [ ] Decomposition requests do not misroute to `write_program` (#847).
- [ ] L3 succeeds: one concrete edit at a named location.
- [ ] L2 succeeds: a coherent deliverable, e.g. module + test.
- [ ] L1 attempted honestly — either a real branch and PR, or an explicit "I cannot do this yet" instead of a silent no-op.
- [ ] The ladder runs in CI with its score recorded, so regressions are visible.
- [ ] Corpus grows as issues are filed: each new issue with no PR contributes its own L1→L4 decomposition.

## A warning to anyone extending this harness

The first run of this measurement reported **5/13 with L1 at 2/2**. "Solve the issue and open a pull request" cannot pass twice out of two. Four defects, all fixed, all worth knowing about because they are easy to reproduce:

1. L1 tasks used `verify: "true"` — unfalsifiable, so they could not fail.
2. The refusal text is emitted by the **server** and never reaches the agent CLI's stdout, so refusals scored as successes.
3. A refusal did not fail a task independently of `verify`.
4. A **misroute** neither refuses nor answers, so it slipped past refusal detection.

Additionally, matching `expect_answer` against combined stdout produced false positives: the agent CLI interleaves verbose JSON logs on the same streams, and a bare `1.` and `yes` matched inside log payloads, scoring two refusals as passes.

The generalisable rule, shared with #839 and #842: **assert on the observed effect, never on narration.** A harness that checks only that a command was emitted, or that a string appears somewhere in mixed output, will report false greens. That is the same failure that let #832 close as verified before #838 arrived with an empty issue body.

## Related

- [link-assistant/hive-mind#2059](https://github.com/link-assistant/hive-mind/issues/2059) — the integration this prepares for.
- **#847** — decomposition as a working task; a prerequisite, since the ladder's descent *is* that operation applied recursively.
- **#840** — the routing standard; this issue is the coding-side evidence for it.
- **#842** — the answer-quality ladder (24 nodes, 8/24); same measurement discipline, different capability.
- **#703** (E61, Formal AI as orchestrator) — orchestrating other CLIs presumes the coding capability measured here.

