#!/usr/bin/env node
// Strict guard against hardcoded, user-facing natural-language strings in the
// React front-end (`src/web/app.js`).
//
// Issue #511 / #514 context: PR #528 reintroduced English prose directly inside
// `h(...)` render calls (panel titles, button labels, status words). That breaks
// the project's "no hardcoded natural language" rule (docs/design/
// no-hardcoded-natural-language.md): every user-facing string must come from the
// i18n catalog via `t(key, params)` so it translates with the active UI language.
//
// What this guard does
// --------------------
// It parses every `h(tag, props, ...children)` call in src/web/app.js and fails
// when a *child* argument is a bare string literal that reads like prose
// (letters plus whitespace — i.e. a phrase a human would read). Children are the
// 3rd and later arguments; the tag (1st) and props object (2nd) are ignored, so
// className / data-testid / intent / evidence strings never trip the check.
//
// Anything dynamic passes by construction: `t(...)` calls, variables, ternaries,
// and template literals are not bare string literals, so the only way to satisfy
// the guard for visible text is to route it through the catalog.
//
// Allowlist
// ---------
// A few pre-existing desktop status-panel `<dt>` labels predate the i18n catalog
// and are entrenched in other suites / the Rust desktop surface tests. They are
// listed in ALLOWED_LITERALS with a justification so the guard stays green while
// documenting the debt; new prose must NOT be added there.
//
// Usage: node scripts/check-web-hardcoded-ui-strings.mjs
// Exit code 0 = clean, 1 = at least one hardcoded user-facing string found.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "../../..");
const appPath = path.join(repoRoot, "src/web/app.js");
const source = fs.readFileSync(appPath, "utf8");

// Pre-existing, test-entrenched desktop status-panel labels rendered as <dt>
// headers in the desktop sidebar. Single-word labels (Shell, API, Network,
// Memory, Agent) are not prose and never trip the check; only "Tool calls" is a
// multi-word literal child, and it predates the i18n catalog and is coupled to
// the issue-280 / issue-353 specs and the Rust desktop-surface tests. Converting
// it is tracked separately; do NOT extend this list for new strings.
const ALLOWED_LITERALS = new Set(["Tool calls"]);

// A child string literal is "user-facing prose" when it contains a letter
// followed (anywhere) by whitespace and another letter: at least two words a
// human reads. Single tokens ("Off", slugs, symbols) are handled by the
// allowlist or are not prose.
function isProse(value) {
  const trimmed = value.trim();
  if (!trimmed) return false;
  // Two or more words containing letters, separated by whitespace.
  return /[A-Za-zÀ-￿].*\s.*[A-Za-zÀ-￿]/.test(trimmed);
}

// Walk a string literal / template / comment-free scan of one `h(` argument list
// starting at the index of the '(' that follows the `h` identifier. Returns the
// list of top-level argument substrings (trimmed) and the index just past the
// matching ')'. Respects nested (), [], {}, string and template-literal state.
function parseCallArgs(text, openParen) {
  const args = [];
  let depth = 0;
  let i = openParen;
  let argStart = openParen + 1;
  let inString = null; // '"' | "'" | '`'
  let escaped = false;
  for (; i < text.length; i += 1) {
    const ch = text[i];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === inString) {
        inString = null;
      }
      continue;
    }
    if (ch === '"' || ch === "'" || ch === "`") {
      inString = ch;
      continue;
    }
    if (ch === "(" || ch === "[" || ch === "{") {
      depth += 1;
      continue;
    }
    if (ch === ")" || ch === "]" || ch === "}") {
      depth -= 1;
      if (depth === 0) {
        // Closing the h( ... ) call.
        args.push(text.slice(argStart, i));
        return { args, end: i + 1 };
      }
      continue;
    }
    if (ch === "," && depth === 1) {
      args.push(text.slice(argStart, i));
      argStart = i + 1;
    }
  }
  return { args, end: i };
}

// If an argument is exactly a single quoted string literal, return its decoded
// content; otherwise return null (it is dynamic: a call, variable, ternary,
// concatenation, or template literal).
function asStringLiteral(arg) {
  const trimmed = arg.trim();
  if (trimmed.length < 2) return null;
  const quote = trimmed[0];
  if (quote !== '"' && quote !== "'") return null;
  if (trimmed[trimmed.length - 1] !== quote) return null;
  // Ensure the closing quote is the literal's terminator (no concatenation).
  let escaped = false;
  for (let i = 1; i < trimmed.length; i += 1) {
    const ch = trimmed[i];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (ch === "\\") {
      escaped = true;
      continue;
    }
    if (ch === quote) {
      // The quote must be the very last character for a pure literal.
      return i === trimmed.length - 1
        ? trimmed.slice(1, -1).replace(/\\(["'\\])/g, "$1")
        : null;
    }
  }
  return null;
}

// Map a character offset to a 1-based line number for readable diagnostics.
function lineAt(offset) {
  let line = 1;
  for (let i = 0; i < offset && i < source.length; i += 1) {
    if (source[i] === "\n") line += 1;
  }
  return line;
}

const violations = [];
const hCallRe = /\bh\(/g;
let match;
while ((match = hCallRe.exec(source)) !== null) {
  const openParen = match.index + 1; // index of '('
  const { args } = parseCallArgs(source, openParen);
  // children are args[2..]; args[0] is the tag, args[1] is props.
  for (let c = 2; c < args.length; c += 1) {
    const literal = asStringLiteral(args[c]);
    if (literal === null) continue;
    if (!isProse(literal)) continue;
    if (ALLOWED_LITERALS.has(literal.trim())) continue;
    const offset = openParen + (args.slice(0, c).join(",").length + 1);
    violations.push({
      line: lineAt(offset),
      text: literal,
    });
  }
}

if (violations.length > 0) {
  console.error(
    "check-web-hardcoded-ui-strings: found hardcoded user-facing string(s) in src/web/app.js.",
  );
  console.error(
    "Route user-facing text through the i18n catalog via t(\"<key>\", params); see",
  );
  console.error("docs/design/no-hardcoded-natural-language.md and CONTRIBUTING.md.");
  for (const v of violations) {
    console.error(`- app.js:${v.line}: ${JSON.stringify(v.text)}`);
  }
  process.exit(1);
}

console.log(
  `check-web-hardcoded-ui-strings: OK — no hardcoded user-facing strings in h() children of src/web/app.js (allowlist: ${ALLOWED_LITERALS.size} legacy labels).`,
);
