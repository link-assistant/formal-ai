TASK (tag UPSTREAM-TR): widen the js -> rust leg (R992, R994, R1000) by fixing its two largest upstream blockers in link-foundation/meta-language. That is our organisation's repository, and we have push access.

- **#202:** an item cannot call or read a sibling top-level item of the same module. 209 carried items in scope.
- **#203:** relative ESM imports (`import { f } from './m.mjs'`) should translate as `use crate::m::f`. 417 carried items in scope.

The measurements and the full refusal table are in `docs/case-studies/pull-request-1188/js-rust-translation-scope.md`.

Setup:
- The checkout `/Users/konard/Code/Archive/link-foundation/meta-language` is shared, so do not switch its branch. Create your own worktree:
  `git -C /Users/konard/Code/Archive/link-foundation/meta-language worktree add ../meta-language-tr origin/main -b issue-202-203-sibling-items-and-imports`
- Self-translation is JavaScript: `js/src/self-translation.js`, with tests in `js/tests/self-translation.test.js`. Run the JS tests with node or bun only.
- **Never run cargo or any Rust build locally**, and do not run scripts that launch one. If the repository's parity rules require a Rust twin, write it, check it with `rustfmt --edition 2024 --check` only, and let the repository's CI build it.
- Install JS dependencies only inside the worktree's `js/` folder, and only if the tests need them. The disk has about 6 GB free, so remove `node_modules` when you finish.

Do:
1. Fix #202:
   - A top-level item may call or read a sibling item of the same module.
   - The Rust output references it by name, and the translated module carries both items.
   - Add tests in the repository's style.
2. Fix #203:
   - A relative ESM named import (`import { a, b as c } from './m.mjs'`, `'../x/y.mjs'`) emits `use crate::m::{a, b as c};`, with the module path derived from the specifier.
   - Bare package imports and namespace imports still refuse, with a clear diagnostic.
   - Add tests.
3. Measure the effect on formal-ai without writing to it:
   `node scripts/translate-js-rust.mjs --verify --meta-language /Users/konard/Code/Archive/link-foundation/meta-language-tr`
   (run from `/Users/konard/Code/Archive/link-assistant/formal-ai`). Report the translated and carried counts before and after. Today's baseline is 249 translated and 2120 carried. Do not run `--write` in formal-ai; LEAD regenerates the fixtures after the upstream merge.
4. Commit, push the branch and open a PR against meta-language main. The PR body says what changed and links #202 and #203 with "Fixes". Write a changelog fragment if the repository uses them. Watch the PR's CI with `gh pr checks`, fix every failure, and push again until it is green. Do not merge; LEAD decides that.
5. Where a small, well-scoped edit fits, try it with Formal AI first:
   `node experiments/js_dogfood/drive.mjs --dir <worktree> --steps 8 "<prompt>"`
   Never put an escaped quote inside the payload; use «» instead. Log each attempt as a ledger row T250–T259 in `docs/case-studies/pull-request-1188/formal-ai-dogfood.md`, and each new failure as a gap in `experiments/formal_ai_subagent/gaps.md`.

Final report: the PR URL, its CI state, the before and after counts, and anything still refused that blocks the next tier.
