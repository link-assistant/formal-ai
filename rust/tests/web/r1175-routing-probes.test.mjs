// Issue #1175 R1175-4: a held-out routing probe corpus over every intent lane.
//
// rust/tests/fixtures/routing-probes/probes-*.lino holds the probes (prompt,
// language, lane, the intent the system SHOULD answer); budget.lino holds the
// per-lane coverage budget and the misroute ratchets. Every probe runs through
// one browser worker. A probe without a `js_misroute` line must answer its
// intent; a probe with one must still misroute (a fixed probe fails until its
// line is removed and `js_misroute_ceiling` lowered), and the number of
// `js_misroute` lines must equal the ceiling, so the count can only fall.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const FIXTURE_DIR = path.join(REPO_ROOT, "rust/tests/fixtures/routing-probes");
const LANGUAGES = new Set(["en", "ru", "hi", "zh"]);

/** Decode a quoted fixture value: a doubled quote is one quote, `\n` a newline. */
function unquote(raw) {
  if (!raw.startsWith('"')) return raw;
  const body = raw.slice(1, -1);
  let out = "";
  for (let i = 0; i < body.length; i += 1) {
    if (body[i] === '"' && body[i + 1] === '"') {
      out += '"';
      i += 1;
    } else if (body[i] === "\\" && body[i + 1] === "n") {
      out += "\n";
      i += 1;
    } else {
      out += body[i];
    }
  }
  return out;
}

/** Read `name value` lines with their indentation depth, skipping comments. */
function fixtureLines(file) {
  return readFileSync(file, "utf8")
    .split("\n")
    .filter((line) => line.trim() !== "" && !line.trimStart().startsWith("#"))
    .map((line) => {
      const depth = (line.length - line.trimStart().length) / 2;
      const trimmed = line.trim();
      const space = trimmed.indexOf(" ");
      return space === -1
        ? { depth, key: trimmed, value: "" }
        : { depth, key: trimmed.slice(0, space), value: unquote(trimmed.slice(space + 1)) };
    });
}

function loadProbes() {
  const probes = [];
  for (const name of readdirSync(FIXTURE_DIR).filter((file) => /^probes-.*\.lino$/u.test(file)).sort()) {
    for (const { depth, key, value } of fixtureLines(path.join(FIXTURE_DIR, name))) {
      if (depth === 0) continue;
      if (depth === 1 && key === "probe") probes.push({ id: value, file: name });
      else if (depth === 2) probes.at(-1)[key] = value;
      else assert.fail(`${name}: unexpected line ${key} ${value}`);
    }
  }
  return probes;
}

function loadBudget() {
  const budget = { extraLanes: new Map(), exemptLanes: new Map() };
  let current = null;
  for (const { depth, key, value } of fixtureLines(path.join(FIXTURE_DIR, "budget.lino"))) {
    if (depth === 1 && (key === "extra_lane" || key === "exempt_lane")) {
      current = key === "extra_lane" ? budget.extraLanes : budget.exemptLanes;
      current.set(value, "");
    } else if (depth === 2 && key === "reason") {
      const lanes = current;
      lanes.set([...lanes.keys()].at(-1), value);
    } else if (depth === 1) {
      budget[key] = Number(value);
    }
  }
  return budget;
}

/** The shared lanes: handler-precedence rows plus intent-routing slugs. */
function seedLanes() {
  const handlers = readFileSync(path.join(REPO_ROOT, "data/seed/handler-precedence.lino"), "utf8");
  const routing = readFileSync(path.join(REPO_ROOT, "data/seed/intent-routing.lino"), "utf8");
  return new Set([
    ...[...handlers.matchAll(/^ {2}handler (\S+)/gmu)].map((match) => match[1]),
    ...[...routing.matchAll(/^ {4}slug (\S+)/gmu)].map((match) => match[1]),
  ]);
}

const probes = loadProbes();
const budget = loadBudget();

const worker = createWorkerContext({
  fetch: async (url) => {
    const target = String(url);
    if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
      const relative = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
      const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return { ok: true, status: 200, text: async () => text };
      } catch {
        return { ok: false, status: 404, text: async () => "" };
      }
    }
    // Offline: every external provider is unreachable.
    return { ok: false, status: 404, text: async () => "" };
  },
});

