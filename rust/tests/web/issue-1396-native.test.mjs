import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { startCli, temporaryHome } from "./support/structured-log-fixture.mjs";

const RUSSIAN_REQUEST = "Как хорошо покакать?";
const RUSSIAN_SLUG = encodeURIComponent("хорошо-покакать");
const WIKIHOW_API = "https://ru.wikihow.com/api.php";
const parseUrl = (title) => `${WIKIHOW_API}?action=parse&page=${title}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
const RUSSIAN_API = parseUrl(RUSSIAN_SLUG);
const RUSSIAN_CAPTURE = readFileSync(new URL("../fixtures/issue-1396/wiki-how-ru.json", import.meta.url), "utf8");

function storeCapture(cache, source, content) {
  const digest = createHash("sha256").update(content).digest("hex");
  // Source capture filenames use the URL's FNV-1a key. The content has its own SHA-256.
  let key = 0xcbf29ce484222325n;
  for (const byte of Buffer.from(source)) {
    key = BigInt.asUintN(64, (key ^ BigInt(byte)) * 0x100000001b3n);
  }
  writeFileSync(path.join(cache, "source-cache", "objects", `${digest}.body`), content);
  writeFileSync(path.join(cache, "source-cache", `${key.toString(16).padStart(16, "0")}.meta`),
    `source-capture-v2\nurl=${source}\nfetched_at=${Math.floor(Date.now() / 1000)}\nsha256=${digest}\n`);
}

test("native localized procedure and follow-up use matching captures over HTTP", {
  skip: !process.env.FORMAL_AI_TEST_NATIVE_CLI,
  timeout: 30_000,
}, async (t) => {
  const cache = temporaryHome(t);
  mkdirSync(path.join(cache, "source-cache", "objects"), { recursive: true });
  storeCapture(cache, RUSSIAN_API, RUSSIAN_CAPTURE);
  const run = await startCli(t, {
    binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
    env: { FORMAL_AI_SOURCE_CACHE_DIR: cache },
    lifetime: 25_000,
  });
  await run.ready();
  async function ask(messages, source = RUSSIAN_API) {
    const response = await fetch(`${run.url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ model: "formal-ai", stream: false, messages }),
    });
    assert.ok(response.ok, await response.clone().text());
    const content = (await response.json()).choices[0].message.content;
    assert.match(content, /1\. Не откладывайте/u);
    assert.ok(content.includes(source), content);
    return content;
  }
  const request = { role: "user", content: RUSSIAN_REQUEST };
  const first = await ask([request]);
  assert.match(first, /## Как хорошо покакать/u);
  await ask([request, { role: "assistant", content: first }, { role: "user", content: "Дай точные шаги" }]);

  const unrelated = JSON.parse(RUSSIAN_CAPTURE);
  unrelated.parse.title = "Домашние растения";
  const title = "хорошо покакать";
  const search = `${WIKIHOW_API}?action=query&list=search&srsearch=${encodeURIComponent(title)}&srlimit=5&format=json&origin=*`;
  const matching = parseUrl(encodeURIComponent(title));
  storeCapture(cache, RUSSIAN_API, JSON.stringify(unrelated));
  storeCapture(cache, search, JSON.stringify({ query: { search: [{ title }] } }));
  storeCapture(cache, matching, RUSSIAN_CAPTURE);
  const recovered = await ask([request], matching);
  assert.ok(!recovered.includes(RUSSIAN_API), recovered);
});
