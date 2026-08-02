// Exercise the browser worker's byte-buffer protocol without the UI.
// Run from the repository root with `node experiments/issue709_wasm_call.js`.
import fs from 'node:fs';

const payload = [
  ['Q', 'Apple', 'en', 'Read more', 'via', 'Other sources'],
  ['S', 'https://duckduckgo.com/Apple', 'Apple',
    'Apple is a fruit produced by an apple tree.', '', 'en', 'duckduckgo#1', '2', 'primary'],
  ['S', 'https://en.wikipedia.org/wiki/Apple', 'Apple',
    'Apple is the edible fruit of an apple tree.', '', 'en', 'wikipedia#1', '1', 'primary'],
  ['S', 'https://www.wikidata.org/wiki/Q89', 'Apple',
    'fruit of the apple tree', '', 'en', 'wikidata#1', '1', 'alternate'],
].map((row) => row.map((value) => encodeURIComponent(value)).join('\t')).join('\n');

WebAssembly.instantiate(fs.readFileSync('src/web/formal_ai_worker.wasm')).then(({ instance }) => {
  const wasm = instance.exports;
  const input = new TextEncoder().encode(payload);
  new Uint8Array(wasm.memory.buffer, wasm.input_ptr(), input.length).set(input);
  const outputLength = wasm.web_search_statement_fusion(input.length);
  const output = new TextDecoder().decode(
    new Uint8Array(wasm.memory.buffer, wasm.output_ptr(), outputLength),
  );
  console.log(`payload_bytes=${input.length}`);
  console.log(`output_bytes=${outputLength}`);
  console.log(output);
});
