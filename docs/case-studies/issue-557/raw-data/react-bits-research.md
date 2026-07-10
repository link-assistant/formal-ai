# React Bits Research Snapshot

Captured 2026-07-10 from the upstream repository and website in response to
PR #643 review feedback.

## Repository snapshot

- Repository: https://github.com/DavidHDev/react-bits
- Website: https://reactbits.dev/
- Description: open-source animated, interactive, customizable React components
- Popularity at capture: 43,104 stars and 2,007 forks
- Upstream update timestamp: 2026-07-10T01:12:42Z
- Distribution model documented upstream: 130+ copy-paste components, with
  JavaScript/TypeScript and CSS/Tailwind variants

## Framework audit

The upstream application `package.json` includes `@chakra-ui/react` 3.20.x and
`@chakra-ui/icons` 2.2.x. It also includes Tailwind CSS for styling variants and
specialized animation/rendering libraries, but it does not expose a distinct UI
framework that formal-ai should mount alongside Chakra and MUI.

Sources:

- https://github.com/DavidHDev/react-bits/blob/main/package.json
- https://github.com/DavidHDev/react-bits#readme

Decision: do not add a nominal “React Bits framework.” React Bits shares Chakra
in its own site and distributes components as owned source. Formal-ai already
has two actual selectable component frameworks: Chakra and MUI.

## Glass implementations compared

### GlassSurface

Source: https://github.com/DavidHDev/react-bits/tree/main/src/content/Components/GlassSurface

- Generates a per-instance SVG displacement map and feature-detects SVG filter
  support.
- Falls back for Safari/Firefox and browsers without `backdrop-filter`.
- Exposes frost, saturation, blur, distortion, channel offsets, blend mode,
  dimensions, and border radius.
- Uses `ResizeObserver` so the displacement map follows responsive dimensions.

Applicable lesson: refraction must be progressive enhancement. Keep a readable
CSS frost fallback and resize-safe surfaces.

### GlassIcons

Source: https://github.com/DavidHDev/react-bits/tree/main/src/content/Components/GlassIcons

- Uses a real labelled `<button>` and separates decorative back/front layers.
- Keeps the icon decorative with `aria-hidden` while the button has a label.

Applicable lesson: the glass effect must never replace the semantic control.
formal-ai's `GlassBacking` is pointer-inert beneath the real button and preserves
the existing test ids, accessible labels, focus, disabled state, and handlers.

### FluidGlass

Source: https://github.com/DavidHDev/react-bits/tree/main/src/content/Components/FluidGlass

- Offers `lens`, `bar`, and `cube` modes.
- Uses Three.js, React Three Fiber/Drei, WebGL framebuffers, 3D model assets,
  pointer-following motion, and transmission materials.

Decision: do not put WebGL hero effects into the chat application's persistent
controls. Their runtime/assets and pointer-following behavior are disproportionate
for productivity chrome. The relevant product modes are instead surface frost,
glass icon chips, and user-adjustable refraction, all with CSS fallback.

## Resulting formal-ai principles

1. Use Chakra for flat/glass/contrast and actually switch to MUI for Material.
2. Keep interactive HTML controls above non-interactive decorative glass.
3. Persist opacity, blur, and refraction independently.
4. Preserve responsive geometry and transparent textarea integration.
5. Provide a no-filter fallback and keep effects scoped to the selected skin.
6. Test every skin and every light/dark colour palette through public DOM state.
