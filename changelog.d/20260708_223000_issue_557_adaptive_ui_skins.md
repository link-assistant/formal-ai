---
bump: patch
---

### Added
- Added multi-framework UI skins to the shared web chat: the Material skin now switches the whole UI framework from Chakra UI to MUI (wrapping the app in `MuiThemeProvider` + `ScopedCssBaseline` and upgrading composer controls to `MuiIconButton` while preserving test ids and disabled state), and the Glass skin builds an Apple "Liquid Glass" treatment on top of Chakra UI using `rdev/liquid-glass-react` for issue link-assistant/formal-ai#557.
- Added configurable Glass parameters — transparency (`glassOpacity`), blur (`glassBlur`), and refraction (`glassRefraction`) sliders — as persisted, localized preferences with reset support, shown only when the Glass skin is active.
- Added Playwright coverage (`tests/e2e/tests/issue-557.spec.js`) for every skin's marker class, the MUI framework root, MUI composer controls, the glass sliders, the blur variable wiring, and the transparent composer textarea.
- Added the issue #557 case study with a 16-shot skin gallery, per-component liquid-glass closeups, raw GitHub metadata, UI-kit research, root-cause analysis, and the implementation plan.

### Changed
- Glass skin and glass composer alpha values now flow through CSS variables driven by the `glassOpacity`/`glassBlur`/`glassRefraction` preferences instead of fixed stylesheet rgba values.
- The composer textarea background is now fully transparent so it blends into the rounded composer pill across all skins.
