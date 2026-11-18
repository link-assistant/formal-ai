import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { startCli, temporaryHome } from "./support/structured-log-fixture.mjs";

const RUSSIAN_REQUEST = "Как хорошо покакать?";
const RUSSIAN_SLUG = encodeURIComponent("хорошо-покакать");
const RUSSIAN_API = `https://ru.wikihow.com/api.php?action=parse&page=${RUSSIAN_SLUG}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
const RUSSIAN_CAPTURE = readFileSync(new URL("../fixtures/issue-1396/wiki-how-ru.json", import.meta.url), "utf8");

test("native localized procedure and follow-up reuse the exact capture over HTTP", {
  skip: !process.env.FORMAL_AI_TEST_NATIVE_CLI,
  timeout: 30_000,
}, async (t) => {
  const cache = temporaryHome(t);
  const objects = path.join(cache, "source-cache", "objects");
  mkdirSync(objects, { recursive: true });
  const digest = createHash("sha256").update(RUSSIAN_CAPTURE).digest("hex");
  // Source capture filenames use the URL's FNV-1a key. The content has its own SHA-256.
  let key = 0xcbf29ce484222325n;
  for (const byte of Buffer.from(RUSSIAN_API)) {
    key = BigInt.asUintN(64, (key ^ BigInt(byte)) * 0x100000001b3n);
  }
  writeFileSync(path.join(objects, `${digest}.body`), RUSSIAN_CAPTURE);
  writeFileSync(path.join(cache, "source-cache", `${key.toString(16).padStart(16, "0")}.meta`),
    `source-capture-v2\nurl=${RUSSIAN_API}\nfetched_at=${Math.floor(Date.now() / 1000)}\nsha256=${digest}\n`);
  const run = await startCli(t, {
    binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
    env: { FORMAL_AI_SOURCE_CACHE_DIR: cache },
    lifetime: 25_000,
  });
  await run.ready();
  async function ask(messages) {
    const response = await fetch(`${run.url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ model: "formal-ai", stream: false, messages }),
    });
    assert.ok(response.ok, await response.clone().text());
    const content = (await response.json()).choices[0].message.content;
    assert.match(content, /1\. Не откладывайте/u);
    assert.ok(content.includes(RUSSIAN_API), content);
    return content;
  }
  const request = { role: "user", content: RUSSIAN_REQUEST };
  const first = await ask([request]);
  assert.match(first, /## Как хорошо покакать/u);
  await ask([request, { role: "assistant", content: first }, { role: "user", content: "Дай точные шаги" }]);
});
