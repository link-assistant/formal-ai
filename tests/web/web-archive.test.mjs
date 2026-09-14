import assert from "node:assert/strict";
import { test } from "node:test";

import {
  checkWaybackMachine,
  extractBrokenUrls,
} from "../../scripts/check-web-archive.mjs";

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

// The Wayback Machine answers three different things, and only one of them is
// a statement about the link. Collapsing "the archive is down" into "the link
// is unarchived" once failed a pull request during an archive.org outage over
// a link that resolved with HTTP 200.

const jsonResponse = (body) => ({
  ok: true,
  status: 200,
  json: async () => body,
});

test("a snapshot in the archive is reported as archived", async () => {
  const result = await checkWaybackMachine(
    "https://example.com/gone",
    async () =>
      jsonResponse({
        archived_snapshots: {
          closest: {
            available: true,
            url: "http://web.archive.org/web/20240101000000/https://example.com/gone",
            timestamp: "20240101000000",
          },
        },
      })
  );

  assert.equal(result.status, "archived");
  assert.equal(result.timestamp, "20240101000000");
  assert.ok(
    result.archiveUrl.startsWith("https://"),
    "the archive URL is upgraded to https"
  );
});

test("an empty snapshot set is reported as unarchived", async () => {
  const result = await checkWaybackMachine("https://example.com/gone", async () =>
    jsonResponse({ archived_snapshots: {} })
  );

  assert.equal(result.status, "unarchived");
  assert.equal(result.archiveUrl, null);
});

test("a Wayback outage is undetermined rather than unarchived", async () => {
  for (const status of [429, 500, 502, 503]) {
    const result = await checkWaybackMachine(
      "https://example.com/fine",
      async () => ({ ok: false, status, json: async () => ({}) })
    );

    assert.equal(
      result.status,
      "unknown",
      `HTTP ${status} from archive.org describes archive.org, not the link`
    );
    assert.match(result.reason, new RegExp(String(status)));
  }
});

test("an unreachable Wayback API is undetermined rather than unarchived", async () => {
  const result = await checkWaybackMachine("https://example.com/fine", async () => {
    throw new Error("getaddrinfo ENOTFOUND archive.org");
  });

  assert.equal(result.status, "unknown");
  assert.match(result.reason, /unreachable/);
});

test("a timed-out Wayback request is undetermined rather than unarchived", async () => {
  const result = await checkWaybackMachine("https://example.com/fine", async () => {
    const error = new Error("aborted");
    error.name = "AbortError";
    throw error;
  });

  assert.equal(result.status, "unknown");
  assert.match(result.reason, /timed out/);
});

test("an unreadable Wayback body is undetermined rather than unarchived", async () => {
  const result = await checkWaybackMachine("https://example.com/fine", async () => ({
    ok: true,
    status: 200,
    json: async () => {
      throw new Error("Unexpected token < in JSON");
    },
  }));

  assert.equal(result.status, "unknown");
  assert.match(result.reason, /unreadable/);
});

test("an unexpected Wayback shape is undetermined rather than unarchived", async () => {
  const result = await checkWaybackMachine("https://example.com/fine", async () =>
    jsonResponse({ message: "rate limited" })
  );

  assert.equal(result.status, "unknown");
});
