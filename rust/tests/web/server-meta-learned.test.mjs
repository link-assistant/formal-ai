// Learned meta-reasoner chunks persist through the JavaScript server's memory
// log (R1007, R1012), as rust/src/memory_sync.rs does: opening the store
// hands every `meta_learned_chunk` event to the worker's `metaImportLearned`,
// and recording an exchange appends the chunks learned since the last record
// as one `meta_learned_chunk` event with id `stable_id("meta_learned_chunk",
// statement)`, between the tool calls and the task.

import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, test } from "node:test";

import { stableId } from "../../../js/server/ids.mjs";
import { SyncStore } from "../../../js/server/memory-store.mjs";
import {
  LEARNED_CHUNK_KIND,
  learnedStatements,
  noteLearned,
  setLearnedImporter,
  takeLearned,
} from "../../../js/server/meta-learned.mjs";
import { symbolicFromWorker } from "../../../js/server/solve.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";

const home = mkdtempSync(path.join(os.tmpdir(), "formal-ai-meta-learned-"));
after(() => {
  setLearnedImporter(null);
  rmSync(home, { recursive: true, force: true });
});

const chunk = (word, operation) =>
  `meta_learned_chunk\n  word ${JSON.stringify(word)}\n  operation ${operation}\n  score 0.9\n  via "test"`;

test("take_learned merges pending chunks in word order and empties the set", () => {
  assert.equal(takeLearned(), null);
  noteLearned({ action: "append", kind: LEARNED_CHUNK_KIND, statement: chunk("zib", "sort_list") });
  noteLearned({ action: "append", kind: LEARNED_CHUNK_KIND, statement: chunk("abe", "reverse_text") });
  noteLearned({ action: "append", kind: "memory_write", statement: chunk("ignored", "x") });
  assert.equal(takeLearned(), `${chunk("abe", "reverse_text")}\n${chunk("zib", "sort_list")}`);
  assert.equal(takeLearned(), null);
});

test("recording an exchange appends the learned chunk event in Rust's shape", () => {
  const env = { FORMAL_AI_MEMORY_PATH: path.join(home, "record.lino"), FORMAL_AI_RECORD_CHAT: "1" };
  const statement = chunk("glorp", "reverse_text");
  symbolicFromWorker({ content: "x", memoryOperation: { action: "append", kind: LEARNED_CHUNK_KIND, statement } });
  const store = SyncStore.open(env);
  assert.equal(store.recordChatExchangeWithTools("prompt", "answer"), 3);
  const kinds = store.events.map((event) => event.kind);
  assert.deepEqual(kinds, ["message", LEARNED_CHUNK_KIND, "task"]);
  const learned = store.events[1];
  assert.equal(learned.id, stableId(LEARNED_CHUNK_KIND, statement));
  assert.equal(learned.role, "assistant");
  assert.equal(learned.content, statement);
  assert.equal(learned.write_count, 1);
  assert.deepEqual(learnedStatements(SyncStore.open(env).events), [statement]);
  assert.equal(SyncStore.open(env).recordChatExchangeWithTools("prompt 2", "answer 2"), 2);
});

test("opening the store imports its chunks into the worker's meta reasoner", async () => {
  const worker = new WorkerHost();
  await worker.boot();
  setLearnedImporter((statements) => worker.run("metaImportLearned(__learned)", { __learned: statements }));
  const file = path.join(home, "import.lino");
  const statement = chunk("snorkle", "sort_list");
  writeFileSync(
    file,
    `demo_memory\n  event "${stableId(LEARNED_CHUNK_KIND, statement)}"\n    kind "${LEARNED_CHUNK_KIND}"\n    role "assistant"\n    content ${JSON.stringify(statement)}\n`,
  );
  const store = SyncStore.open({ FORMAL_AI_MEMORY_PATH: file });
  assert.deepEqual(learnedStatements(store.events), [statement]);
  assert.equal(await worker.run("metaLearnedChunks.has('snorkle') && metaLearnedChunks.get('snorkle').operation"), "sort_list");
});
