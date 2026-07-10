---
bump: patch
---

### Added
- Added multi-framework UI skins to the shared web chat: the Material skin now switches the whole UI framework from Chakra UI to MUI (wrapping the app in `MuiThemeProvider` + `ScopedCssBaseline` and upgrading composer controls to `MuiIconButton` while preserving test ids and disabled state), and the Glass skin builds an Apple "Liquid Glass" treatment on top of Chakra UI using `rdev/liquid-glass-react` for issue link-assistant/formal-ai#557.
- Added configurable Glass parameters — transparency (`glassOpacity`), blur (`glassBlur`), and refraction (`glassRefraction`) sliders — as persisted, localized preferences with reset support, shown only when the Glass skin is active.
- Added seven persisted colour palettes (Emerald, Ocean, Indigo, Violet, Rose, Amber, and Graphite), each with light/dark variants shared by the Chakra/CSS and MUI skins.
- Added Playwright coverage (`tests/e2e/tests/issue-557.spec.js`) for every skin's marker class and transparent textarea, the MUI framework root and composer controls, Glass configuration, and the complete light/dark colour-palette matrix.
- Added the issue #557 case study with skin, component, and colour-theme galleries; raw GitHub metadata; UI-kit and React Bits framework research; root-cause analysis; and the implementation plan.

### Changed
- Glass skin and glass composer alpha values now flow through CSS variables driven by the `glassOpacity`/`glassBlur`/`glassRefraction` preferences instead of fixed stylesheet rgba values.
- The composer textarea background is now fully transparent so it blends into the rounded composer pill across all skins.
- Restructured the composer into a single-row pill with the attach and send controls embedded on either side of the auto-growing text area (buttons truly inside the text field) instead of a taller two-row stack, tightening the paddings/offsets on desktop, tablet, and mobile; the text area still grows upward while the controls stay pinned to the bottom edge.
