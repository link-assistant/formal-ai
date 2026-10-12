// The white-box derivation record in the browser worker (issue #1184, R1184-9).
//
// The twin of rust/src/derivation.rs (and of its JavaScript root
// js/agentic/crate/derivation.mjs, which a classic worker cannot import):
// `finalizeWorkerDerivation` mirrors `finalize_answer`, run on every answer the
// web UI receives, and `explainWorkerDerivation` mirrors
// `formal-ai explain <answer-id> [--format text|links]` (`cli_explain::run_explain`)
// as a chat request. One event log in (`answer.solverEvents` plus the
// `render:emit` event), one record out; a stage a route never populated stays
// empty and is printed as "not recorded", never fabricated.
//
// Persistence goes through an injected store, `{ readText(path), writeText(path,
// text) }`, the same io `persist` / `load` take in the crate module. In the
// browser the store is the app's append-only memory event log (js/memory.js,
// `FormalAiMemory`): reads come from the `memoryEvents` snapshot the app sends
// with every request, and a write is returned as the answer's
// `derivationRecord`, which the app appends as a `derivation` event, the same
// convention as `memoryOperation` (the worker stays pure, the app applies the
// write). The record's path is the Rust one, `data/cache/derivations/<id>.lino`,
// relative to the browser's empty repository root. The applied-rule kinds come
// from data/seed/derivation-schema.lino and the miss wording from the seed's
// `derivation_record_missing` response, never from this file.

const DERIVATION_NOT_RECORDED = "not recorded";
const DERIVATION_KIND = Object.freeze({
  search: "web_search:request",
  source: "source:http",
  fragment: "formalize:fragment",
  part: "decompose:part",
  bind: "recompose:bind",
  render: "render:emit",
  verify: "verify:evidence",
});
const DERIVATIONS_DIR = "data/cache/derivations";
const DERIVATION_SCHEMA_FILE = "seed/derivation-schema.lino";
const DERIVATION_RULE_FIELD = "rule";
const DERIVATION_EVENT_KIND = "derivation";
const DERIVATION_EXPLAIN_ROLE = "capability_act_explain";
const DERIVATION_EXPLAIN_INTENT = "derivation_explain";
const DERIVATION_MISSING_INTENT = "derivation_record_missing";
const DERIVATION_ID_PATTERN = /\banswer_[0-9a-f]{16}\b/u;
const DERIVATION_LINKS_FORMAT = /(?:^|\s)--format(?:=|\s+)links(?:\s|$)/u;

let derivationRuleKindsCache = null;

/** `parse_lino` root shape over the shared seed parser (seed_parser.mjs `parseRoot`). */
function derivationParseRoot(text) {
  const tree = self.FormalAiSeed.parse(String(text || ""));
  if (tree.indent === -1 || (tree.name === "" && tree.value === "" && tree.indent === undefined)) return tree;
  return { name: "", id: "", value: "", children: [tree], indent: -1 };
}

function derivationChildValue(node, name) {
  const child = ((node && node.children) || []).find((candidate) => candidate.name === name);
  return child && child.value !== undefined ? child.value : "";
}

/** Mirrors `fn applied_rule_kinds`: the schema's `stage` kinds that collect `rule`. */
function derivationAppliedRuleKinds() {
  if (derivationRuleKindsCache) return derivationRuleKindsCache;
  const kinds = new Set();
  for (const schema of derivationParseRoot(SEED_RAW[DERIVATION_SCHEMA_FILE]).children) {
    for (const row of schema.children) {
      if (row.name !== "stage" || derivationChildValue(row, "collects") !== DERIVATION_RULE_FIELD) continue;
      const kind = derivationChildValue(row, "kind");
      if (kind !== "") kinds.add(kind);
    }
  }
  derivationRuleKindsCache = kinds;
  return kinds;
}

const derivationTrim = (text) => String(text).replace(/^\p{White_Space}+|\p{White_Space}+$/gu, "");

function derivationSplitOnce(text, separator, last = false) {
  const index = last ? text.lastIndexOf(separator) : text.indexOf(separator);
  return index < 0 ? null : [text.slice(0, index), text.slice(index + separator.length)];
}

/** Rust `i64` parse: optional sign, ASCII digits, within range; else `null`. */
function derivationParseI64(text) {
  if (!/^[+-]?[0-9]+$/u.test(text)) return null;
  const value = BigInt(text);
  return value < -(2n ** 63n) || value > 2n ** 63n - 1n ? null : Number(value);
}

