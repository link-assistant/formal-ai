// Issue #557: the `material` skin switches the UI framework from Chakra UI to
// MUI (Material UI). This module builds the MUI theme that bridges the app's
// `--fa-*` CSS design tokens into MUI's palette so Material components inherit
// the exact same colours, corner radii and dark-mode behaviour as the rest of
// the app.
//
// Surfaces/text/divider reference the CSS custom properties directly, so a
// runtime light↔dark switch (which only flips the `--fa-*` values) is picked up
// by MUI components for free. The primary colour is a concrete hex because MUI
// derives light/dark/contrast variants from it and cannot compute on a
// `var(...)` string.

import { createTheme } from "@mui/material/styles";

// Default brand hex — matches --fa-accent-solid-bg (the app's emerald brand).
// Issue #557: the colour-theme selector overrides this per theme by passing a
// resolved `brandHex` into the factory below.
const BRAND = "#1f7a5b";
const BRAND_DARK_BORDER = "#2a8f6a";

// Build a MUI theme for the given resolved colour scheme ("light" | "dark") and
// brand hex. Kept as a factory so the provider can rebuild it when the app
// theme or the selected colour theme flips.
export function createMuiTheme(mode = "light", brandHex = BRAND) {
  const isDark = mode === "dark";
  const brand = brandHex || BRAND;
  return createTheme({
    palette: {
      mode: isDark ? "dark" : "light",
      primary: {
        main: brand,
        contrastText: "#ffffff",
      },
      background: {
        default: "var(--fa-surface-card)",
        paper: "var(--fa-surface-card)",
      },
      text: {
        primary: "var(--fa-text-body)",
        secondary: "var(--fa-text-muted)",
      },
      divider: "var(--fa-border-subtle)",
    },
    shape: {
      borderRadius: 12,
    },
    typography: {
      fontFamily:
        'var(--fa-font-sans, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif)',
    },
    components: {
      // Buttons in the composer are pill-shaped to match the rounded field.
      MuiButtonBase: {
        defaultProps: {
          disableRipple: false,
        },
      },
      MuiIconButton: {
        styleOverrides: {
          root: {
            color: "var(--fa-text-body)",
          },
        },
      },
      MuiButton: {
        styleOverrides: {
          root: {
            borderRadius: 999,
            textTransform: "none",
            fontWeight: 600,
          },
        },
      },
      MuiPaper: {
        styleOverrides: {
          root: {
            backgroundImage: "none",
            borderColor: isDark
              ? brand || BRAND_DARK_BORDER
              : "var(--fa-border-subtle)",
          },
        },
      },
    },
  });
}

export default createMuiTheme;
