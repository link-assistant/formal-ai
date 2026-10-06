// User-facing TRIZ/contradiction solver (issue #901), mirroring
// rust/src/triz_solver.rs. Every family, benchmark task and cue phrase is a
// record of data/seed/triz-principles.lino; the answer renders through the
// `triz_resolution_map` template of data/seed/multilingual-responses-triz.lino.

/** The range-selection note (Rust `LINK_NOTE`). */
const TRIZ_LINK_NOTE = "A contradiction is a link whose value (0-1) is chosen from the requirement's own clauses — basis points in the selection heuristic, with no default 50 %. Say which side the requirements favor and the point on the range follows.";

let cachedTrizRecords = null;

/**
 * The top-level records of the TRIZ seed.
 * @returns {{name: string, value: string, children: object[]}[]}
 */
function trizRecords() {
  if (cachedTrizRecords) return cachedTrizRecords;
  const text = seedRawText(SEED_RAW, "triz-principles.lino");
  if (!text) return [];
  cachedTrizRecords = parseLinoTree(text).children;
  return cachedTrizRecords;
}

/**
 * @param {{children: object[]}} node
 * @param {string} name
 * @returns {string}
 */
function trizChildValue(node, name) {
  const child = (node.children || []).find((candidate) => candidate.name === name);
  return child ? String(child.value || "") : "";
}

/**
 * The seed records of one type. The seed names its records individually
 * (`triz_family_range_selection`, `triz_task_umbrella_crowd`) and types them
 * with a `record_type` field, so the type is read from that field; a record
 * literally named after the type is accepted as well. (The native
 * `records_named` compares the record *name* only, which selects none of
 * the typed family/task rows — see the parity notes of issue #901.)
 * @param {string} recordType
 * @param {string} recordName
 * @returns {object[]}
 */
function trizRecordsOfType(recordType, recordName) {
  return trizRecords().filter((record) =>
    record.name === recordName || trizChildValue(record, "record_type") === recordType);
}

/**
 * The general resolution families, in seed order.
 * @returns {{methodId: string, name: string, mechanism: string}[]}
 */
function trizFamilies() {
  return trizRecordsOfType("triz_resolution_family", "triz_resolution_family")
    .map((record) => ({
      methodId: trizChildValue(record, "method_id").trim(),
      name: trizChildValue(record, "name"),
      mechanism: trizChildValue(record, "mechanism"),
    }))
    .filter((family) => family.methodId !== "");
}

/**
 * The top-20 benchmark corpus, in seed order.
 * @returns {{taskId: string, domain: string, statement: string, contradiction: string, methods: string[]}[]}
 */
function trizBenchmarkTasks() {
  return trizRecordsOfType("triz_benchmark_task", "triz_benchmark_task")
    .map((record) => ({
      taskId: trizChildValue(record, "task_id").trim(),
      domain: trizChildValue(record, "domain"),
      statement: trizChildValue(record, "statement"),
      contradiction: trizChildValue(record, "contradiction"),
      methods: trizChildValue(record, "methods").split(/\s+/u).filter((method) => method !== ""),
    }))
    .filter((task) => task.taskId !== "");
}

/**
 * The cue phrases that mark a contradiction question.
 * @returns {string[]}
 */
function trizCues() {
  const cues = [];
  for (const record of trizRecordsOfType("triz_intent_cues", "triz_cues")) {
    for (const child of record.children) {
      if (child.name !== "phrase") continue;
      const phrase = String(child.value || "").trim().toLowerCase();
      if (phrase !== "") cues.push(phrase);
    }
  }
  return cues;
}

/**
 * Benchmark tasks sharing a five-plus-character word with the prompt, best
 * match first (Rust `relevant_tasks`).
 * @param {string} prompt
 * @param {object[]} tasks
 * @returns {object[]}
 */
function trizRelevantTasks(prompt, tasks) {
  const words = String(prompt || "")
    .split(/\s+/u)
    .map((word) => word.replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, ""))
    .filter((word) => Array.from(word).length >= 5);
  const scored = [];
  for (const task of tasks) {
    const haystack = `${task.domain} ${task.statement}`.toLowerCase();
    const score = words.filter((word) => haystack.includes(word.toLowerCase())).length;
    if (score > 0) scored.push({ score, task });
  }
  scored.sort((left, right) => {
    if (left.score !== right.score) return right.score - left.score;
    if (left.task.taskId < right.task.taskId) return -1;
    return left.task.taskId > right.task.taskId ? 1 : 0;
  });
  return scored.map((entry) => entry.task);
}

/**
 * `triz_resolution` precedence row (Rust `handle_triz`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryTrizResolution(prompt, normalized) {
  const lower = String(prompt || "").toLowerCase();
  const text = String(normalized || "");
  const cued = trizCues().some((cue) => text.includes(cue) || lower.includes(cue));
  if (!cued) return null;
  const families = trizFamilies();
  const tasks = trizBenchmarkTasks();
  const relevant = trizRelevantTasks(prompt, tasks);
  const evidence = ["handler:triz_resolution", "triz_solver:cued:contradiction question"];
  for (const task of relevant) evidence.push(`triz_solver:precedent:${task.taskId}`);
  const cited = (relevant.length === 0 ? tasks : relevant).slice(0, 3);
  const familiesText = families
    .map((family) => `- ${family.name}: ${family.mechanism}`)
    .join("\n");
  const precedentsText = cited
    .map((task) =>
      `- ${task.statement} (${task.domain}: ${task.contradiction}) — methods: ${task.methods.join(", ")}`)
    .join("\n");
  const table = MULTILINGUAL_ANSWERS.triz_resolution_map || {};
  const raw = table.en || table.unknown;
  const template = raw ? (typeof raw === "string" ? raw : String(raw.text || "")) : "";
  const content = template
    .split("{families}").join(familiesText)
    .split("{precedents}").join(precedentsText)
    .split("{link_note}").join(TRIZ_LINK_NOTE);
  evidence.push("response:triz_resolution_map");
  return { intent: "triz_resolution", content, confidence: 0.7, evidence };
}
