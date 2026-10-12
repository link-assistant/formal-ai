// Issue #1178 JS parity (PR #1188, JavaScript-first): brainstorming, creative
// writing, planning and advice compose in the browser worker exactly as the
// native handlers do, for the prompts rust/tests/unit/web-engine-core/issue_1178_creative_composition.rs
// pins, and the four precedence rows bind worker functions.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

async function answerOf(handler, prompt) {
  await seeded;
  const literal = JSON.stringify(prompt);
  return plain(evaluate(worker, `${handler}(${literal}, normalizePrompt(${literal}))`));
}

async function bodyOf(handler, prompt) {
  const answer = await answerOf(handler, prompt);
  assert.ok(answer, `${handler} should answer: ${prompt}`);
  return answer.content;
}

function poemOf(answer) {
  return answer.split("Constraint check")[0].split("\n").map((line) => line.trim()).filter(Boolean);
}

function endWord(line) {
  const tokens = line.split(/\s+/);
  return tokens[tokens.length - 1].toLowerCase();
}

test("the four precedence rows bind worker functions", async () => {
  await seeded;
  const registry = plain(evaluate(worker, "WORKER_HANDLER_REGISTRY.workerHandlers"));
  const bound = {
    brainstorm_composition: "tryBrainstormComposition",
    advice_request: "tryAdviceRequest",
    creative_writing: "tryCreativeWritingRequest",
    planning_request: "tryPlanningRequest",
  };
  for (const [slug, name] of Object.entries(bound)) {
    assert.equal(registry[slug], name, slug);
    assert.equal(evaluate(worker, `typeof ${name}`), "function", name);
  }
});

test("R1: brainstorming composes from the formalized topic", async () => {
  const answer = await bodyOf("tryBrainstormComposition", "five name ideas for a coffee shop near a university");
  assert.ok(answer.includes("Coffee"), answer);
  assert.ok(answer.includes("coffee") && answer.includes("university"), answer);
  assert.ok(!answer.includes("Web search requested for"), answer);
  assert.ok(!answer.includes("TraceLint"), answer);
  assert.ok(!answer.includes("Links Notation notebook"), answer);
});

test("R2: brainstorming states its metric and constraints", async () => {
  const answer = await bodyOf("tryBrainstormComposition", "five short name ideas for a coffee shop without cafe in them");
  assert.ok(answer.includes("levenshtein"), answer);
  assert.ok(answer.includes("12 characters"), answer);
  assert.ok(answer.includes("excluding"), answer);
  assert.ok(!answer.includes("Cafe"), answer);
  for (const line of answer.split("\n")) {
    const match = /^\d+\. (.*)$/.exec(line);
    if (match) assert.ok(Array.from(match[1]).length <= 12, `${match[1]} exceeds the cap`);
  }
});

test("brainstorming honors the requested count", async () => {
  const answer = await bodyOf("tryBrainstormComposition", "three name ideas for a bakery");
  assert.ok(answer.includes("3 name candidates"), answer);
});

test("brainstorming multilingual probes", async () => {
  const probes = [
    ["ru", "придумай пять идей для кофейной рядом с университетом", "кофейной"],
    ["hi", "विश्वविद्यालय के पास कॉफी की दुकान के लिए नाम के विचार दो", "कॉफी"],
    ["zh", "给大学附近的咖啡店起个名字", "咖啡店"],
  ];
  for (const [language, prompt, concept] of probes) {
    const answer = await bodyOf("tryBrainstormComposition", prompt);
    assert.ok(answer.includes("1. "), `${language}: ${answer}`);
    assert.ok(answer.includes(concept), `${language}: ${answer}`);
    assert.ok(!answer.includes("Web search requested for"), `${language}: ${answer}`);
  }
});

test("R3: a four-line poem about the sea passes its constraint check", async () => {
  const answer = await bodyOf("tryCreativeWritingRequest", "Write a four-line poem about the sea");
  const poem = poemOf(answer);
  assert.equal(poem.length, 4, answer);
  assert.ok(poem.some((line) => line.toLowerCase().includes("sea")), answer);
  assert.ok(answer.includes("rhyme scheme abcb"), answer);
  assert.ok(answer.includes("holds"), answer);
  assert.ok(!answer.includes("Web search requested for"), answer);
  const nightClass = ["night", "light", "bright", "white", "sight", "flight"];
  assert.ok(nightClass.includes(endWord(poem[1])) && nightClass.includes(endWord(poem[3])), answer);
});

test("R3: a violating poem is never returned", async () => {
  const answer = await bodyOf("tryCreativeWritingRequest", "Write a rhyming sonnet about the star");
  assert.ok(answer.includes("never returned"), answer);
  assert.ok(!answer.includes("Constraint check, run on the rendered poem"), answer);
});

test("a haiku carries three lines and states its meter unverified", async () => {
  const answer = await bodyOf("tryCreativeWritingRequest", "Write a haiku about the moon");
  assert.equal(poemOf(answer).length, 3, answer);
  assert.ok(answer.includes("not machine-verified"), answer);
});

