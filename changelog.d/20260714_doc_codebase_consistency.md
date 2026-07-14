---
bump: patch
---

### Fixed
- Corrected documentation that had drifted from the codebase: `ARCHITECTURE.md` no longer claims a nonexistent `Event::Impulse` enum variant or `parent_id`/`language`/`surface` event fields, lists all 18 `SolverConfig` knobs instead of 9, drops four event kinds that no longer exist, renumbers a duplicated section 4.4, counts five rule shapes instead of four, documents the VS Code surface, and states honestly that ~26,700 lines of solver logic still live in `src/web/worker/*.js` (issue #658) rather than implying the JavaScript boundary is already narrow.
- `CONTRIBUTING.md` no longer carries template boilerplate: the title and clone URL name `formal-ai` instead of `rust-ai-driven-development-pipeline-template`, the project-structure tree reflects the real repository, and the line-limit rule distinguishes the 1000-line Rust cap from the 1500-line `.lino`/worker-JS caps.
- `docs/meta-algorithm.md` records the previously undocumented recursive-core recipe (issue #559) and corrects the procedural how-to record counts (11 roles, 8 functions, 6 stages, 4 parity pairs) to match the grounding suite.
- `docs/ci-cd/troubleshooting.md` invokes `rust-script scripts/publish-crate.rs` instead of a `node scripts/publish-crate.mjs` file that does not exist.
- `docs/testing/agentic-cli-tools.md` and the generated `docs/diagrams/agentic-recipes.md` now state their real scope instead of implying the multi-CLI CI matrix (issues #625/#671) and the full planner router set are already covered.
- Fixed the stale `SolverConfig::selection_mode` doc comment, which described `Legacy`/`Registry`/`Compare` variants that R344 replaced with `Off`/`Record`.
