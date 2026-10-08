// App-wide constants: version stamps, the issue repository and the example
// prompts in the sidebar.

import EXAMPLE_PROMPTS_LINO from "./example-prompts.lino" with { type: "text" };

// The meta tag is stamped with the published crate version by
// `scripts/stamp-pages-artifact.sh` during the GitHub Pages deploy. When the
// site is served straight from the source tree (e.g. local Playwright runs)
// the placeholder is preserved verbatim; we fall back to `"dev"` so issue
// reports never advertise a hardcoded stale version like `0.16.0`.
export const APP_VERSION = (() => {
  const raw = document.querySelector('meta[name="formal-ai-version"]')?.content;
  if (!raw || raw.startsWith("__") || raw.endsWith("__")) {
    return "dev";
  }
  return raw;
})();

const ASSET_VERSION =
  typeof window !== "undefined" ? window.FORMAL_AI_ASSET_VERSION || "" : "";

export const ISSUE_REPOSITORY = "link-assistant/formal-ai";

export const ISSUE_LABELS = "bug";

export const SOURCE_CODE_URL = `https://github.com/${ISSUE_REPOSITORY}`;

// Issue #27: the sidebar's example prompts, read from js/app/example-prompts.lino
// (R1188-U1: data, not code). Each line is `example "<label>" "<text>"`.
export const EXAMPLE_PROMPTS = [
  ...EXAMPLE_PROMPTS_LINO.matchAll(/^\s*example\s+"((?:[^"\\]|\\.)*)"\s+"((?:[^"\\]|\\.)*)"\s*$/gmu),
].map(([, label, text]) => ({ label: JSON.parse(`"${label}"`), text: JSON.parse(`"${text}"`) }));

export function withAssetVersion(path) {
  if (!ASSET_VERSION) {
    return path;
  }
  const separator = path.includes("?") ? "&" : "?";
  return `${path}${separator}v=${encodeURIComponent(ASSET_VERSION)}`;
}
