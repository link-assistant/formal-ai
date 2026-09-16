# Wave F outcomes

One row per task × language. Every row was produced by a run, never by reading a
plan. `chat` is `formal-ai chat` (`solver::solve`); `CLI` is the real
`@link-assistant/agent` binary against a local `formal-ai serve --agent-mode`.

Outcome classes are defined in `README.md`.

## Plan 01 — held-out concept lookup

Binary `formal-ai 0.350.0`, commit `dc9b0574`.

| task | lang | CLI | chat | class | what it lacked |
| --- | --- | --- | --- | --- | --- |
| `isogram` | en | refusal | refusal | `honest_refusal_without_a_trail` | no source consulted; `attempts=` empty |
| `isogram` | ru | refusal | refusal | `honest_refusal_without_a_trail` | same |
| `isogram` | hi | refusal, macaronic | refusal | `honest_refusal_without_a_trail` | same, plus partial Hindi |
| `isogram` | zh | refusal | refusal | `honest_refusal_without_a_trail` | same |
| `isogram` | es | refusal | refusal | `honest_refusal_without_a_trail` | same |
| `lipogram` | en | search + 2 fetches → empty summary scaffold | provider description | `silent_unknown` | a sense bound from the page it fetched |
| `lipogram` | ru | search + 3 fetches → same scaffold | provider description (ru) | `silent_unknown` | same |
| `lipogram` | hi | search + 3 fetches → raw page text incl. `AbortError` | provider description (**en**) | `wrong_answer` | same, plus Hindi routing |
| `lipogram` | zh | search + 2 fetches → same scaffold | provider description (**en**) | `silent_unknown` | same |
| `lipogram` | es | narration in **en**, then scaffold | `unsupported language` fallback | `wrong_answer` | Spanish rows in the response seed |

**Totals.** 10 runs, 0 solved, 5 honest-but-untraceable refusals, 3 silent
unknowns, 2 wrong answers.

## Not run in this pass

`F-2`, `F-3`, `F-4`, `F-8`, `F-10`, `F-11`, `F-12` are out of scope for this
session: they depend on waves I6–I8 or on outward-facing actions (landing a pull
request, filing upstream issues, cutting a release). They are recorded here as
**not run**, not as delivered.
