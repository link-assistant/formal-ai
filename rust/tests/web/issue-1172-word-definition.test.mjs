// Issue #1172 R1172-5 browser twin: "What does X mean?" resolves through the
// registry's dictionary source (Wiktionary) before any encyclopedia or search
// lane. The worker's fetch serves the committed captures of
// rust/tests/fixtures/issue-1172-definitions and refuses everything else, so
// the test never reaches the network. Native pins:
// rust/tests/unit/issue_1172_word_definition.rs.

import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const CACHE_DIR = path.join(REPO_ROOT, "rust/tests/fixtures/issue-1172-definitions/source-cache");

function captures() {
  const out = new Map();
  for (const name of readdirSync(CACHE_DIR).filter((entry) => entry.endsWith(".meta"))) {
    const meta = readFileSync(path.join(CACHE_DIR, name), "utf8");
    const field = (key) => (meta.split("\n").find((line) => line.startsWith(`${key}=`)) || "").slice(key.length + 1);
    out.set(field("url"), readFileSync(path.join(CACHE_DIR, "objects", `${field("sha256")}.body`), "utf8"));
  }
  return out;
}

const bodies = captures();
const worker = createWorkerContext({
  fetch: (url) => {
    const target = String(url);
    const parsed = new URL(target, "http://localhost/");
    const relative = parsed.pathname.replace(/^\/+/, "");
    if (parsed.origin === "http://localhost" || !target.startsWith("http")) {
      const onDisk = relative.startsWith("seed/")
        ? path.join(REPO_ROOT, "data", relative)
        : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(text) });
      } catch {
        return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
      }
    }
    const body = bodies.get(target);
    return Promise.resolve(body
      ? { ok: true, status: 200, text: () => Promise.resolve(body) }
      : { ok: false, status: 404, text: () => Promise.resolve("") });
  },
});
await evaluate(worker, "loadSeed()");

test("R1172-5: the English definition is read from the Wiktionary capture", async () => {
  const answer = await worker.solve("What does ephemeral mean?", [], {}, {}, [], {});
  assert.equal(answer.intent, "concept_lookup", JSON.stringify(answer));
  assert.equal(answer.content, "ephemeral, as Wiktionary defines it:\n  1. (noun) Something which lasts for a short period of time.\n  2. (adjective) Lasting for a short period of time.\n  3. (adjective) Existing for only one day, as with some flowers, insects, and diseases.\nSource: https://api.dictionaryapi.dev/api/v2/entries/en/ephemeral (sha256 1069f27b1e20a804; CC BY-SA 3.0)");
  assert.ok(answer.evidence.includes("word_definition:source:wiktionary"), JSON.stringify(answer.evidence));
});

test("R1172-5: the Russian definition is read from the Wiktionary capture", async () => {
  const answer = await worker.solve("Что означает эфемерный?", [], {}, {}, [], {});
  assert.equal(answer.intent, "concept_lookup", JSON.stringify(answer));
  assert.equal(answer.content, "эфемерный — значения по словарю Wiktionary:\n  1. книжн. скоропреходящий, непрочный, мимолётный, временный\n  2. книжн. мнимый, воображаемый, призрачный\nИсточник: https://ru.wiktionary.org/w/api.php?action=query&format=json&prop=extracts&explaintext=1&titles=%D1%8D%D1%84%D0%B5%D0%BC%D0%B5%D1%80%D0%BD%D1%8B%D0%B9 (sha256 79a720af10980ed5; CC BY-SA 3.0)");
  assert.ok(answer.evidence.includes("word_definition:source:wiktionary"), JSON.stringify(answer.evidence));
});

test("R1172-5: the Hindi definition is read from the Wiktionary capture", async () => {
  const answer = await worker.solve("क्षणिक का क्या अर्थ है?", [], {}, {}, [], {});
  assert.equal(answer.intent, "concept_lookup", JSON.stringify(answer));
  assert.equal(answer.content, "क्षणिक — Wiktionary के अनुसार अर्थ:\n  1. क्षणिक ^१ वि॰ [सं॰] एक क्षण रहनेवाला । क्षणभंगुर । अनित्य ।\n  2. क्षणिक ^२ संज्ञा पुं॰ [सं॰] दे॰ 'क्षणिकवाद' ।\nस्रोत: https://hi.wiktionary.org/w/api.php?action=query&format=json&prop=extracts&explaintext=1&titles=%E0%A4%95%E0%A5%8D%E0%A4%B7%E0%A4%A3%E0%A4%BF%E0%A4%95 (sha256 fbf4884f705fb060; CC BY-SA 3.0)");
  assert.ok(answer.evidence.includes("word_definition:source:wiktionary"), JSON.stringify(answer.evidence));
});

test("R1172-5: the Spanish definition is read from the Wiktionary capture", async () => {
  const answer = await worker.solve("¿Qué significa efímero?", [], {}, {}, [], {});
  assert.equal(answer.intent, "concept_lookup", JSON.stringify(answer));
  assert.equal(answer.content, "efímero, según Wiktionary:\n  1. Que dura un solo día.\n  2. Que dura poco tiempo.\n  3. Que comienza y acaba rápido, de forma fugaz.\nFuente: https://es.wiktionary.org/w/api.php?action=query&format=json&prop=extracts&explaintext=1&titles=ef%C3%ADmero (sha256 aa9f241595cf51db; CC BY-SA 3.0)");
  assert.ok(answer.evidence.includes("word_definition:source:wiktionary"), JSON.stringify(answer.evidence));
});

test("R1172-5: a Chinese page without senses and an uncaptured word fall through", async () => {
  worker.__prompt = "苹果是什么意思？";
  assert.equal(JSON.stringify(await evaluate(worker, "wordDefinitionTerm(__prompt)")), JSON.stringify({ term: "苹果", language: "zh" }));
  assert.equal(await evaluate(worker, "tryWordDefinition(__prompt, {})"), null);
  worker.__prompt = "What does banana mean?";
  assert.equal(await evaluate(worker, "tryWordDefinition(__prompt, {})"), null);
  worker.__prompt = "What is the capital of France?";
  assert.equal(await evaluate(worker, "wordDefinitionTerm(__prompt)"), null);
});
