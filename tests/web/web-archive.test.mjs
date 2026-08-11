import assert from "node:assert/strict";
import { test } from "node:test";

import { extractBrokenUrls } from "../../scripts/check-web-archive.mjs";

test("the archive fallback ignores successful redirects in Lychee Markdown", () => {
  const report = `# Summary

## Errors per input

### Errors in docs/reference.md

* [502] <https://broken.example/reference> (at 65:1) | Rejected status code: 502

## Redirects per input

### Redirects in README.md

* https://working.example/old --[301]--> https://working.example/current
`;

  assert.deepEqual(extractBrokenUrls(report), ["https://broken.example/reference"]);
});

test("the archive fallback still accepts Lychee output without section headings", () => {
  const report = `
* [404] https://broken.example/missing | Rejected status code: 404
* [ERROR] https://broken.example/network | Network failure
`;

  assert.deepEqual(extractBrokenUrls(report), [
    "https://broken.example/missing",
    "https://broken.example/network",
  ]);
});
