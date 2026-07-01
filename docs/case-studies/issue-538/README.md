# Issue 538 Case Study: Make Our Meanings and Words More Detailed

Status: the **concrete verifiable core is implemented and tested** in PR #601;
the **broad aspirational programme is decomposed, researched, and routed to the
roadmap**. This README explains what the issue asked, what shipped, and — openly
— what did not and why.

## Source Material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/538>
- Prepared pull request: <https://github.com/link-assistant/formal-ai/pull/601>
- Raw GitHub data: [raw-data/](raw-data/)
- Requirements decomposition: [requirements.md](requirements.md)
- Per-requirement solution plan: [solution-plan.md](solution-plan.md)
- Online research notes: [raw-data/online-research.md](raw-data/online-research.md)
- The enriched tomato seed block: [raw-data/tomato-block-after.lino](raw-data/tomato-block-after.lino)

## The issue in one sentence

Starting from the concrete observation that the tomato meaning lists Russian
surfaces `помидор`, `помидоры`, `томат` without saying *which is singular or
plural* — and that `помидор` has a plural while its synonym `томат` does not —
the issue asks to make both **meanings** (reverse dictionary: concept → words)
and **words** (direct dictionary: word → concepts) much more detailed, grounded
in real external data, and bidirectionally linked; and then expands into a large
vision: a self-inspecting universal meta algorithm, Rust→WASM workers, CST/AST of
the code in data, mermaid diagrams, an embedded-VS-Code debug view, and solving
the whole task by driving Formal AI through its own Agent CLI.

## Honest scope

The issue deliberately mixes two very different things:

1. A **small, precise, verifiable data improvement** (the помидор/томат example).
2. A **sweeping multi-project vision** (self-hosting Agent CLI, WASM worker,
   AST-in-data, mermaid generation, interactive debug UI, contradiction
   reasoning) — each item a programme in its own right.

Trying to deliver (2) in one pull request would either be a shallow stub of many
things or would block the one improvement (1) that is fully specified and
testable *today*. We therefore made an explicit, recorded decision: **ship (1)
completely and honestly, and decompose (2) into tracked roadmap follow-ups with
research and concrete next steps.** This is itself an instance of the
contradiction-surfacing the issue asks for (requirement R21): the issue's own
scope contains a tension between "make помидор detailed" and "rebuild the whole
system", and we resolve it by separating the verifiable core from the programme
rather than silently doing part of everything. See
[requirements.md](requirements.md) for the full status matrix and
[solution-plan.md](solution-plan.md) for the smallest next step on each tracked
item.

## What shipped (the verifiable core)

The tomato meaning now carries full grammatical detail, grounded in Wikidata,
in every supported language, and the word→meaning direction is explicit:

| Surface     | Language | Part of speech | Grammatical number | Grounded via        |
| ----------- | -------- | -------------- | ------------------ | ------------------- |
| tomato      | en       | noun           | singular           | `L7993-F1`, `Q110786` |
| tomatoes    | en       | noun           | plural             | `L7993-F2`, `Q146786` |
| помидор     | ru       | noun           | singular           | `L3526-F1`, `Q110786` |
| помидоры    | ru       | noun           | plural             | `L3526-F3`, `Q146786` |
| томат       | ru       | noun           | singular           | `L170542-F1`, `Q110786` |
| томаты (new)| ru       | noun           | plural             | `L170542-F7`, `Q146786` |
| टमाटर        | hi       | noun           | —                  | —                   |
| 番茄 / 西红柿 | zh       | noun           | —                  | —                   |

Concretely, this PR:

- adds a new **`grammatical_number`** semantic facet kind to the closed
  `FACET_KINDS` vocabulary (`src/seed/meanings.rs`) and public accessors
  `WordForm::grammatical_number()`, `WordForm::part_of_speech()`, and
  `WordForm::denotations()`;
- adds grounded, multilingual **`grammatical_number` / `singular` / `plural`**
  meanings (`data/seed/meanings-lexical-meta.lino`), grounded in `Q104083` /
  `Q110786` / `Q146786`;
- enriches the **tomato** block (`data/seed/meanings-translation.lino`) so every
  surface pins its part of speech and grammatical number, references its real
  Wikidata lexeme form, and — for `томат` — **adds the previously missing plural
  `томаты`**, closing the asymmetry the issue reported;
- caches the needed Wikidata data (`data/cache/wikidata/entity/Q104083.*`,
  `lexeme/L3526.*`, `lexeme/L170542.*`) so the grounding-closure tests run
  offline;
- covers all of the above with `tests/unit/issue_538.rs` (5 tests):
  grammatical-number tagging, part-of-speech exposure, bidirectional denotation,
  distinct singular/plural pairs per language, and grounded multilingual
  grammatical meanings.

Every surface still denotes the `tomato` meaning (bidirectional word ⇄ meaning),
and the Russian `помидор`/`tomato` surface ordering is preserved so the existing
translation tests (issue #221) stay green.

## What did not ship, and where it went

The following are recorded as roadmap follow-ups with research and a smallest
next step in [solution-plan.md](solution-plan.md): bulk semantics import (R9),
hardcoded-string audit (R10), Rust→WASM worker (R11–R12), CST/AST-in-data and
on-demand rebuild (R13–R14), generated mermaid diagrams (R15–R16), interactive
debug view (R17), full self-inspecting universal meta algorithm (R18–R20),
contradiction detection (R21), and solving the task by driving the Agent CLI to
self-host the change (R22–R24). The Agent-CLI self-hosting requirement is
reported honestly as **not performed**: it is a research programme that would
have blocked the concrete improvement, and claiming otherwise would misreport the
outcome.

## Reproduce

```sh
# the focused tests for this issue
cargo test --test unit -- issue_538

# the grounding-closure and data-floor guards the new data must satisfy
cargo test --test unit -- semantic_grounding
cargo test --test unit -- data_files
```

Run one binary at a time (the issue asks for single-repository test runs to keep
the cargo cache from filling the disk).
