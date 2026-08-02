import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const binary = readFileSync("src/web/formal_ai_worker.wasm");
const { instance } = await WebAssembly.instantiate(binary, {});
const wasm = instance.exports;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

const fields = [
  "id", "kind", "role", "intent", "tool", "inputs", "outputs", "content",
  "sentAt", "demoLabel", "conversationId", "conversationTitle", "evidence",
  "accessCount", "writeCount",
];

function encodeValue(value) {
  if (value === null || value === undefined) return "n";
  if (Array.isArray(value)) return `l${value.map((item) => encodeURIComponent(String(item))).join(",")}`;
  if (typeof value === "boolean") return value ? "b1" : "b0";
  if (typeof value === "number") return `${Number.isInteger(value) ? "i" : "f"}${value}`;
  return `s${encodeURIComponent(String(value))}`;
}

function run(query, events = []) {
  const lines = [`q\t${encodeURIComponent(query)}`];
  for (const event of events) {
    lines.push(`e\t${fields.map((field) => encodeValue(event[field])).join("\t")}`);
  }
  const bytes = encoder.encode(lines.join("\n"));
  assert.ok(bytes.length <= wasm.input_capacity());
  new Uint8Array(wasm.memory.buffer, wasm.input_ptr(), bytes.length).set(bytes);
  const length = wasm.engine_memory_query(bytes.length);
  if (length === 0) return null;
  const output = decoder.decode(new Uint8Array(wasm.memory.buffer, wasm.output_ptr(), length));
  return JSON.parse(output);
}

function queryId(answer) {
  return /memory_query\n  id "([^"]+)"/u.exec(answer.evidence[0])?.[1];
}

const allFields = fields.join(", ");
const sqlAll = run(`SELECT ${allFields} FROM memory WHERE kind = 'fact'`);
const graphAll = run("query { memory(where: { kind: { eq: \"fact\" } }) { id kind role intent tool inputs outputs content sentAt demoLabel conversationId conversationTitle evidence accessCount writeCount } }");
assert.equal(sqlAll.intent, "memory_exact_query");
assert.equal(graphAll.intent, "memory_exact_query");
assert.equal(queryId(sqlAll), queryId(graphAll));
assert.match(sqlAll.evidence[0], /parser_engine "rust_shared_exact_parser"/u);
assert.match(sqlAll.evidence[0], /link_cli_substitution/u);

for (const [sql, graphql, effect] of [
  [
    "INSERT INTO memory (id, kind, content) VALUES ('m2', 'fact', 'created') RETURNING id, kind, content",
    "mutation { createMemory(input: { id: \"m2\", kind: \"fact\", content: \"created\" }) { id kind content } }",
    "create",
  ],
  [
    "UPDATE memory SET content = 'updated' WHERE id = 'm1' RETURNING id, content",
    "mutation { updateMemory(where: { id: { eq: \"m1\" } }, set: { content: \"updated\" }) { id content } }",
    "update",
  ],
  [
    "DELETE FROM memory WHERE id = 'm1' RETURNING id",
    "mutation { deleteMemory(where: { id: { eq: \"m1\" } }) { id } }",
    "delete",
  ],
]) {
  const left = run(sql);
  const right = run(graphql);
  assert.equal(queryId(left), queryId(right));
  assert.match(left.evidence[0], new RegExp(`effect ${effect}`, "u"));
}

const activeEvents = [
  { id: "m1", kind: "fact", accessCount: 2, writeCount: 1 },
  { id: "m2", kind: "fact", accessCount: 4, writeCount: 2 },
];
const events = [...activeEvents, { id: "r1", kind: "memory_retraction", inputs: "m1" }];
const statistics = run("SELECT COUNT(*) AS count, SUM(accessCount) AS sum, AVG(accessCount) AS average, MIN(accessCount) AS minimum, MAX(accessCount) AS maximum, VAR_POP(accessCount) AS variance, STDDEV_POP(accessCount) AS deviation FROM memory", events);
assert.equal(statistics.intent, "memory_exact_query");
for (const expected of [
  'count "integer:1"',
  'sum "integer:4"',
  'average "float:4.000000000000"',
  'minimum "integer:4"',
  'maximum "integer:4"',
  'variance "float:0.000000000000"',
  'deviation "float:0.000000000000"',
]) assert.ok(statistics.content.includes(expected), expected);

const inserted = run("INSERT INTO memory (id, content) VALUES ('m3', 'created') RETURNING id", activeEvents);
assert.equal(inserted.intent, "memory_exact_query");
assert.equal(inserted.memoryOperation.action, "program");
assert.equal(inserted.memoryOperation.appends[0].id, "m3");

const updated = run("UPDATE memory SET content = 'updated' WHERE id = 'm1' RETURNING id, content", activeEvents);
assert.equal(updated.memoryOperation.updates[0].id, "m1");
assert.equal(updated.memoryOperation.updates[0].fields.content, "updated");

const refused = run("DELETE FROM memory WHERE id = 'm1'", activeEvents);
assert.equal(refused.intent, "memory_exact_query_refused");
assert.equal(refused.memoryOperation, undefined);

for (const invalid of [
  "SELECT FROM memory",
  "SELECT content FROM secrets",
  "query { memory(where: { nope: { eq: 1 } }) { id } }",
  "query { memory { id ",
]) assert.equal(run(invalid).intent, "memory_exact_query_rejected", invalid);

assert.equal(run("remember this is not SQL"), null);
console.log("issue-708 browser query-language parity: ok");
