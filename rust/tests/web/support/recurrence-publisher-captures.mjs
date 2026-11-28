import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";

const root = new URL("../../fixtures/coding-discovery/wikifunctions/", import.meta.url);
export const recurrenceFixture = (name) => readFileSync(new URL(name, root), "utf8");
const objects = Object.assign({}, ...readdirSync(root)
  .filter(name => /^fetch-.*-en\.json$/u.test(name))
  .map(name => JSON.parse(recurrenceFixture(name))));

export const recurrenceCases = [
  { language: "en", prompt: "Write a Python function that returns the factorial of n", name: "factorial" },
  { language: "ru", prompt: "Напиши функцию на Python, которая возвращает факториал n", name: "factorial" },
  { language: "hi", prompt: "Python में एक फ़ंक्शन लिखो जो n का फ़ैक्टोरियल लौटाए", name: "factorial" },
  { language: "en", prompt: "Write a Python function that returns the nth Fibonacci number", name: "fibonacci" },
];

export function recurrencePublisherReplay(url) {
  const query = new URL(url);
  const parameters = query.searchParams;
  if (query.hostname === "www.wikifunctions.org") {
    const ids = parameters.get("zids")?.split("|");
    if (ids) {
      return JSON.stringify(Object.fromEntries(ids.filter(id => objects[id]).map(id => [id, objects[id]])));
    }
    const phrase = parameters.get("wikilambdasearch_functions_search")?.toLowerCase();
    if (phrase === "factorial" && parameters.get("wikilambdasearch_functions_language") === "en") {
      return recurrenceFixture("search-factorial-en.json");
    }
    if (phrase === "nth fibonacci number" && parameters.get("wikilambdasearch_functions_language") === "en") {
      return recurrenceFixture("search-nth-fibonacci-number-en.json");
    }
    return recurrenceFixture("search-no-match-en.json");
  }
  const language = query.hostname.match(/^(hi|ru)\.wikipedia\.org$/u)?.[1];
  if (language === "ru") {
    if (parameters.get("srsearch") === "факториал") return recurrenceFixture("bridge-ru-search.json");
    if (parameters.get("titles") === "Факториал") return recurrenceFixture("bridge-ru-langlinks.json");
  }
  if (language === "hi") {
    const subject = parameters.get("srsearch")?.normalize("NFKD").replace(/\p{M}/gu, "");
    if (subject === "फकटरयल") return recurrenceFixture("bridge-hi-search.json");
    if (parameters.get("titles") === "क्रमगुणित") return recurrenceFixture("bridge-hi-langlinks.json");
  }
  return null;
}

export function recurrenceNativeCaptures() {
  const api = "https://www.wikifunctions.org/w/api.php";
  const urls = [];
  for (const phrase of ["factorial", "Factorial", "nth fibonacci number"]) {
    urls.push(api + "?action=query&format=json&formatversion=2&list=wikilambdasearch_functions"
      + "&wikilambdasearch_functions_search=" + encodeURIComponent(phrase)
      + "&wikilambdasearch_functions_language=en&wikilambdasearch_functions_limit=5");
  }
  for (const [language, phrase, title] of [
    ["ru", "факториал", "Факториал"],
    ["hi", "फ़ैक्टोरियल", "क्रमगुणित"],
  ]) {
    const wikipedia = `https://${language}.wikipedia.org/w/api.php?action=query&format=json&formatversion=2`;
    urls.push(wikipedia + "&list=search&srlimit=1&srnamespace=0&srsearch=" + encodeURIComponent(phrase));
    urls.push(wikipedia + "&prop=langlinks&lllang=en&titles=" + encodeURIComponent(title));
  }
  const ids = values => values.filter(id => /^Z[1-9]\d+$/u.test(id)).slice(0, 8);
  const fetchUrl = batch => api + "?action=wikilambda_fetch&format=json&zids=" + batch.join("%7C") + "&language=en";
  for (const zid of ["Z13667", "Z13835"]) {
    const definition = JSON.parse(objects[zid].wikilambda_fetch).Z2K2;
    const implementations = ids(definition.Z8K4.filter(id => id !== "Z14"));
    const testers = ids(definition.Z8K3.filter(id => id !== "Z20"));
    urls.push(fetchUrl([zid]), fetchUrl(implementations), fetchUrl(testers));
    for (const id of implementations) {
      urls.push(fetchUrl([id]));
      const body = objects[id]?.wikilambda_fetch;
      const operators = [...new Set([...String(body ?? "").matchAll(/"Z7K1"\s*:\s*"(Z[1-9]\d+)"/gu)]
        .map(match => match[1]).filter(operator => operator !== zid))].sort();
      if (operators.length) urls.push(fetchUrl(operators));
    }
  }
  return [...new Set(urls)].map(url => {
    const capture = recurrencePublisherReplay(url);
    assert.ok(capture, `Missing test capture for ${url}`);
    return [url, capture];
  });
}
