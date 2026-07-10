# Issue 557 UI evidence matrix

This matrix is the completion checklist required by the authoritative PR review.
An item is complete only when its behavior is asserted, its image has been
visually inspected, and any finding is recorded here. Generated files alone do
not count as verification.

## Dimensions

| Dimension | Required values |
|---|---|
| Viewport | desktop 1280x860; tablet 820x1180; mobile 390x800 |
| Scheme | light; dark |
| Framework/skin | Chakra flat; Chakra glass; MUI flat; MUI Material; Chakra contrast |
| Brand palette | emerald; ocean; indigo; violet; rose; amber; graphite |
| Glass appearance | balanced; clear; frosted |
| Composer state | empty/disabled; focused; populated; multiline; attachment menu open; sending |
| Interaction state | rest; hover; keyboard focus; active; disabled; loading; error; empty |

The full-page framework/skin matrix is 3 viewports x 2 schemes x 5 skins (30
images). Palette evidence uses each palette in both schemes and automated token
checks across each skin. Glass-mode evidence uses all three modes in both
schemes, with reduced-motion behavior asserted separately.

## Named UI areas

| Area | Behavior to verify | Visual evidence |
|---|---|---|
| Top bar and navigation | accessible names, focus, active mode, overflow | full pages; topbar closeups |
| Conversations | empty/list/selected/deleted, create/copy/delete controls | full pages; conversations closeups |
| Settings and reset bar | conditional controls, persistence, individual/all reset | settings closeups |
| Example prompts | hover/focus, multilingual labels, long text | prompts closeups |
| Tools and trace | status chips, long descriptions, collapse/isolate controls | tool closeups |
| Context/desktop panels | resize, overflow, surface hierarchy | full pages and panel closeups |
| Chat transcript | user/assistant/thinking/tool cards, long/multilingual content | full pages; card closeups |
| Composer | single pill, transparent textarea, all states and viewports | composer state matrix |
| Mobile drawer | open/closed, focus, scroll containment | mobile state images |
| Dialogs/menus | composer attachment menu, report/memory surfaces, focus order | state images |

## Inspection record

| Date | Artifact set | Finding | Resolution |
|---|---|---|---|
| 2026-07-10 | Existing 16 full pages and 10 glass closeups | No tablet coverage; only five glass areas; no state/mode images; prior settings image predates blur/refraction; glass topbar icon glyphs appeared as dark blocks in one capture set | Open; recapture from the final bundle with web fonts settled and expand the matrix |
| 2026-07-10 | Existing desktop/mobile composer frames | Single pill and symmetric edge insets are visible; mobile multiline/disabled/focus evidence was absent | Added behavioral matrix tests and desktop/tablet/mobile composer visual baselines |
| 2026-07-10 | Final 30 full pages, 18 glass closeups, six glass-mode frames, and 14 palette frames | Desktop/tablet/mobile layouts are structurally sound in light and dark; MUI Flat and Material both show MUI-backed controls; glass mode differences are visible without changing DOM semantics; palette accents are distinct. Settings closeups are clipped to visible content to avoid blank scroll tails. | Completed for implementation review; PR remains draft until human visual review and finalization approval |

## Final gate

- [x] 30 full-page skin/scheme/viewport images generated and inspected.
- [x] Named-area closeups generated and inspected in light and dark.
- [x] Composer state matrix generated and inspected for Chakra, glass, and MUI.
- [x] Six glass-mode images generated and inspected; reduced motion asserted.
- [x] Palette contrast and cross-skin token assertions pass.
- [x] Automated visual baselines pass on the final commit.
- [x] Relevant Issue 557 local suite passes.
- [ ] Full local E2E suite passes without unrelated failures.
- [ ] Final PR ledger links each requirement to code, test, CI, and image evidence.

## Local verification notes

- `npm --prefix tests/e2e run test:local -- --grep "Issue #557" --workers=1`
  passed: 52 tests.
- `npm --prefix tests/e2e run test:local` exercised the Issue 557 functional
  and visual tests successfully in the full suite, but the overall run did not
  complete cleanly because unrelated `issue-336` and `issue-501` tests timed
  out and the suite hit its 900-second global cap near the tail.
- `cargo test --all-features --verbose` passed: 1463 tests, 0 failures, 2
  ignored; doctests passed.