const derivationExitText = (code) => (code === null || code === undefined ? "none" : String(code));

/** Mirrors `fn VerificationRecord::parse_payload`. */
function derivationParseVerification(payload) {
  if (!payload.startsWith("evidence_id=")) return null;
  const head = derivationSplitOnce(payload.slice(12), ";command=") || derivationSplitOnce(payload.slice(12), " command=");
  if (head === null) return null;
  const body = derivationSplitOnce(head[1], ";exit=", true) || derivationSplitOnce(head[1], " exit=", true);
  if (body === null || head[0] === "") return null;
  return { evidence_id: head[0], command: body[0], exit_code: derivationParseI64(body[1]) };
}

/** Mirrors `fn parse_source_http`: either payload spelling, `null` without a url. */
function derivationParseSourceHttp(payload) {
  const trimmed = derivationTrim(payload);
  if (trimmed === "") return null;
  const parts = trimmed.startsWith("url=") ? trimmed.slice(4).split(";") : trimmed.split(" ");
  const url = trimmed.startsWith("url=") ? derivationTrim(parts[0]) : parts[0];
  if (url === "") return null;
  const fields = [];
  for (const raw of parts.slice(1)) {
    const pair = derivationSplitOnce(derivationTrim(raw), "=");
    if (pair !== null) fields.push([derivationTrim(pair[0]), derivationTrim(pair[1])]);
  }
  const field = (name) => (fields.find(([key]) => key === name) || [])[1] || "";
  return { url, sha256: field("sha256"), fetched_at: field("fetched_at") };
}

function derivationEmpty(answerId) {
  return {
    answer_id: answerId, search_queries: [], fetches: [], formalized_fragments: [], decomposed_parts: [],
    recomposition: null, rendering: null, verification: [], applied_rules: [],
  };
}

/** Mirrors `fn Derivation::record_for` over `{kind, payload}` events in append order. */
function derivationRecordFor(events, answerId) {
  const derivation = derivationEmpty(answerId);
  for (const { kind, payload } of events) {
    if (kind === DERIVATION_KIND.search) derivation.search_queries.push(payload);
    else if (kind === DERIVATION_KIND.source) {
      const fetched = derivationParseSourceHttp(payload);
      if (fetched !== null) derivation.fetches.push(fetched);
    } else if (kind === DERIVATION_KIND.fragment) derivation.formalized_fragments.push(payload);
    else if (kind === DERIVATION_KIND.part) derivation.decomposed_parts.push(payload);
    else if (kind === DERIVATION_KIND.bind) derivation.recomposition = payload;
    else if (kind === DERIVATION_KIND.render) derivation.rendering = payload;
    else if (kind === DERIVATION_KIND.verify) {
      const record = derivationParseVerification(payload);
      if (record !== null) derivation.verification.push(record);
    } else if (derivationAppliedRuleKinds().has(kind)) derivation.applied_rules.push({ kind, detail: payload });
  }
  return derivation;
}

/** links_format.mjs `formatLinoValue`: sanitize, then the codec's quoting. */
function derivationLinoValue(value) {
  const text = String(value).replaceAll("\\", "\\\\").replaceAll("\r", "\\r").replaceAll("\n", "\\n").replaceAll("\t", "\\t");
  const single = text.includes("'");
  const double = text.includes("\"");
  if (double && !single) return `'${text}'`;
  if (single && !double) return `"${text}"`;
  return single ? `'${text.replaceAll("'", "''")}'` : `"${text}"`;
}

/** Mirrors `fn Derivation::to_lino` (`push_lino_node` rows). */
function derivationToLino(derivation) {
  const rows = [];
  const push = (indent, name, value = null) =>
    rows.push(`${" ".repeat(indent)}${name}${value === null ? "" : ` ${derivationLinoValue(value)}`}\n`);
  push(0, "derivation");
  push(2, "answer_id", derivation.answer_id);
  for (const query of derivation.search_queries) push(2, "search_query", query);
  for (const fetched of derivation.fetches) {
    push(2, "fetch");
    push(4, "url", fetched.url);
    push(4, "sha256", fetched.sha256);
    push(4, "fetched_at", fetched.fetched_at);
  }
  for (const fragment of derivation.formalized_fragments) push(2, "formalized_fragment", fragment);
  for (const part of derivation.decomposed_parts) push(2, "decomposed_part", part);
  if (derivation.recomposition !== null) push(2, "recomposition", derivation.recomposition);
  if (derivation.rendering !== null) push(2, "rendering", derivation.rendering);
  for (const record of derivation.verification) {
    push(2, "verification");
    push(4, "evidence_id", record.evidence_id);
    push(4, "command", record.command);
    push(4, "exit", derivationExitText(record.exit_code));
  }
  for (const rule of derivation.applied_rules) {
    push(2, DERIVATION_RULE_FIELD);
    push(4, "kind", rule.kind);
    push(4, "detail", rule.detail);
  }
  return rows.join("");
}

