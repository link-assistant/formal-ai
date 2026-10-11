// R1188-U19: where the chat translation route picks among several surfaces,
// the surface that survives the round trip source -> meta -> target -> meta ->
// source wins (js/agentic/crate/round_trip_translation.mjs `roundTripChoice`,
// read by the worker's `roundTripSurface`). The page Wiktionary serves is a
// stubbed fetch, so no test touches the network. The Rust twin is the
// `Translation::round_trip_surface` pin of
// rust/tests/unit/issue_1188_round_trip_translation.rs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

/** The wikitext of each Wiktionary page the stub serves, by page title. */
const PAGES = {
  fix: "{{trans-top|to repair}}\n* Russian: {{t|ru|добавь}}, {{t+|ru|исправить}}\n{{trans-bottom}}",
  blarg: "{{trans-top|nonsense}}\n* Russian: {{t|ru|бларг}}, {{t|ru|блорг}}\n{{trans-bottom}}",
};

/** A fetch that answers Wiktionary's parse API from `PAGES` and nothing else. */
function wiktionaryFetch(url) {
  const page = new URL(String(url)).searchParams.get("page");
  const wikitext = Object.hasOwn(PAGES, page ?? "") ? PAGES[page] : null;
  return Promise.resolve({
    ok: wikitext !== null,
    status: wikitext === null ? 404 : 200,
    json: () => Promise.resolve({ parse: { wikitext: { "*": wikitext } } }),
  });
}

const worker = createWorkerContext();
const ready = Promise.resolve(evaluate(worker, "loadSeed()")).then(() => {
  worker.fetch = wiktionaryFetch;
});

async function solve(prompt) {
  await ready;
  return plain(await worker.solve(prompt, [], {}, {}, [], {}));
}

test("the round trip chooses among the surfaces Wiktionary offers", async () => {
  await ready;
  assert.deepEqual(
    plain(evaluate(worker, 'roundTripSurface("fix", "en", "ru", ["добавь", "исправить"])')),
    "исправить",
  );
  assert.equal(evaluate(worker, 'roundTripSurface("blarg", "en", "ru", ["бларг", "блорг"])'), "бларг");
  assert.equal(evaluate(worker, 'roundTripSurface("fix", "en", "ru", [])'), null);
});

test("the chat translation answers with the surface whose meaning comes back", async () => {
  const answer = await solve("translate 'fix' to Russian");
  assert.equal(answer.intent, "translate_en_to_ru");
  assert.equal(answer.content, '"исправить"');
});

test("where the round trip cannot tell the surfaces apart, the first one stands", async () => {
  const answer = await solve("translate 'blarg' to Russian");
  assert.equal(answer.content, '"бларг"');
});

test("a stock phrase keeps its first form", async () => {
  const answer = await solve("translate 'Hello' to Russian");
  assert.equal(answer.content, '"Привет"');
});
