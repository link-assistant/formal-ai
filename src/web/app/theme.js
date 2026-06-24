// Issue #550: Chakra UI theme bridge.
//
// The app's colours live as `--fa-*` CSS design tokens defined once in
// styles.css and themed across the light / dark / OS-preference axis there
// (see the header comment in src/web/styles.css). This module maps those
// tokens 1:1 into Chakra semantic tokens so any Chakra component can reference
// them — e.g. `<Box bg="fa.surface.card">` resolves to `var(--fa-surface-card)`
// and themes automatically because styles.css overrides the var's value per
// theme. styles.css therefore stays the single source of truth for colour.
//
// Two deliberate choices keep the existing, pixel-tested UI byte-identical
// while Chakra is layered on top incrementally:
//
//   * `preflight: false` — Chakra does NOT inject its global CSS reset, so it
//     cannot restyle existing elements (margins, box-sizing, headings, …).
//   * `globalCss: {}` — Chakra's default `html { color: fg; bg: bg }` /
//     `* { font-feature-settings }` global rules are removed, so the provider
//     only emits CSS-variable definitions (in `@layer tokens`) and the layer
//     declaration. Cascade layers rank below unlayered author styles, so even
//     those variable layers cannot win over styles.css.
//
// The net effect of mounting <ChakraProvider> with this system is that the
// rendered DOM and computed styles are unchanged until a component is actually
// converted to a Chakra primitive — which is exactly what the incremental
// migration needs to stay green at every step.

import { createSystem, defaultConfig } from "@chakra-ui/react";

// Wrap a raw `--fa-*` custom property as a Chakra semantic-token value. The
// value is the CSS var itself (not a copied colour literal), so the light/dark
// override authored in styles.css flows through without duplication here.
const faVar = (token) => ({ value: `var(${token})` });

// Mirror of the --fa-* tokens in src/web/styles.css. Grouped by role to read
// naturally as Chakra token paths (fa.surface.card, fa.control.hoverBg, …).
const faColors = {
  fa: {
    surface: {
      card: faVar("--fa-surface-card"),
      raised: faVar("--fa-surface-raised"),
    },
    border: {
      subtle: faVar("--fa-border-subtle"),
      control: faVar("--fa-border-control"),
    },
    text: {
      body: faVar("--fa-text-body"),
      muted: faVar("--fa-text-muted"),
      strong: faVar("--fa-text-strong"),
      modeStatus: faVar("--fa-mode-status-text"),
    },
    accent: {
      link: faVar("--fa-accent-link"),
      solidBg: faVar("--fa-accent-solid-bg"),
      solidBorder: faVar("--fa-accent-solid-border"),
      solidText: faVar("--fa-accent-solid-text"),
    },
    service: {
      dotIdle: faVar("--fa-service-dot-idle"),
    },
    error: {
      text: faVar("--fa-error-text"),
    },
    sidebarToggle: {
      collapsedBg: faVar("--fa-sidebar-toggle-collapsed-bg"),
    },
    toolMode: {
      bg: faVar("--fa-tool-mode-bg"),
      text: faVar("--fa-tool-mode-text"),
      agentBg: faVar("--fa-tool-mode-agent-bg"),
      agentText: faVar("--fa-tool-mode-agent-text"),
    },
    control: {
      hoverBorder: faVar("--fa-control-hover-border"),
      hoverBg: faVar("--fa-control-hover-bg"),
      activeHoverBorder: faVar("--fa-control-active-hover-border"),
      activeHoverBg: faVar("--fa-control-active-hover-bg"),
    },
    focusRing: faVar("--fa-focus-ring"),
  },
};

// Build the system from Chakra's defaultConfig (so component recipes/tokens
// remain available) but with the reset and global body styling neutralised and
// the --fa-* bridge added. createSystem(config) feeds the single config object
// through mergeConfigs, so the explicit `globalCss: {}` and `preflight: false`
// replace the defaults rather than deep-merging the html/body rules back in.
export const system = createSystem({
  ...defaultConfig,
  preflight: false,
  globalCss: {},
  theme: {
    ...defaultConfig.theme,
    semanticTokens: {
      ...defaultConfig.theme.semanticTokens,
      colors: {
        ...(defaultConfig.theme.semanticTokens?.colors ?? {}),
        ...faColors,
      },
    },
  },
});
