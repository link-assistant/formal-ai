// R1188-U18: "formalize <url>" and "formalize this page: <url>" are answered
// in chat, in every seeded language, with the statements of the fetched page.
//
// The route is the URL-fetch path: the `page_formalization` meaning of
// data/seed/meanings-web-navigation.lino carries the `http_fetch` role, so the
// fetch recognizer claims the prompt, and the `page_formalization_action`
// role, which turns the fetched page into statements
// (js/worker/formal_ai_worker_page_formalization.js, running the JavaScript
// root's js/agentic/crate/page_formalization.mjs). The page is the Moon page
// of data/benchmarks/web-formalization/en.lino served by a stubbed fetch, so
// no test touches the network. rust/tests/unit/page_formalization_route.rs
// pins the same answers natively through a fixture transport.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { createWorkerContext, evaluate, plain, REPO_ROOT } from "./support/browser-runtime.mjs";

const URL_UNDER_TEST = "https://en.wikipedia.org/wiki/Moon";

/** The paragraphs of the fixture page titled `title`, as the corpus stores them. */
function fixtureParagraphs(title) {
  const lines = readFileSync(path.join(REPO_ROOT, "data/benchmarks/web-formalization/en.lino"), "utf8").split("\n");
  const paragraphs = [];
  for (let index = lines.indexOf(`  title "${title}"`) + 1; index < lines.length && lines[index].startsWith("  "); index += 1) {
    const paragraph = /^ {2}paragraph "(.*)"$/u.exec(lines[index]);
    if (paragraph) paragraphs.push(paragraph[1]);
  }
  return paragraphs;
}

const PAGE = `<html><body><h1>Moon</h1>${fixtureParagraphs("Moon").map((text) => `<p>${text}</p>`).join("")}</body></html>`;

/** A fetch that serves the fixture page for its URL (directly or through the capture component) and 404 otherwise. */
function fixtureFetch(url) {
  const served = String(url) === URL_UNDER_TEST || String(url).includes(encodeURIComponent(URL_UNDER_TEST));
  return Promise.resolve({
    ok: served,
    status: served ? 200 : 404,
    headers: { get: () => (served ? "text/html" : "") },
    text: () => Promise.resolve(served ? PAGE : ""),
  });
}

const worker = createWorkerContext();
const ready = Promise.resolve(evaluate(worker, "loadSeed()")).then(() => {
  worker.fetch = fixtureFetch;
});

async function solve(prompt) {
  await ready;
  return plain(await worker.solve(prompt, [], {}, {}, [], {}));
}

/** The first statement line both runtimes give the Moon page. */
const FIRST_STATEMENT = "- The Moon is the only natural satellite of Earth. → "
  + "(moon wikidata_property_instance_of unknown:only unknown:natural unknown:satellite name:earth)";

/** Prompts in every seeded language and the first line of the answer. */
const SUMMARIES = [
  [
    `formalize this page: ${URL_UNDER_TEST}`,
    `Formalized \`${URL_UNDER_TEST}\`: 25 sentences became 67 statements. 3 sentences are fully formal; `
      + "196 of 423 terms have no meaning yet; 61 of 62 facts survive the round trip back to text.",
    "… and 27 more statements.",
  ],
  [
    `формализуй эту страницу: ${URL_UNDER_TEST}`,
    `Страница \`${URL_UNDER_TEST}\` формализована: из 25 предложений получено 67 утверждений. `
      + "Полностью формальны предложений: 3; терминов без значения: 196 из 423; "
      + "фактов, переживших обратный перевод в текст: 61 из 62.",
    "… и ещё утверждений: 27.",
  ],
  [
    `इस पेज को औपचारिक बनाओ: ${URL_UNDER_TEST}`,
    `\`${URL_UNDER_TEST}\` को औपचारिक बनाया गया: 25 वाक्यों से 67 कथन बने। 3 वाक्य पूरी तरह औपचारिक हैं; `
      + "423 में से 196 पदों का अभी कोई अर्थ नहीं है; 62 में से 61 तथ्य पाठ में वापस बदलने पर बचे रहते हैं।",
    "… और 27 कथन।",
  ],
  [
    `形式化这个网页 ${URL_UNDER_TEST}`,
    `已形式化 \`${URL_UNDER_TEST}\`：25 个句子变为 67 条陈述。3 个句子完全形式化；423 个术语中有 196 个尚无含义；`
      + "62 个事实中有 61 个在转回文本后保留。",
    "……另有 27 条陈述。",
  ],
  [
    `formaliza esta página: ${URL_UNDER_TEST}`,
    `Página \`${URL_UNDER_TEST}\` formalizada: 25 oraciones se convirtieron en 67 enunciados. `
      + "Oraciones totalmente formales: 3; términos aún sin significado: 196 de 423; "
      + "hechos que sobreviven a la vuelta al texto: 61 de 62.",
    "… y 27 enunciados más.",
  ],
];

test("formalize <url> answers the statements of the fetched page in every language", async () => {
  for (const [prompt, summary, more] of SUMMARIES) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "page_formalization", prompt);
    const lines = answer.content.split("\n");
    assert.equal(lines[0], summary, prompt);
    assert.equal(lines[1], "", prompt);
    assert.equal(lines[2], FIRST_STATEMENT, prompt);
    assert.equal(lines.length, 43, prompt);
    assert.equal(lines.at(-1), more, prompt);
    assert.ok(answer.evidence.includes(`page_formalization:request:${URL_UNDER_TEST}`), prompt);
    assert.ok(answer.evidence.includes("response:page_formalization"), prompt);
  }
});

test("the bare form formalizes too", async () => {
  const answer = await solve(`formalize ${URL_UNDER_TEST}`);
  assert.equal(answer.intent, "page_formalization");
  assert.equal(answer.content.split("\n")[2], FIRST_STATEMENT);
});

test("a refused fetch says so and formalizes nothing", async () => {
  const answer = await solve("formalize https://example.org/missing");
  assert.equal(answer.intent, "page_formalization");
  assert.equal(answer.content, "I could not fetch `https://example.org/missing` to formalize it: HTTP 404.");
  assert.ok(answer.evidence.includes("response:page_formalization_unavailable"));
});

test("offline, the answer says nothing was fetched", async () => {
  await ready;
  const navigator = worker.navigator;
  worker.navigator = { ...navigator, onLine: false };
  try {
    const answer = plain(await worker.solve(`formalize ${URL_UNDER_TEST}`, [], {}, {}, [], {}));
    assert.equal(
      answer.content,
      `I cannot formalize \`${URL_UNDER_TEST}\`: the page is not in the source cache and live fetching is off, `
        + "so nothing was fetched.",
    );
  } finally {
    worker.navigator = navigator;
  }
});

test("a formalization request with no URL stays with the formalization handler", async () => {
  const answer = await solve("formalize every man is mortal");
  assert.equal(answer.intent, "formalization");
});

test("a plain fetch still shows the page body", async () => {
  const answer = await solve(`fetch ${URL_UNDER_TEST}`);
  assert.equal(answer.intent, "http_fetch");
  assert.ok(answer.content.includes("The Moon is the only natural satellite of Earth."));
});
