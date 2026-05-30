import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createMemorySync, eventIdsFromLino, memoryUrl, pullSince, push } = require(
  "../lib/memory-sync.cjs",
);

const SAMPLE = `demo_memory
  event "a"
    role "user"
    content "Hi"
  event "b"
    role "assistant"
    content "Hi, how may I help you?"
`;

test("eventIdsFromLino reads ids in order from a demo_memory document", () => {
  assert.deepEqual(eventIdsFromLino(SAMPLE), ["a", "b"]);
  assert.deepEqual(eventIdsFromLino(""), []);
});

test("memoryUrl joins base and path without doubling slashes", () => {
  assert.equal(memoryUrl("http://127.0.0.1:9000/", "/v1/memory/since"), "http://127.0.0.1:9000/v1/memory/since");
  assert.equal(memoryUrl("http://127.0.0.1:9000", "/v1/memory/import"), "http://127.0.0.1:9000/v1/memory/import");
});

test("pullSince requests the delta endpoint with the watermark", async () => {
  let requestedUrl = "";
  const fetchImpl = async (url) => {
    requestedUrl = url;
    return { ok: true, status: 200, text: async () => SAMPLE };
  };
  const text = await pullSince("http://local", "a", fetchImpl);
  assert.equal(requestedUrl, "http://local/v1/memory/since?event=a");
  assert.equal(text, SAMPLE);
});

test("push posts Links-Notation text to the import endpoint", async () => {
  let captured = null;
  const fetchImpl = async (url, options) => {
    captured = { url, options };
    return { ok: true, status: 200, json: async () => ({ object: "memory.import", added: 2, total: 2 }) };
  };
  const result = await push("http://local", SAMPLE, fetchImpl);
  assert.equal(captured.url, "http://local/v1/memory/import");
  assert.equal(captured.options.method, "POST");
  assert.equal(captured.options.body, SAMPLE);
  assert.equal(result.added, 2);
});

test("createMemorySync advances the watermark as it pulls deltas", async () => {
  const fetchImpl = async (url) => {
    if (url.includes("/v1/memory/since")) {
      return { ok: true, status: 200, text: async () => SAMPLE };
    }
    return { ok: true, status: 200, json: async () => ({ added: 0, total: 2 }) };
  };
  const sync = createMemorySync({ apiBase: "http://local", fetchImpl });
  assert.equal(sync.getLastSeen(), null);
  const pulled = await sync.pull();
  assert.equal(pulled.added, 2);
  assert.equal(pulled.lastSeen, "b");
  assert.equal(sync.getLastSeen(), "b");
});

test("pull surfaces a non-OK HTTP response as an error", async () => {
  const fetchImpl = async () => ({ ok: false, status: 503, text: async () => "" });
  const sync = createMemorySync({ apiBase: "http://local", fetchImpl });
  await assert.rejects(() => sync.pull(), /memory pull failed: HTTP 503/);
});
