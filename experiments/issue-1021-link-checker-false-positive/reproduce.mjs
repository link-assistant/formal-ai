#!/usr/bin/env node
// Issue #1021 -- the Broken Link Checker reports healthy redirects as broken
// links whenever any link times out.
//
// Twice on this branch (runs 32242196357 and 32454084765) and once on `main`
// (run over d1439e557) the gate went red naming links that are fine. Both
// branch runs produced the same lychee summary:
//
//     | ⏳ Timeouts    | 1     |
//     | 🔀 Redirected  | 18    |
//     | 🚫 Errors      | 0     |
//
// Zero errors, and the job still failed with "lychee found one or more broken
// links", naming six URLs that all answer 200.
//
// The cause is in `extractBrokenUrls`. It narrows its permissive bullet parser
// to the failure section by looking for one hard-coded heading, `## Errors per
// input`. lychee only writes that heading when there is at least one error. A
// report whose only failure is a *timeout* is headed `## Timeouts per input`,
// no match is found, and the function falls back to parsing the whole document
// -- including `## Redirects per input`, where every healthy 301 and 302 lives.
// Those redirect targets are then sent to the Wayback Machine, and whichever
// ones have no snapshot are printed as `::error::Broken link detected:`.
//
// The unit tests did not catch it because all three of them describe reports
// that *have* an errors section, which is the one case where the hard-coded
// heading is found.
//
// The fixture is the real report, lifted verbatim from run 32454084765.
//
// Usage:
//   node experiments/issue-1021-link-checker-false-positive/reproduce.mjs
//
// Recorded result before the fix (2026-08-21):
//   17 URL(s) reported broken, 16 of which lychee had called healthy redirects
// Recorded result after the fix:
//   1 URL(s) reported broken, 0 of which lychee had called healthy redirects
//   (the one is the timeout, which is exactly what the Wayback fallback is for)

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

import { extractBrokenUrls } from "../../scripts/check-web-archive.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const report = readFileSync(path.join(here, "lychee-out-32454084765.md"), "utf-8");

// Everything lychee itself put under `## Redirects per input`, read back out of
// the same report so the two lists cannot drift apart.
const redirectSection = report.slice(report.indexOf("## Redirects per input"));
const redirected = new Set(
  [...redirectSection.matchAll(/^\*\s+(\S+)\s+--\[/gm)].map((m) => m[1]),
);

// One URL is in both lists: `docs.anthropic.com/en/docs/claude-code/cli-usage`
// timed out where one file links it and redirected four hops where another
// does. It is a real failure, so it is not a false positive no matter which
// section also mentions it. Subtracting the failing sections keeps this
// measurement honest instead of scoring the fix by an easier question.
const failing = new Set(
  [...report.matchAll(/^\*\s+\[(?:\d{3}|ERROR|TIMEOUT|UNKNOWN)\]\s+<?(\S+?)>?\s/gim)].map(
    (m) => m[1],
  ),
);
const healthyOnly = new Set([...redirected].filter((url) => !failing.has(url)));

const broken = extractBrokenUrls(report);
const falsePositives = broken.filter((url) => healthyOnly.has(url));

console.log(`lychee classified 0 link(s) as errors and ${failing.size} as timed out`);
console.log(`lychee classified ${healthyOnly.size} further link(s) as healthy redirects`);
console.log(`extractBrokenUrls reported ${broken.length} URL(s) broken`);
console.log(`of those, ${falsePositives.length} are links lychee called healthy redirects:\n`);
for (const url of falsePositives) {
  console.log(`  ${url}`);
}

process.exitCode = falsePositives.length === 0 ? 0 : 1;
