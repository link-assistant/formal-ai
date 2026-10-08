# Progressive delivery: the progressive JPEG method

The architect's direction of 2026-10-08
([architect note](architect-notes/2026-10-08-progressive-jpeg-javascript-first-and-automated-translation.md),
R1188-U28 to R1188-U30) is to attack every problem, and to develop every pull
request, by the progressive JPEG method of
[§ 167 of Artemy Lebedev's "Ководство"](https://www.artlebedev.ru/kovodstvo/sections/167/):
"at any second, any project is 100% ready, although it may be only 4% worked
out". A progressive JPEG first shows the whole picture blurred, and each pass
sharpens all of it. A baseline JPEG shows the top rows sharp while the rest is
missing.

## How it applies here

- **The whole first, at low resolution.** A new requirement is first recorded as a row, then measured by a gate or ratchet, even if the measure reads 4%, and then delivered end to end in the simplest general way. Only after that is it refined. One requirement is not polished while another has no row, no measure or no working path.
- **Every pass leaves a valid release.** Each commit is complete at its resolution: the gates pass, the generated files are current and the release builds (R1188-U27). A pass that cannot finish is shipped behind its measure. It is never left half-applied.
- **The lowest level goes first.** [`docs/progressive-plan.md`](progressive-plan.md) is generated from the requirement-status ledger by `node scripts/render-progressive-plan.mjs --write` (gate `check-progressive-plan`). It places every requirement on a level:
  1. **recorded**: the row exists, and nothing measures it yet;
  2. **measured**: a test pins it, but it is not delivered;
  3. **partial**: delivered in part;
  4. **implemented**: delivered, with a test.

  The next pass is the lowest non-empty level. `data/meta/progressive-plan-ratchet.lino` holds the plan: a recorded, measured or partial level may grow only by the requirements recorded since it was written, so a pass that refines a high level while a lower one grows fails the gate. `--write` lowers its ceilings and never raises one.
- **Refinement is a ratchet.** The resolution knob of a delivered capability is its measured value (term survival, recall, coverage, the number of specializations, the jobs over 30 minutes). A pass moves the number, and the ratchet holds it.
- **Each problem the same way.** A bug or a gap is first answered by the smallest general rule that covers its class (R1188-U1), with a test, and then sharpened. A special case that makes one prompt sharp while the class stays blurred is the baseline-JPEG mistake.

## JavaScript first (R1188-U29)

The JavaScript root is the fast loop:
- Formal AI runs from its JavaScript source with no Rust build;
- the JavaScript requirements pass first;
- the other roots follow by translation.

More of the work itself goes through Formal AI from JavaScript. Every delegated step is a row of the [dogfood ledger](case-studies/pull-request-1188/formal-ai-dogfood.md), and the [tally](case-studies/pull-request-1188/formal-ai-tally.md) shows what Formal AI did beside the edits made by hand. The plan's "JavaScript first" table counts which root's test pins each requirement; the progressive passes raise the share pinned by a JavaScript test.

## Automated translation with recorded workarounds (R1188-U30)

Translation between JavaScript, TypeScript, Rust and the meta language is automated, so less code is translated by hand:

| Direction | How | State |
| --- | --- | --- |
| JavaScript → TypeScript | `node scripts/translate-es.mjs --write` | Automated: `ts/` is regenerated from `js/` (349 files), layout included. |
| JavaScript → Rust | `node scripts/translate-js-rust.mjs --write --fetch`, using [link-foundation/meta-language](https://github.com/link-foundation/meta-language) self-translation at the pinned commit, plus the recorded workarounds of `data/meta/translation-workarounds.lino` | Partial. `data/meta/js-rust-translation.lino` counts 281 of 2513 top-level items translated by meta-language and 31 more by workarounds. The other 2201 are carried as hand-written Rust twins. `data/meta/translation-blockers.lino` names every construct that keeps each carried item out. |
| Rust → JavaScript | `node scripts/check-specification-in-javascript.mjs` | A workaround for tests, not for code. It carries 136 of the 1169 Rust specification tests to the browser worker, and 102 of them pass. The gaps are in `data/meta/specification-javascript-gaps.lino`. No Rust module is translated to JavaScript yet. meta-language's self-translation at the same pin does run Rust → JavaScript: measured once on 2026-10-09, it translates 4,377 of the 51,040 top-level items of the 774 files in `rust/src`, and something in 314 of them. That leg has no ledger and no CI step yet. |
| TypeScript → Rust | none | Not automated. `ts/` is generated from `js/`, so the JavaScript → Rust leg covers it. |
| Source ↔ links (meta) | the self-AST census (R480, R482) | One to one by file; the full lossless form is R1188-U3. |

Where relative meta logic (RML) or the meta language (ML) cannot translate a construct yet, a **temporary workaround** in this repository does it. The workarounds in use are recorded in `data/meta/translation-workarounds.lino`. They are import pruning, one crate per root for the compile check, and the lowering of string methods, `.push` and array callbacks before the translator runs (`scripts/lib/translation-workarounds.mjs`, `scripts/lib/translation-lowering.mjs`). They were chosen by the full-refusal scan `data/meta/translation-blockers.lino`, biggest first. A projection's header names the workarounds it needed, and the ledger counts their items apart from upstream's. Each workaround is recorded with the upstream issue that will retire it. It is removed when RML and ML, developed in parallel, handle the construct; the census ratchet then shows the gain. Upstream fixes, such as the long-line layout of meta-language #217, are preferred whenever they can land.
