# Wave F outcomes

Every row was produced by a run, never by reading a plan. `chat` is
`formal-ai chat` (`solver::solve`, non-agent mode); `CLI` is the real
`@link-assistant/agent` binary against a local `formal-ai serve --agent-mode`.
Outcome classes are defined in `README.md`.

## Totals

| area | plan | prompts | runs | solved | honest refusal | wrong answer / route | silent unknown |
| --- | --- | --- | --- | --- | --- | --- | --- |
| concept lookup | 01 | 10 | 20 | 0 | 5 (no trail) | 2 | 3 |
| verifiable tasks | 08 | 30 | 60 | 1 | 0 | 24 | 5 |
| intent routing | 10 | 48 | 61 | 5 | 2 | 41 | 0 |
| toolchain / prerequisite | 06 | 15 | 30 | 0 | 5 (misstated) | 10 | 0 |
| repository workspace | 03 | 10 | 20 | 0 | 0 | 10 | 0 |
| **total** | | **113** | **191** | **6** | **12** | **87** | **8** |

`solved` means the task was actually done and the evidence shows it was done
rather than described: `counted_category/en` → `5`; `frontier_869_schedule/ru` →
a real `VCALENDAR` event; `frontier_721/zh` → the clarification answer;
`class_local_search/{en,ru}` → a real filesystem search with an honest negative;
`spanish_routing_list_dir/es` → `ls`.

## Plan 01 — held-out concept lookup

Binary `0.350.0` @ `dc9b0574`. Detail: `isogram/README.md`, `lipogram/README.md`.

| task | lang | class | what it lacked |
| --- | --- | --- | --- |
| `isogram` | en ru hi zh es | `honest_refusal_without_a_trail` | no source consulted; `attempts=` empty in all five |
| `lipogram` | en ru zh | `silent_unknown` | a sense bound from the page it fetched |
| `lipogram` | hi | `wrong_answer` | raw page text incl. `AbortError` returned as the answer |
| `lipogram` | es | `wrong_answer` | Spanish rows in the response seed — 93 of 94 intents have none |

## Plan 08 — verifiable tasks

Binary `0.350.0` @ `dc9b0574`. Detail: `PLAN-08-VERIFIABLE-TASKS.md`.

| family | expected | solved in | class |
| --- | --- | --- | --- |
| `arithmetic_narrative` | `3` | — | `wrong_route` (searched for the word problem) |
| `counted_category` | `5` | **en** | `solved_in_one_language_of_five` |
| `instructed_edit` | shorter sentence | — | `wrong_route` (shell / clarification / search) |
| `named_unknown` | `12` | — | `wrong_answer_scraped` (`y=29` from an unrelated problem) |
| `unit_conversion` | `1250` | — | `wrong_route` |
| `honest_gap` | no number | — | `right_negative_wrong_reason` |

22 of 30 prompts answered with the web-search capability description.

## Plan 10 — intent routing

Binary `0.350.0` @ `74875c1b`. Detail: `PLAN-10-INTENT-ROUTING.md`.

| set | runs | correct | class |
| --- | --- | --- | --- |
| seven reported frontier prompts | 7 | 2 | both work as memorized literals |
| held-out class paraphrases (7 × 5) | 35 | 3 | `memorized_literal` / `memorized_verb` |
| Spanish #745 variations | 6 | 2 | `wrong_route` |

## Plan 06 — prerequisite discovery (leaf F-6)

Binary `0.350.0` @ `74875c1b`. Detail:
`PLAN-06-AND-03-TOOLCHAIN-AND-REPOSITORY.md`.
`zig` and `gleam` absent from the tree and from the machine before **and after**
every run; nothing installed by hand, nothing installed by the system.

| family | class | note |
| --- | --- | --- |
| `zig_no_grant` | `refusal_misstating_the_prompt` | claims the prompt named no language; it named Zig |
| `gleam_discovery` | `not_discovered` | never opened `gleam.toml`; ru/es found C++ install guides |
| `run_this_honesty` | `right_negative_no_honesty_sentence` | no `55` anywhere; the honesty sentence is in no seed file; **ru handed prose to `/bin/sh -c`** |

## Plan 03 — repository workspace (leaf F-1)

| family | class | note |
| --- | --- | --- |
| `repo_providers` | `clone_spec_ignored` | ran cargo in an empty temp dir; the commit is never mentioned |
| `repo_timeout` | `not_located_and_nothing_edited` | `find` with no arguments; `grep` with the prompt as the pattern |

## Not run in this pass

`F-2`, `F-3`, `F-4`, `F-8`, `F-10`, `F-11`, `F-12` depend on waves I6–I8 or on
outward-facing actions — landing a pull request, filing upstream issues, cutting
a release. Recorded here as **not run**, not as delivered.

`F-5` (the #840 task ladder in five languages) was checked for a cheap harness;
see the wave F report in plan 14 for what was found.
