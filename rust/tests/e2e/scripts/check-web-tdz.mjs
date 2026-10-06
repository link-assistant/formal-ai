#!/usr/bin/env node
// Static guard against a temporal-dead-zone (TDZ) class of bug in the React
// front-end (`js/app.js`).
//
// React evaluates a hook's dependency array *during render*, top-to-bottom.
// When an effect's dep array references a `const foo = useCallback(...)` /
// `useMemo(...)` that is declared *later* in the same component, the reference
// hits the binding's temporal dead zone and throws
// `ReferenceError: Cannot access 'foo' before initialization`, crashing the
// whole component before it can mount. `node --check` and the bundlers only
// validate syntax, so this never surfaces without actually executing the app.
//
// This guard parses each top-level component or hook (a column-0
// `function Name(...)`, optionally `export`ed) in every JSX module and fails if
// any hook dependency array references a `useCallback`/`useMemo` const that is
// declared below the array within the same component.
//
// Usage: node scripts/check-web-tdz.mjs
// Exit code 0 = clean, 1 = at least one ordering violation found.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
// Issue #550: the front-end source is authored as JSX modules under js/app/
// (entry: main.jsx) and bundled by bun into the served js/app.js. The
// hand-written h(...) render calls this guard parses live in the JSX source,
// not the minified bundle.
const appDir = path.resolve(here, "../../../../js/app");
const appFiles = fs
  .readdirSync(appDir)
  .filter((name) => name.endsWith(".jsx"))
  .sort();

// A hook dependency array is the `}, [ ... ])` tail of useEffect / useMemo /
// useCallback / useLayoutEffect. We only need the bracketed identifier list.
const depArrayRe = /\}\s*,\s*\[([^\]]*)\]\s*\)/;
// `const NAME = useCallback(` / `= useMemo(` declarations.
const memoDeclRe = /^\s*const\s+([A-Za-z0-9_]+)\s*=\s*(?:useCallback|useMemo)\b/;

const problems = [];
let memoTotal = 0;

function scanFile(appFile) {
  const source = fs.readFileSync(path.join(appDir, appFile), "utf8");
  const lines = source.split("\n");

  // Component boundaries: a `function Name(` (optionally exported) starting at
  // column 0. Each component runs until the next one (or end of file).
  const componentStarts = [];
  lines.forEach((line, index) => {
    if (/^(?:export\s+)?(?:async\s+)?function\s+[A-Za-z0-9_]+\s*\(/.test(line)) {
      componentStarts.push(index);
    }
  });
  memoTotal += componentStarts.length;

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
            `${appFile}:${i + 1}: hook dependency '${dep}' is used before its ` +
              `useCallback/useMemo declaration at ${appFile}:${declLine.get(dep) + 1} ` +
              "(temporal dead zone — would crash the component on render)",
          );
        }
      }
    }
  }
}

appFiles.forEach(scanFile);

if (problems.length > 0) {
  console.error(
    `check-web-tdz: found ${problems.length} TDZ ordering violation(s) in js/app/*.jsx:`,
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
  `check-web-tdz: OK — scanned ${memoTotal} component(s) in js/app/*.jsx, ` +
    "no hook dependency references a useCallback/useMemo const declared later.",
);