test("R4: the non-English rhyme gap is stated, not guessed", async () => {
  const prompts = [
    ["ru", "напиши стихотворение о море в четыре строки"],
    ["hi", "चार पंक्तियों में समुद्र पर कविता लिखो"],
    ["zh", "写一首关于海的四行诗"],
  ];
  for (const [language, prompt] of prompts) {
    const answer = await bodyOf("tryCreativeWritingRequest", prompt);
    assert.equal(poemOf(answer).length, 4, `${language}: ${answer}`);
    assert.ok(answer.includes("gap"), `${language}: ${answer}`);
    assert.ok(!answer.includes("rhyme scheme") || answer.includes("none"), `${language}: ${answer}`);
  }
});

test("R5: the Rome itinerary is cited and feasible", async () => {
  const answer = await bodyOf("tryPlanningRequest", "3-day itinerary for Rome");
  assert.ok(answer.includes("Day 1") && answer.includes("Day 3"), answer);
  assert.ok(answer.includes("Colosseum"), answer);
  assert.ok(answer.split("wikivoyage.org").length - 1 >= 5, answer);
  assert.ok(answer.includes("no overlapping items"), answer);
  assert.ok(answer.includes("travel time accounted"), answer);
  assert.ok(!answer.includes("terminal"), answer);
  assert.ok(!answer.includes("Web search requested for"), answer);
});

test("R5: every scheduled window sits inside the day and in order", async () => {
  const answer = await bodyOf("tryPlanningRequest", "3-day itinerary for Rome");
  let previousEnd = 0;
  for (const line of answer.split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("Day ")) {
      previousEnd = 0;
      continue;
    }
    const match = /^(\d\d):(\d\d)-(\d\d):(\d\d) /.exec(trimmed);
    if (!match) continue;
    const start = Number(match[1]) * 60 + Number(match[2]);
    const end = Number(match[3]) * 60 + Number(match[4]);
    assert.ok(start >= 9 * 60, trimmed);
    assert.ok(end <= 19 * 60, trimmed);
    assert.ok(start >= previousEnd, trimmed);
    previousEnd = end;
  }
});

test("planning refuses an uncached destination honestly", async () => {
  const answer = await bodyOf("tryPlanningRequest", "plan a trip to Atlantis");
  assert.ok(answer.includes("no places in the seeded cache"), answer);
  assert.ok(!answer.includes("Web search requested for"), answer);
});

test("planning multilingual probes", async () => {
  for (const [language, prompt] of [["ru", "составь маршрут по Риму на 3 дня"], ["zh", "罗马三天行程"]]) {
    const answer = await bodyOf("tryPlanningRequest", prompt);
    assert.ok(answer.includes("Day 1"), `${language}: ${answer}`);
    assert.ok(answer.includes("3-day"), `${language}: ${answer}`);
    assert.ok(answer.includes("wikivoyage.org"), `${language}: ${answer}`);
    assert.ok(!answer.includes("terminal"), `${language}: ${answer}`);
  }
});

test("R6: advice is cited, graded and weighted", async () => {
  const answer = await bodyOf("tryAdviceRequest", "evidence-based tips for trouble sleeping");
  assert.ok(answer.includes("CDC"), answer);
  assert.ok(answer.includes("cdc.gov"), answer);
  assert.ok(answer.includes("strong (health-authority guideline)"), answer);
  assert.ok(answer.includes("issue #1179"), answer);
  assert.ok(answer.includes("authority_guideline 0.75 > primary_study 0.60"), answer);
  assert.ok(answer.indexOf("moderate (individual primary study)") > answer.lastIndexOf("strong (health-authority guideline)"), answer);
  assert.ok(!answer.includes("Web search requested for"), answer);
});

test("advice multilingual probes", async () => {
  for (const prompt of ["дай советы для борьбы с бессонницей", "नींद के लिए सलाह दो", "改善睡眠的建议"]) {
    const answer = await bodyOf("tryAdviceRequest", prompt);
    assert.ok(answer.includes("CDC") && answer.includes("cdc.gov"), `${prompt}: ${answer}`);
  }
});

test("advice refuses an unformalized topic honestly", async () => {
  const answer = await bodyOf("tryAdviceRequest", "advice for choosing a hoverboard");
  assert.ok(answer.includes("no formalized recommendations"), answer);
});

test("R8: unrelated prompts are not claimed", async () => {
  const unrelated = [
    "Hello, how are you today?",
    "What is the capital of France?",
    "Convert this JSON to YAML:\n```json\n{\"a\": 1}\n```",
  ];
  for (const prompt of unrelated) {
    for (const handler of ["tryBrainstormComposition", "tryCreativeWritingRequest", "tryPlanningRequest", "tryAdviceRequest"]) {
      assert.equal(await answerOf(handler, prompt), null, `${handler} must not claim: ${prompt}`);
    }
  }
});

test("the full worker routes the four classes to their composition handlers", async () => {
  await seeded;
  const routed = [
    ["five name ideas for a coffee shop near a university", "brainstorming", "name candidates"],
    ["Write a four-line poem about the sea", "creative_writing", "Constraint check"],
    ["3-day itinerary for Rome", "planning", "Day 1"],
    ["evidence-based tips for trouble sleeping", "advice", "cdc.gov"],
  ];
  for (const [prompt, intent, marker] of routed) {
    const answer = await worker.solve(prompt, [], {}, {}, [], {});
    assert.equal(answer.intent, intent, `${prompt}: ${answer.intent} ${answer.content}`);
    assert.ok(answer.content.includes(marker), answer.content);
  }
});