/** Mirrors `fn Derivation::from_lino`: `null` for a non-record or a missing `answer_id`. */
function derivationFromLino(text) {
  const record = derivationParseRoot(text).children.find((child) => child.name === "derivation");
  if (record === undefined) return null;
  const answerId = derivationChildValue(record, "answer_id");
  if (answerId === "") return null;
  const derivation = derivationEmpty(answerId);
  for (const child of record.children) {
    const field = (name) => derivationChildValue(child, name);
    if (child.name === "search_query") derivation.search_queries.push(child.id);
    else if (child.name === "fetch") derivation.fetches.push({ url: field("url"), sha256: field("sha256"), fetched_at: field("fetched_at") });
    else if (child.name === "formalized_fragment") derivation.formalized_fragments.push(child.id);
    else if (child.name === "decomposed_part") derivation.decomposed_parts.push(child.id);
    else if (child.name === "recomposition") derivation.recomposition = child.id;
    else if (child.name === "rendering") derivation.rendering = child.id;
    else if (child.name === "verification") {
      const exit = field("exit");
      derivation.verification.push({ evidence_id: field("evidence_id"), command: field("command"), exit_code: exit === "none" ? null : derivationParseI64(exit) });
    } else if (child.name === DERIVATION_RULE_FIELD) derivation.applied_rules.push({ kind: field("kind"), detail: field("detail") });
  }
  return derivation;
}

function derivationStage(name, rows) {
  return `  stage ${name}\n${(rows.length === 0 ? [DERIVATION_NOT_RECORDED] : rows).map((row) => `    ${row}\n`).join("")}`;
}

/** Mirrors `fn Derivation::explain_text`. */
function derivationExplainText(derivation) {
  return [
    `derivation ${derivation.answer_id}\n`,
    derivationStage("search_queries", derivation.search_queries),
    derivationStage("fetches", derivation.fetches.map((fetched) =>
      ["url", fetched.url, "sha256", fetched.sha256, "fetched_at", fetched.fetched_at].join(" "))),
    derivationStage("formalized_fragments", derivation.formalized_fragments),
    derivationStage("decomposed_parts", derivation.decomposed_parts),
    `  stage recomposition: ${derivation.recomposition === null ? DERIVATION_NOT_RECORDED : derivation.recomposition}\n`,
    `  stage rendering: ${derivation.rendering === null ? DERIVATION_NOT_RECORDED : derivation.rendering}\n`,
    derivationStage("verification", derivation.verification.map((record) => `${record.command} exit=${derivationExitText(record.exit_code)}`)),
    derivationStage("applied_rules", derivation.applied_rules.map((rule) => `${rule.kind} ${rule.detail}`)),
  ].join("");
}

const derivationJoin = (root, relative) => (root === "" ? relative : root.endsWith("/") ? `${root}${relative}` : `${root}/${relative}`);

/** Mirrors `fn store_path`: `null` unless the id is a `[A-Za-z0-9_-]` token. */
function derivationStorePath(root, answerId) {
  return /^[A-Za-z0-9_-]+$/u.test(answerId) ? derivationJoin(derivationJoin(root, DERIVATIONS_DIR), `${answerId}.lino`) : null;
}

/** Mirrors `fn Derivation::persist` over the injected store: `{ok, path}` or `{ok: false, error}`. */
function derivationPersist(derivation, root, store) {
  const path = derivationStorePath(root, derivation.answer_id);
  if (path === null) return { ok: false, error: "derivation_answer_id_unusable" };
  if (!store || typeof store.writeText !== "function") return { ok: false, error: "derivation_store_has_no_writer" };
  try {
    store.writeText(path, derivationToLino(derivation));
    return { ok: true, path };
  } catch (error) {
    return { ok: false, error: `derivation_write_failed:${(error && error.message) || error}` };
  }
}