let answered = null;
/** Run every probe once, in order, on the single worker. */
function answers() {
  answered ??= (async () => {
    await evaluate(worker, "loadSeed()");
    const intents = new Map();
    for (const probe of probes) {
      const answer = plain(await worker.solve(probe.prompt, [], {}, {}, [], {}));
      intents.set(probe.id, String(answer.intent));
    }
    return intents;
  })();
  return answered;
}

test("R1175-4: the corpus is well formed and large enough", () => {
  assert.ok(probes.length >= budget.minimum_probes, `${probes.length} probes < ${budget.minimum_probes}`);
  const lanes = seedLanes();
  const prompts = new Set();
  const ids = new Set();
  for (const probe of probes) {
    for (const field of ["lane", "language", "intent", "prompt"]) {
      assert.ok(probe[field], `${probe.id} lacks ${field}`);
    }
    assert.ok(LANGUAGES.has(probe.language), `${probe.id}: language ${probe.language}`);
    assert.ok(lanes.has(probe.lane) || budget.extraLanes.has(probe.lane), `${probe.id}: unknown lane ${probe.lane}`);
    assert.ok(!prompts.has(probe.prompt), `${probe.id}: duplicate prompt ${JSON.stringify(probe.prompt)}`);
    assert.ok(!ids.has(probe.id), `duplicate probe id ${probe.id}`);
    assert.notEqual(probe.js_misroute, probe.intent, `${probe.id}: a misroute equal to the intent is no misroute`);
    // R1173-3: the native misroute is measured by rust/tests/unit/issue_1175_routing_probes.rs, never assumed.
    assert.notEqual(probe.rust_misroute, "unverified", `${probe.id}: a native misroute names the measured intent`);
    assert.notEqual(probe.rust_misroute, probe.rust_intent || probe.intent, `${probe.id}: a native misroute equal to the intent is no misroute`);
    prompts.add(probe.prompt);
    ids.add(probe.id);
  }
  assert.equal(probes.filter((probe) => probe.rust_misroute).length, budget.rust_misroute_ceiling,
    "rust_misroute_ceiling must equal the listed native misroutes (it may only fall)");
});

test("R1175-4: every lane meets the coverage budget or is exempt with a reason", () => {
  const lanes = new Set([...seedLanes(), ...budget.extraLanes.keys()]);
  const coverage = (lane) => probes.filter((probe) => probe.lane === lane || probe.intent === lane || probe.rust_intent === lane).length;
  const short = [];
  for (const lane of lanes) {
    const count = coverage(lane);
    if (budget.exemptLanes.has(lane)) {
      assert.ok(budget.exemptLanes.get(lane), `exempt lane ${lane} needs a reason`);
      assert.equal(count, 0, `lane ${lane} is covered by ${count} probes; drop its stale exemption`);
    } else if (count < budget.minimum_probes_per_lane) {
      short.push(`${lane} (${count})`);
    }
  }
  for (const lane of budget.exemptLanes.keys()) assert.ok(lanes.has(lane), `exemption names unknown lane ${lane}`);
  assert.deepEqual(short, [], `lanes under ${budget.minimum_probes_per_lane} probes`);
});

test("R1175-4: the browser worker routes every probe; misroutes only ratchet down", async () => {
  const intents = await answers();
  const newMisroutes = [];
  const fixed = [];
  const changed = [];
  for (const probe of probes) {
    const intent = intents.get(probe.id);
    if (!probe.js_misroute) {
      if (intent !== probe.intent) newMisroutes.push(`${probe.id} [${probe.lane}] ${JSON.stringify(probe.prompt)} -> ${intent}, expected ${probe.intent}`);
    } else if (intent === probe.intent) {
      fixed.push(`${probe.id} ${JSON.stringify(probe.prompt)} now answers ${intent}: remove its js_misroute line and lower js_misroute_ceiling`);
    } else if (intent !== probe.js_misroute) {
      changed.push(`${probe.id} ${JSON.stringify(probe.prompt)} now answers ${intent} (recorded ${probe.js_misroute})`);
    }
  }
  // `changed`: a ratcheted misroute moved to another wrong intent; update its js_misroute line.
  assert.deepEqual({ newMisroutes, fixed, changed }, { newMisroutes: [], fixed: [], changed: [] });
  const listed = probes.filter((probe) => probe.js_misroute).length;
  assert.equal(listed, budget.js_misroute_ceiling, "js_misroute_ceiling must equal the listed misroutes (it may only fall)");
});
