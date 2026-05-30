#!/usr/bin/env node
// Static guard against a temporal-dead-zone (TDZ) class of bug in the React
// front-end (`src/web/app.js`).
//
// React evaluates a hook's dependency array *during render*, top-to-bottom.
// When an effect's dep array references a `const foo = useCallback(...)` /
// `useMemo(...)` that is declared *later* in the same component, the reference
// hits the binding's temporal dead zone and throws
// `ReferenceError: Cannot access 'foo' before initialization`, crashing the
// whole component before it can mount. `node --check` and the bundlers only
// validate syntax, so this never surfaces without actually executing the app.
//
// This guard parses each top-level component (a column-0 `function Name(...)`)
// and fails if any hook dependency array references a `useCallback`/`useMemo`
// const that is declared below the array within the same component.
//
// Usage: node scripts/check-web-tdz.mjs
// Exit code 0 = clean, 1 = at least one ordering violation found.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appPath = path.resolve(here, "../../../src/web/app.js");

const source = fs.readFileSync(appPath, "utf8");
const lines = source.split("\n");

// Component boundaries: a `function Name(` starting at column 0. Each component
// runs until the next column-0 `function ` (or end of file).
const componentStarts = [];
lines.forEach((line, index) => {
  if (/^function\s+[A-Za-z0-9_]+\s*\(/.test(line)) {
    componentStarts.push(index);
  }
});

// A hook dependency array is the `}, [ ... ])` tail of useEffect / useMemo /
// useCallback / useLayoutEffect. We only need the bracketed identifier list.
const depArrayRe = /\}\s*,\s*\[([^\]]*)\]\s*\)/;
// `const NAME = useCallback(` / `= useMemo(` declarations.
const memoDeclRe = /^\s*const\s+([A-Za-z0-9_]+)\s*=\s*(?:useCallback|useMemo)\b/;

const problems = [];

for (let c = 0; c < componentStarts.length; c += 1) {
  const start = componentStarts[c];
  const end = c + 1 < componentStarts.length ? componentStarts[c + 1] : lines.length;

  // Map every memoised const in this component to its declaration line.
  const declLine = new Map();
  for (let i = start; i < end; i += 1) {
    const m = lines[i].match(memoDeclRe);
    if (m) {
      declLine.set(m[1], i);
    }
  }

  // Flag dep arrays that reference a memoised const declared further down.
  for (let i = start; i < end; i += 1) {
    const m = lines[i].match(depArrayRe);
    if (!m) {
      continue;
    }
    const deps = m[1]
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    for (const dep of deps) {
      if (declLine.has(dep) && declLine.get(dep) > i) {
        problems.push(
          `app.js:${i + 1}: hook dependency '${dep}' is used before its ` +
            `useCallback/useMemo declaration at app.js:${declLine.get(dep) + 1} ` +
            "(temporal dead zone — would crash the component on render)",
        );
      }
    }
  }
}

const memoTotal = componentStarts.length;
if (problems.length > 0) {
  console.error(
    `check-web-tdz: found ${problems.length} TDZ ordering violation(s) in src/web/app.js:`,
  );
  for (const problem of problems) {
    console.error(`  - ${problem}`);
  }
  console.error(
    "\nFix: move the `const ... = useCallback/useMemo` declaration above every " +
      "hook whose dependency array references it.",
  );
  process.exit(1);
}

console.log(
  `check-web-tdz: OK — scanned ${memoTotal} component(s) in src/web/app.js, ` +
    "no hook dependency references a useCallback/useMemo const declared later.",
);
