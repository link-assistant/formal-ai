# Case Study - Issue #557 Adaptive UI And Skins

Analysis and implementation record for
[link-assistant/formal-ai#557](https://github.com/link-assistant/formal-ai/issues/557)
and PR [#643](https://github.com/link-assistant/formal-ai/pull/643).

Companion evidence in this folder:

| Path | Contents |
|---|---|
| [`assets/`](assets/) | The two issue screenshots downloaded from GitHub and verified as PNG images. |
| [`raw-data/issue-557.json`](raw-data/issue-557.json) | Full issue snapshot. |
| [`raw-data/pr-643.json`](raw-data/pr-643.json) | Prepared PR snapshot. |
| [`raw-data/issue-108.json`](raw-data/issue-108.json) | Prior mobile/configurable-input issue. |
| [`raw-data/pr-109.json`](raw-data/pr-109.json), [`pr-111.json`](raw-data/pr-111.json), [`pr-113.json`](raw-data/pr-113.json) | Related merged PRs that already shipped the adaptive composer foundation. |
| [`raw-data/react-bits-research.md`](raw-data/react-bits-research.md) | React Bits repository/framework audit and glass-component comparison captured for the second PR review. |
| [`raw-data/issue557-test-before.log`](raw-data/issue557-test-before.log) | Failing focused Playwright repro before the fix. |
| [`screenshots/after-glass-opacity-settings.png`](screenshots/after-glass-opacity-settings.png) | After screenshot showing the Glass skin opacity slider in settings. |
| [`screenshots/after-material-settings.png`](screenshots/after-material-settings.png) | After screenshot showing the Material skin applied in the app. |
| [`screenshots/skin-*.png`](screenshots/) | 16-shot skin gallery (4 skins × light/dark × desktop/mobile). See [gallery](#visual-polish-pass-2026-07-09). |
| [`screenshots/*-glass-component-*.png`](screenshots/) | Per-component liquid-glass closeups (composer, message cards, top bar, sidebar) in light + dark. See [components](#liquid-glass-on-chakra-components). |
| [`screenshots/color-theme-*.png`](screenshots/) | 14-shot colour gallery (7 palettes × light/dark). See [colour themes](#colour-themes-light--dark). |

## Requirement Trace

| Requirement from #557 | Status | Evidence |
|---|---|---|
| Desktop/tablet buttons should be embedded inside the text field. | Preserved from the #108/#109 work and covered by existing Playwright tests. | `tests/e2e/tests/demo.spec.js` has the one-row composer layout test. |
| Mobile UI should match the same adaptive embedded-composer model. | Preserved from the #108/#111/#113 work. | Existing mobile viewport tests assert same-row action/input/send geometry. |
| Settings should expose multiple skins: default, glass, Material, etc. | Completed. | `UI_SKINS` now includes `material`; settings has a Material option and CSS has `.ui-skin-material`. |
| Glass skin should expose a transparency setting. | Completed. | `glassOpacity` is a persisted preference, settings range input, reset descriptor, user-context field, and CSS variable source. |
| UI should be "as polished as possible" / the skins should look impressive. | Completed (2026-07-09 polish pass). | Glass now renders a living ambient gradient behind heavy `backdrop-filter` frosted surfaces with inner highlights and floating shadows; Material follows M3 tonal surfaces, surface-tint backdrop, elevation shadows, and filled tonal buttons. See the [Visual Polish Pass](#visual-polish-pass-2026-07-09) gallery. |
| Keep the basic Chakra UI skin, but polish the input field + buttons. | Completed (2026-07-09). | Flat skin keeps Chakra; composer textarea is transparent inside the rounded pill and buttons are polished. See [Composer + button polish](#composer--button-polish-all-skins). |
| Material skin should also switch the UI framework from Chakra UI to MUI. | Completed (2026-07-09). | `uiFrameworkForSkin` maps `material` → `mui`; app wraps in `MuiThemeProvider` + `ScopedCssBaseline[data-testid=mui-framework-root]`; composer controls become `MuiIconButton`. See [UI framework switch](#ui-framework-switch-chakra--mui). |
| Support multiple UI frameworks. | Completed. | Chakra (flat/glass/contrast) and MUI (material) coexist, selectable per skin. |
| Study React Bits; add its framework if it differs from Chakra. | Completed without a redundant framework dependency. | React Bits' own application depends on `@chakra-ui/react`; its distributable components are copied as React/CSS or React/Tailwind source, not mounted through a third component framework. See [React Bits framework decision](#react-bits-framework-decision). |
| Apply React Bits best practices and support different Liquid Glass modes. | Completed. | The implementation combines surface frost/refraction, glass icon controls, and responsive fallbacks while preserving accessible native controls. The deliberately excluded WebGL lens/bar/cube modes are documented below. |
| Glass skin ("glass") built on Chakra UI using rdev/liquid-glass-react + Apple glass guidelines. | Completed (2026-07-09). | `src/web/app/liquid-glass.jsx` bridges `liquid-glass-react` as a decorative backing behind Chakra controls; no ready-made integration existed, so built in-house for later extraction. |
| Configurable glass transparency + other glass settings. | Completed. | Three persisted, localized sliders: `glassOpacity`, `glassBlur`, `glassRefraction`. See [Glass configuration settings](#glass-configuration-settings). |
| Multiple configurable colour themes, each in light and dark. | Completed. | Emerald, Ocean, Indigo, Violet, Rose, Amber, and Graphite are persisted, localized, shared by Chakra/CSS and MUI, and captured in the [14-shot gallery](#colour-themes-light--dark). |
| Component screenshots saved in the repo for verification. | Completed. | 10 per-component glass closeups, 16-shot skin gallery, and 14-shot colour gallery under `screenshots/`. |
| Study best-rated market UI kits and compile the data under `docs/case-studies/issue-557`. | Completed. | This file plus `raw-data/ui-kit-*.json`. |
| List requirements, analysis, solutions, and plans. | Completed. | Sections below. |

## Source Screenshots

The issue included two reference screenshots:

- [`assets/issue-557-screenshot-1.png`](assets/issue-557-screenshot-1.png) shows a Claude-like composer where model/actions/status are visually part of the input surface.
- [`assets/issue-557-screenshot-2.png`](assets/issue-557-screenshot-2.png) shows a ChatGPT-like composer with plus, web/app controls, mic, and send inside one rounded input zone.

The important product requirement is not to clone either screenshot literally. The shared principle is one adaptive composer surface where input and actions read as one control on desktop, tablet, and mobile.

## Related Work

Issue [#108](https://github.com/link-assistant/formal-ai/issues/108) already asked for a mobile UI, configurable input, glass transparency direction, and strict e2e coverage. The follow-up PRs established the base:

- [#109](https://github.com/link-assistant/formal-ai/pull/109) added the mobile one-row composer and settings for composer style/action.
- [#111](https://github.com/link-assistant/formal-ai/pull/111) added UI skin settings (`flat`, `glass`, `contrast`) and viewport fixes.
- [#113](https://github.com/link-assistant/formal-ai/pull/113) tightened mobile conversation and composer behavior.

Issue #557 is therefore a completion pass, not a rewrite. The adaptive composer itself already existed; the missing pieces were richer skin choice and user-controlled glass opacity.

## UI Kit Research

Snapshot captured on 2026-07-08 from GitHub and official docs:

| Kit | GitHub snapshot | Relevant lesson for formal-ai |
|---|---:|---|
| [shadcn/ui](https://github.com/shadcn-ui/ui) | 118,465 stars / 9,288 forks | Treat UI as owned code and composable primitives; this supports keeping formal-ai's custom composer while adding skin tokens. |
| [Ant Design](https://github.com/ant-design/ant-design) | 98,610 stars / 54,645 forks | Mature enterprise settings surfaces use explicit controls and predictable density. |
| [MUI Material UI](https://github.com/mui/material-ui) | 98,555 stars / 32,591 forks | Material is best represented as tonal surfaces, 8px-ish radii, and subtle elevation rather than a decorative theme. |
| [Chakra UI](https://github.com/chakra-ui/chakra-ui) | 40,485 stars / 3,623 forks | Component-system ergonomics favor shared controls and theme tokens. |
| [React Bits](https://github.com/DavidHDev/react-bits) | 43,104 stars / 2,007 forks (2026-07-10 follow-up) | Copy-paste visual recipes are useful as design references; audit dependencies before treating a component collection as another UI framework. |
| [Mantine](https://github.com/mantinedev/mantine) | 31,404 stars / 2,329 forks | Practical React UI kits expose controls directly and keep forms compact. |

Additional official design references:

- [Material Design components](https://m3.material.io/components) for the Material skin direction.
- [MUI getting started](https://mui.com/material-ui/getting-started/) for a React implementation of Material Design.
- [Chakra UI docs](https://chakra-ui.com/) for token/component-system patterns.
- [Ant Design docs](https://ant.design/) for dense enterprise control patterns.
- [Apple HIG materials](https://developer.apple.com/design/human-interface-guidelines/materials) and [Liquid Glass overview](https://developer.apple.com/documentation/technologyoverviews/liquid-glass) for the glass/translucency setting.

### React Bits framework decision

The second PR review conditionally requested another framework if React Bits
uses something other than Chakra. The repository audit found that React Bits is
a source-component collection rather than a runtime UI framework: its project
depends on `@chakra-ui/react` 3.x, and the public installation model offers each
component as JavaScript/TypeScript plus CSS/Tailwind source. Adding React Bits as
a third framework would therefore duplicate Chakra rather than satisfy the
multi-framework goal; formal-ai already provides the meaningful split of Chakra
for flat/glass/contrast and MUI for Material.

The audit still influenced the implementation. React Bits' glass catalogue has
three relevant modes:

- `GlassSurface`: CSS/SVG refraction with browser capability detection and a
  blur/saturation fallback. formal-ai uses the same progressive-enhancement
  principle through `liquid-glass-react` plus its CSS frost fallback.
- `GlassIcons`: decorative front/back glass layers around real labelled
  buttons. formal-ai likewise keeps the real button accessible and testable,
  placing a pointer-inert liquid-glass layer behind it.
- `FluidGlass`: WebGL lens, navigation bar, and cube modes backed by Three.js.
  These are immersive hero/demo effects, not appropriate application chrome:
  they add a WebGL renderer, 3D assets, and pointer-following motion to controls
  that must remain fast and predictable. The equivalent product modes here are
  component-appropriate surface glass, icon glass, and adjustable refraction.

The evidence and exact source links are preserved in
[`raw-data/react-bits-research.md`](raw-data/react-bits-research.md).

## Root Cause

The prior #108 work solved the hardest layout problem: the composer is already adaptive and the action/send buttons sit in the input row. What remained for #557 was configuration depth:

- `UI_SKINS` only allowed `flat`, `glass`, and `contrast`, so `material` could not be selected or persisted.
- Glass opacity was hardcoded in `styles.css` as fixed rgba alpha values, so the settings panel could not change transparency.
- The settings reset list, i18n catalog, and persisted preference payload did not know about any glass opacity value.

The focused pre-fix Playwright test in [`raw-data/issue557-test-before.log`](raw-data/issue557-test-before.log) failed while selecting `material`, proving the missing option before implementation.

## Shipped Plan

1. Add a failing Playwright test for #557 that selects Material, verifies `.ui-skin-material`, switches to Glass, moves the opacity slider, and checks persistence.
2. Extend preferences with `glassOpacity`, including normalization, defaults, persistence, reset support, user context, and local command handling.
3. Extend `UI_SKINS` with `material` and add the Material option to the settings panel.
4. Add the conditional glass opacity slider, shown only when the Glass skin is active.
5. Convert glass CSS alpha values to app-level CSS variables driven by `glassOpacity`.
6. Add `.ui-skin-material` CSS using Material 3 tonal surfaces, elevation shadows, the M3 shape scale (16px radii), filled-tonal buttons, and light/dark token overrides.
7. Add localized settings labels for all four supported locales and enforce them in the i18n catalog check.
8. Rebuild `src/web/app.js`, run focused e2e/i18n checks, save after screenshots, and update PR #643.
9. **Visual polish pass (2026-07-09):** deepen the glass and material skins so they read as genuinely premium (see next section).

## Visual Polish Pass (2026-07-09)

Feedback on the first cut ("on screenshot I don't see something impressive") drove a
dedicated polish pass so each non-flat skin looks distinctly designed rather than a
lightly tinted default.

**Glass** now layers a living ambient gradient behind the app and frosts every panel
on top of it:

- `.ui-skin-glass.app` paints a multi-stop radial + linear ambient gradient (blue /
  violet / mint washes in light, navy / purple / teal in dark). The radial alphas
  (light `0.7`, dark `0.68`) and reach (transparent stop at `60-64%`) are tuned so
  the colour washes across the whole conversation canvas instead of only pooling in
  the corners.
- Topbar, context panel, composer field, sidebar bodies, and message cards become
  translucent surfaces with `backdrop-filter: blur(22-28px) saturate(1.7-1.85)`,
  inner-highlight borders (`inset 0 1px 0 rgb(255 255 255 / 0.65)`), and soft floating
  shadows, so the ambient gradient shows through and shifts as content scrolls.
- The opacity slider drives the surface alpha via `--fa-glass-*` CSS variables, so
  users can dial the frost from nearly clear to solid.

**Material** now follows Material 3 rather than a flat tint:

- Tonal surface roles (`--fa-material-surface`, `-container`, `-container-high`) plus a
  primary "surface tint" radial wash on `.ui-skin-material.app`.
- Elevation tokens (`--fa-material-elevation-1/2`) applied to cards and the composer.
- The M3 shape scale (16px radii) on cards, tools, context panel, and composer.
- Filled-tonal action buttons using the secondary-container + accent-text pairing.

**Critical root cause found and fixed during this pass:** the skin classes
(`.ui-skin-glass`, `.ui-skin-material`) are applied to the *same* `<main>` element as
`.app`. The initial CSS used the descendant selector `.ui-skin-glass .app`, which can
never match a single element, so the ambient background never rendered and the skin
looked flat. Switching all six occurrences to the compound selector
`.ui-skin-glass.app` / `.ui-skin-material.app` made the gradients render.

The polish pass is captured as a 16-shot gallery (4 skins x 2 themes x 2 viewports)
regenerated with [`experiments/skin-screenshots.mjs`](../../../experiments/skin-screenshots.mjs):

| Viewport / theme | Flat | Glass | Material | Contrast |
|---|---|---|---|---|
| Desktop light | [png](screenshots/skin-desktop-light-flat.png) | [png](screenshots/skin-desktop-light-glass.png) | [png](screenshots/skin-desktop-light-material.png) | [png](screenshots/skin-desktop-light-contrast.png) |
| Desktop dark | [png](screenshots/skin-desktop-dark-flat.png) | [png](screenshots/skin-desktop-dark-glass.png) | [png](screenshots/skin-desktop-dark-material.png) | [png](screenshots/skin-desktop-dark-contrast.png) |
| Mobile light | [png](screenshots/skin-mobile-light-flat.png) | [png](screenshots/skin-mobile-light-glass.png) | [png](screenshots/skin-mobile-light-material.png) | [png](screenshots/skin-mobile-light-contrast.png) |
| Mobile dark | [png](screenshots/skin-mobile-dark-flat.png) | [png](screenshots/skin-mobile-dark-glass.png) | [png](screenshots/skin-mobile-dark-material.png) | [png](screenshots/skin-mobile-dark-contrast.png) |

## Multi-Framework Skins (2026-07-09)

PR #643 review asked for three things beyond the first polish pass: keep the
basic Chakra UI skin but genuinely polish the composer + buttons; build the
glass skin *on top of Chakra UI* using
[rdev/liquid-glass-react](https://github.com/rdev/liquid-glass-react) with
configurable glass parameters; and make the Material skin switch the whole UI
framework from Chakra UI to **MUI** (so the app supports more than one UI
framework at once). All three are now implemented.

### UI framework switch (Chakra ⇄ MUI)

`uiFrameworkForSkin(skin)` maps the `material` skin to the `mui` framework and
every other skin to `chakra`. When the framework is `mui`, the whole app tree is
wrapped in `<MuiThemeProvider><ScopedCssBaseline data-testid="mui-framework-root">`
and the composer's action/send controls render as `MuiIconButton` instead of
plain Chakra buttons. `ScopedCssBaseline` keeps MUI's reset from leaking onto
the landing page, and MUI's IconButton forwards `data-testid`, `aria-*`,
`disabled`, and `onClick` to its root `<button>` — so the entire existing E2E
suite keeps targeting the same controls regardless of framework. The MUI theme
(`src/web/app/mui-theme.js`) maps the same `--fa-*` design tokens that drive
Chakra, so both frameworks share one source of truth.

Research note (requirement: "make sure there is no ready Chakra UI +
liquid-glass-react solution"): no published package integrates Chakra UI with
`liquid-glass-react`. We therefore built the bridge ourselves in
`src/web/app/liquid-glass.jsx`, isolated so it can later be extracted into its
own repository.

### Liquid Glass on Chakra components

`liquid-glass-react` renders a genuine refractive pane via an SVG
`feDisplacementMap` warp. It assumes Tailwind utility classes and centres itself
for fixed-size floating controls, so `liquid-glass.jsx` (a) injects a tiny
scoped stylesheet defining exactly the utilities the library references (no
Tailwind in the bundle) and (b) uses it as a **non-interactive decorative
backing** behind the real, accessible, test-targeted Chakra control. This keeps
the DOM/behaviour of interactive controls identical across skins while showing
authentic liquid glass behind them. The glass settings map onto library props
through the pure, unit-testable `glassConfigToProps(config)`.

Per-component closeups (glass skin, light + dark) let each Chakra element's
glass treatment be verified in isolation:

| Component | Light | Dark |
|---|---|---|
| Composer pill + liquid-glass buttons | [png](screenshots/light-glass-component-composer.png) | [png](screenshots/dark-glass-component-composer.png) |
| Assistant message card | [png](screenshots/light-glass-component-message-assistant.png) | [png](screenshots/dark-glass-component-message-assistant.png) |
| User message card | [png](screenshots/light-glass-component-message-user.png) | [png](screenshots/dark-glass-component-message-user.png) |
| Top bar | [png](screenshots/light-glass-component-topbar.png) | [png](screenshots/dark-glass-component-topbar.png) |
| Conversations sidebar | [png](screenshots/light-glass-component-sidebar-conversations.png) | [png](screenshots/dark-glass-component-sidebar-conversations.png) |

### Glass configuration settings

The glass skin exposes three sliders (shown only when the glass skin is active),
all persisted as preferences and localized in all four locales:

| Setting | Preference key | Range | Drives |
|---|---|---|---|
| Transparency | `glassOpacity` | 0.35–0.95 | Surface alpha via `--fa-glass-*` |
| Blur | `glassBlur` | 4–40 px | `--fa-glass-blur` (frost strength) + liquid-glass `blurAmount` |
| Refraction | `glassRefraction` | 0–120 | liquid-glass `displacementScale` (light bending) |

### Composer + button polish (all skins)

The composer textarea background is now fully transparent so it blends into the
rounded composer pill instead of showing a lighter inner rectangle (the explicit
PR request). In the glass skin the action/send buttons render as translucent
glass chips (specular rim + `backdrop-filter`) sitting over the liquid-glass
backing; in Material they become filled-tonal `MuiIconButton`s; in flat/contrast
they keep the polished Chakra pill.

The multi-framework behaviour is guarded by
[`tests/e2e/tests/issue-557.spec.js`](../../../tests/e2e/tests/issue-557.spec.js)
(every skin's marker class and transparent textarea, the MUI framework root
mounting only for Material, MuiIconButton controls preserving test ids and
disabled state, the three glass sliders showing only in Glass, live blur, and
all seven colour palettes in both light and dark).

### Colour themes (light + dark)

The colour selector is independent of skin, so every palette can tint Chakra
flat/glass/contrast or the MUI Material theme. Selection persists in
`formal-ai.preferences.v1`; `.app[data-color-theme]` drives shared accent tokens,
and MUI receives the matching concrete brand value. Surfaces and typography
continue to use the light/dark semantic tokens, keeping contrast separate from
brand choice.

| Palette | Light | Dark |
|---|---|---|
| Emerald | [png](screenshots/color-theme-light-emerald.png) | [png](screenshots/color-theme-dark-emerald.png) |
| Ocean | [png](screenshots/color-theme-light-ocean.png) | [png](screenshots/color-theme-dark-ocean.png) |
| Indigo | [png](screenshots/color-theme-light-indigo.png) | [png](screenshots/color-theme-dark-indigo.png) |
| Violet | [png](screenshots/color-theme-light-violet.png) | [png](screenshots/color-theme-dark-violet.png) |
| Rose | [png](screenshots/color-theme-light-rose.png) | [png](screenshots/color-theme-dark-rose.png) |
| Amber | [png](screenshots/color-theme-light-amber.png) | [png](screenshots/color-theme-dark-amber.png) |
| Graphite | [png](screenshots/color-theme-light-graphite.png) | [png](screenshots/color-theme-dark-graphite.png) |

## Verification

Local checks run for this PR:

| Check | Result | Log |
|---|---|---|
| `bun run build:web` | Passed | [`raw-data/build-web.log`](raw-data/build-web.log) |
| `npm --prefix tests/e2e run check:i18n` | Passed | [`raw-data/check-i18n.log`](raw-data/check-i18n.log) |
| `cd tests/e2e && npm run test:local -- --grep "Issue #557"` | Passed | [`raw-data/issue557-test-after.log`](raw-data/issue557-test-after.log) |
| `cd tests/e2e && npm run test:local -- --grep "Issue #(108|557)"` | Passed | [`raw-data/issue108-557-test-after.log`](raw-data/issue108-557-test-after.log) |
| `npm --prefix tests/e2e run check:web-hardcoded-ui` | Passed | [`raw-data/check-web-hardcoded-ui.log`](raw-data/check-web-hardcoded-ui.log) |
| `node --check src/web/app.js` | Passed | [`raw-data/node-check-app-js.log`](raw-data/node-check-app-js.log) |
| `cargo test check_file_size` | Passed | [`raw-data/cargo-test-check-file-size.log`](raw-data/cargo-test-check-file-size.log) |
| `cargo fmt --all -- --check` | Passed | [`raw-data/cargo-fmt-check.log`](raw-data/cargo-fmt-check.log) |

The environment did not have `rust-script`, so the direct `scripts/check-file-size.rs`
and `scripts/check-changelog-fragment.rs` invocations were not available; those
attempts are recorded in `raw-data/check-file-size.log` and
`raw-data/check-changelog-fragment.log`. The Rust test fallback covered the file
size check, and this PR includes the changelog fragment listed above.

Screenshot PNGs were verified by PNG signature and dimensions with Python because
the `file` utility is not installed in this environment.
