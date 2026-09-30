# Issue #447 Case Study: The Sidebar That Would Not Scroll

Issue [#447](https://github.com/link-assistant/formal-ai/issues/447).
Reported on 0.193.0 (wasm) at 2026-06-13T14:11:52Z by a ru-RU user in
Africa/Nairobi; case study written on the `qa-reasoning-coding-bulk-fixes`
branch.

## What a user saw

On a laptop screen with the browser window at 1280×565 CSS pixels
(physical 1280×720 @1x), the wasm app's left button panel was cut off at
the bottom edge and mouse-wheel scrolling did nothing:

> левая часть с кнопками видна не вся, проматывать её не даёт мышкой.
> интерфейс ужасен.

The frustration line went to the agent, which answered with the generic
unknown-intent pool — technically correct, conversationally tone-deaf for
a complaint about the very surface the user is typing into.

## Requirements

1. The left panel's full content is reachable at short viewports (≤ 565px
   CSS height) — scrolled with the wheel, trackpad, and keyboard.
2. The panel never clips its controls at any width ≥ 320px (mobile column
   collapse included).
3. The dialog half: interface complaints are answered by the seeded
   unknown-opener pool (no Rust literal), which this change pins.

## Root cause

Classic flexbox min-size default: the app shell is a flex row whose
sidebar is a flex column with `min-height: auto` (the CSS initial value),
so when the viewport is shorter than the sidebar's content the column
refuses to shrink, overflows the shell, and because the *shell* owns
`overflow: hidden` (or the scroll container is the body, which the sidebar
escapes via its own unconstrained height), the wheel has nothing to scroll
inside the panel. The bug is viewport-height-dependent, which is why it
survives on tall desktops and bites 720p laptops with browser chrome
docked.

## Solution plan (CSS, web lane)

1. On the shell row: `min-height: 100dvh` (dynamic viewport units so mobile
   browser chrome is not double-counted).
2. On the sidebar: `min-height: 0; overflow-y: auto; overscroll-behavior:
   contain;` — the column may now shrink below its content and scrolls
   itself, wheel and keyboard both.
3. On the button stack: keep `flex: 0 0 auto` buttons in a scrollable
   wrapper so focus jumps scroll the panel (`scrollIntoView` on focus, or
   `tabindex` order preserved).
4. Regression: a Playwright viewport test at 1280×565 asserting the last
   button's bounding box intersects the viewport after `mouse.wheel(0,
   400)`.

Known prior art surveyed: every major app shell (VS Code activity bar,
GitHub's left sidebar) makes the navigation column its own scroll
container with `min-height: 0` — the fix is the standard one, not novel.

## Solution (dialog half, this pull request)

`rust/tests/unit/issue_447_dialog_politeness.rs` pins that the exact
complaint string is answered from the seeded ru opener pool
(`data/seed/unknown-openers.lino`) and that ru remains a declared answer
language. The CSS change belongs to the web surface, which this batch's
wave-3 lanes do not own; the plan above is the hand-off.

## Verification

- Automated: the two tests in
  `rust/tests/unit/issue_447_dialog_politeness.rs` (CI).
- Manual: replay the environment (1280×565, Firefox, Windows) and scroll
  the panel to its last button.
