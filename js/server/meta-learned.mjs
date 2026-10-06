// Learned meta-reasoner chunks across sessions (R1007, R1012): the server half
// of rust/src/memory_sync.rs `LEARNED_CHUNK_KIND`, `learned_statements`, the
// `meta_reasoner::import_learned` call in `SyncStore::open_at`, and the
// `meta_reasoner::take_learned` event `record_chat_exchange_with_tools`
// appends.
//
// The worker keeps the chunks themselves (`metaImportLearned`,
// `metaAttachLearned` in js/worker/formal_ai_worker_meta_composite.js) and
// hands the newly learned ones out as an answer's `append` memory operation.
// The server holds those statements here until the exchange is recorded, as
// the native `NEW_CHUNKS` set holds them until `take_learned`.

/** Mirrors `memory_sync::LEARNED_CHUNK_KIND`. */
export const LEARNED_CHUNK_KIND = 'meta_learned_chunk';
const APPEND_ACTION = 'append';
const CHUNK_HEADER = LEARNED_CHUNK_KIND;

/** Statements learned since the last `takeLearned`, keyed by chunk word. */
const pending = new Map();
let importer = null;

/** Mirrors `memory_sync::learned_statements`: the contents of the log's chunk events. */
export function learnedStatements(events) {
  return (events || [])
    .filter((event) => event.kind === LEARNED_CHUNK_KIND && event.content !== null && event.content !== undefined)
    .map((event) => String(event.content));
}

/**
 * Install the worker-side `import_learned` (the server wires
 * `metaImportLearned` here once its worker exists).
 * @param {((statements: Array<string>) => unknown) | null} fn
 */
export function setLearnedImporter(fn) {
  importer = typeof fn === 'function' ? fn : null;
}

/** The `import_learned` call `SyncStore::open_at` makes over every opened log. */
export function importLearned(events) {
  const statements = learnedStatements(events);
  if (!statements.length || !importer) return;
  try {
    Promise.resolve(importer(statements)).catch(() => undefined);
  } catch {
    // Importing is best-effort, exactly as Rust ignores the loaded count.
  }
}

/** Split a statement into its `meta_learned_chunk` blocks, keyed by word. */
function chunkBlocks(statement) {
  const blocks = [];
  for (const line of String(statement).split('\n')) {
    if (line === CHUNK_HEADER) blocks.push([line]);
    else if (blocks.length) blocks[blocks.length - 1].push(line);
  }
  return blocks.map((lines) => {
    const wordLine = lines.find((line) => line.startsWith('  word '));
    let word = '';
    try {
      word = wordLine ? String(JSON.parse(wordLine.slice('  word '.length))) : '';
    } catch {
      word = wordLine ? wordLine.slice('  word '.length) : '';
    }
    return { word, text: lines.join('\n') };
  });
}

/** Hold a worker answer's learned-chunk `append` memory operation, if any. */
export function noteLearned(memoryOperation) {
  if (!memoryOperation || memoryOperation.kind !== LEARNED_CHUNK_KIND || memoryOperation.action !== APPEND_ACTION) return;
  for (const block of chunkBlocks(memoryOperation.statement)) {
    if (!pending.has(block.word)) pending.set(block.word, block.text);
  }
}

/**
 * Mirrors `meta_reasoner::take_learned`: every chunk learned since the last
 * call as one statement, in word order, or null; the set is emptied.
 * @returns {string|null}
 */
export function takeLearned() {
  if (!pending.size) return null;
  const words = [...pending.keys()].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
  const statement = words.map((word) => pending.get(word)).join('\n');
  pending.clear();
  return statement;
}