/** Mirrors `fn Derivation::load`: the stored record, or `null` for any miss. */
function derivationLoad(root, answerId, store) {
  const path = derivationStorePath(root, answerId);
  if (path === null || !store || typeof store.readText !== "function") return null;
  let text;
  try {
    text = store.readText(path);
  } catch (_error) {
    return null;
  }
  const derivation = typeof text === "string" ? derivationFromLino(text) : null;
  return derivation !== null && derivation.answer_id === answerId ? derivation : null;
}

/** Mirrors `fn miss_message`: the seed's English report text with its slots filled. */
function derivationMissMessage(root, answerId) {
  const table = MULTILINGUAL_ANSWERS[DERIVATION_MISSING_INTENT] || {};
  const raw = table.en;
  const template = typeof raw === "string" ? raw : raw && typeof raw.text === "string" ? raw.text : DERIVATION_MISSING_INTENT;
  const slots = { answer_id: answerId, directory: derivationJoin(root, DERIVATIONS_DIR) };
  return template.replace(/\{([a-z_]+)\}/gu, (whole, slot) => (Object.prototype.hasOwnProperty.call(slots, slot) ? slots[slot] : whole));
}

/**
 * The browser store over the app's memory event log: the newest `derivation`
 * event filed under `path` reads back, and a write lands in `writes` for the
 * app to append through `FormalAiMemory.appendEvent`.
 */
function derivationMemoryStore(memoryEvents, writes = []) {
  return {
    readText(path) {
      const events = Array.isArray(memoryEvents) ? memoryEvents : [];
      const found = events.filter((event) => event && event.kind === DERIVATION_EVENT_KIND && event.inputs === path).pop();
      if (!found) throw new Error(`${DERIVATION_EVENT_KIND}:${path}`);
      return String(found.content || "");
    },
    writeText(path, text) {
      writes.push({ action: DERIVATION_EVENT_KIND, path, text });
    },
  };
}

/**
 * Mirrors `fn finalize_answer` for a browser answer: append the `render:emit`
 * event, project the record, link it and persist it through `store`.
 */
function finalizeWorkerDerivation(answer, store = null) {
  if (!answer || typeof answer.content !== "string") return answer;
  const answerId = stableBehaviorRuleId("answer", answer.content);
  const events = (Array.isArray(answer.solverEvents) ? answer.solverEvents : [])
    .map((event) => ({ kind: String(event.kind), payload: String(event.payload === undefined || event.payload === null ? "" : event.payload) }));
  events.push({ kind: DERIVATION_KIND.render, payload: `answer_id=${answerId};format=text` });
  const record = derivationRecordFor(events, answerId);
  const writes = [];
  const persisted = derivationPersist(record, "", store || derivationMemoryStore([], writes));
  const evidence = [...(answer.evidence || []), `derivation:${answerId}`,
    persisted.ok ? persisted.path : `derivation:persistence_failed:${persisted.error}`];
  return { ...answer, derivationId: answerId, evidence, derivationRecord: writes[0] || null };
}

/**
 * Mirrors `formal-ai explain <answer-id> [--format text|links]` as a chat
 * request: a prompt that names an answer id and evidences the seed's explain
 * act prints the stored record (`explain_text`, or its Links Notation), and a
 * miss answers with the seed's `derivation_record_missing` text.
 */
function explainWorkerDerivation(prompt, memoryEvents, store = null) {
  const text = String(prompt || "");
  const match = DERIVATION_ID_PATTERN.exec(text);
  if (!match || !lexiconMentionsRole(DERIVATION_EXPLAIN_ROLE, normalizePrompt(text))) return null;
  const answerId = match[0];
  const derivation = derivationLoad("", answerId, store || derivationMemoryStore(memoryEvents));
  const found = derivation !== null;
  return {
    intent: found ? DERIVATION_EXPLAIN_INTENT : DERIVATION_MISSING_INTENT,
    content: !found ? derivationMissMessage("", answerId)
      : DERIVATION_LINKS_FORMAT.test(text) ? derivationToLino(derivation) : derivationExplainText(derivation),
    confidence: found ? 1 : 0,
    evidence: found ? [`derivation:${answerId}`, derivationStorePath("", answerId)] : [],
    steps: [],
    toolCalls: [],
  };
}
