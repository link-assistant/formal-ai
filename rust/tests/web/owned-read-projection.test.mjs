import test from 'node:test';
import assert from 'node:assert/strict';
import { qualifyOwnedFullReads, rustTokenSignature } from '../../../scripts/qualify-owned-read-providers.mjs';

const owned = `
fn observe() {
    let mut records: Vec<(String, String)> = vec![];
    runner("arbitrary request", &mut |operation, params| match operation {
        "Write" => {
            records.push((params["file_path"].as_str().unwrap().to_owned(),
                params["content"].as_str().unwrap().to_owned()));
            String::from("ok")
        }
        "Read" => {
            let destination = params["file_path"].as_str().expect("path");
            records.iter().rev().find(|(key, _)| key == destination)
                .map_or_else(|| String::from("unknown failure"), |(_, bytes)| bytes.clone())
        }
        other => panic!("unknown operation: {other}"),
    });
    assert_eq!("unchanged assertion", "unchanged assertion");
}
fn runner(prompt: &str, execute: &mut dyn FnMut(&str, &Value) -> String) {
    let name = "Read";
    let arguments = json!({"file_path":"varied.txt"});
    let result = execute(name, &arguments);
    let message = json!({"content": result});
}
`;

test('owned full current Vec lookup generates typed transport with unchanged operands', async () => {
  const result = await qualifyOwnedFullReads(owned);
  assert.equal(result.edits.length, 7);
  assert.match(result.source, /complete_owned_read\(destination, bytes\)/u);
  assert.match(result.source, /tool_capability\(name\)/u);
  assert.match(result.source, /unwrap_or\(""\)/u);
  assert.ok(result.source.includes('assert_eq!("unchanged assertion", "unchanged assertion")'));
  assert.equal(await rustTokenSignature(result.source),
    await rustTokenSignature(result.source.replaceAll('    ', '  ')));
});

const refused = [
  ['stale first entry', source => source.replace('.iter().rev()', '.iter()')],
  ['partial data', source => source.replace('bytes.clone()', 'bytes.chars().take(2).collect()')],
  ['unknown storage type', source => source.replace('Vec<(String, String)>', 'ForeignStorage')],
  ['foreign key lookup', source => source.replace('key == destination', 'key == other')],
  ['non-equality lookup', source => source.replace('key == destination', 'key != destination')],
  ['invented path', source => source.replace('params["file_path"].as_str().expect("path")', '"foreign.txt"')],
  ['unknown path schema', source => source.replace('params["file_path"].as_str().expect("path")', 'params["other"].as_str().expect("path")')],
  ['unknown result branch', source => source.replace('String::from("ok")', 'external_response()')],
  ['shadowed callback arguments', source => source.replace('let destination =',
    'let params = json!({"file_path":"foreign.txt"}); let destination =')],
  ['destructured callback arguments', source => source.replace('let destination =',
    'let (params, _) = foreign_pair(); let destination =')],
  ['nested callback argument binding', source => source.replace('let destination =',
    'let _hidden = |params| params; let destination =')],
];
for (const [name, alter] of refused) {
  test(`unknown provider refuses: ${name}`, async () => {
    await assert.rejects(qualifyOwnedFullReads(alter(owned)), /No source-qualified/u);
  });
}

test('different owned binding names remain general', async () => {
  const source = owned.replaceAll('records', 'store').replaceAll('bytes', 'full_value');
  const result = await qualifyOwnedFullReads(source);
  assert.match(result.source, /complete_owned_read\(destination, full_value\)/u);
});
