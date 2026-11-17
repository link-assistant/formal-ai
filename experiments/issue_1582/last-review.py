"""Supplied review corrections, zero autonomous authorship credit."""
from pathlib import Path

p=Path('rust/tests/integration/with_formal_ai/fixture_env.rs')
s=p.read_text().replace('  echo "ANTHROPIC_BASE_URL=$ANTHROPIC_BASE_URL"', '  echo "ANTHROPIC_BASE_URL=$ANTHROPIC_BASE_URL"\n  echo "CLAUDE_CODE_MAX_CONTEXT_TOKENS=$CLAUDE_CODE_MAX_CONTEXT_TOKENS"\n  echo "CLAUDE_CODE_MAX_OUTPUT_TOKENS=$CLAUDE_CODE_MAX_OUTPUT_TOKENS"')
p.write_text(s)
p=Path('data/seed/meanings-feature-capability.lino')
s=p.read_text().replace('  internet-access-help\n    defined-by action\n', '  internet-access-help\n    defined-by action\n    precondition "user asks how to enable read-only network access or grant harness permissions"\n    effect "explain named read permissions, explicit server restrictions and connectivity troubleshooting in the user language"\n    unit "one permission guidance response"\n    example "How do I enable internet access?"\n')
p.write_text(s)
p=Path('js/server/structured-log.mjs')
s=p.read_text().replace(" const redact=makeRedactor(env),records=input===''?[]:input.slice(0,-1).split('\\n').map", " // Fold only the indentation emitted by formatLogRecord; keep strict canonical decoding.\n const source=from==='lino'?input.replace(/\\n +(?=\\()/gu,' '):input;\n const redact=makeRedactor(env),records=source===''?[]:source.slice(0,-1).split('\\n').map")
p.write_text(s)
p=Path('rust/src/cli_logs.rs')
s=p.read_text().replace('    let mut records = Vec::<Value>::new();', '''    // Fold the whitespace-only nested presentation before canonical decoding.
    let mut flattened = String::new();
    if from == InputFormat::Lino {
        for line in input.split_inclusive('\\n') {
            let trimmed = line.trim_start_matches(' ');
            if line.starts_with(' ') && trimmed.starts_with('(') && flattened.ends_with('\\n') {
                flattened.pop();
                flattened.push(' ');
                flattened.push_str(trimmed);
            } else {
                flattened.push_str(line);
            }
        }
    }
    let input = if from == InputFormat::Lino { &flattened } else { input };
    let mut records = Vec::<Value>::new();''')
p.write_text(s)
p=Path('rust/tests/web/server-structured-log.test.mjs')
s=p.read_text().replace('encodeTyped,decodeTyped,createLogger,makeRedactor,convertLog,','encodeTyped,decodeTyped,createLogger,makeRedactor,convertLog,formatLogRecord,')
s+='''
test('nested diagnostic streams convert losslessly and still reject partial records',()=>{
  const records=[{message:'first',metadata:{value:[1,'two']}},{message:'last'}];
  const canonical=records.map(value=>encodeTyped(value)+'\\n').join('');
  const nested=records.map(value=>formatLogRecord(encodeTyped(value),'nested')+'\\n').join('');
  assert.equal(convertLog(nested,{env:{}}),canonical);
  assert.deepEqual(JSON.parse(convertLog(nested,{to:'pretty',env:{}})),records);
  assert.throws(()=>convertLog(nested.slice(0,-2)));
  assert.throws(()=>convertLog(nested.replace('two','%GG')));
});
'''
p.write_text(s)
p=Path('rust/tests/unit/cli_logs.rs')
s=p.read_text()+'''
#[test]
fn nested_diagnostic_streams_convert_without_changing_typed_values() {
    let nested = "(formal-ai-log-v1\\n  (object\\n    (field message\\n      (string first))))\\n(formal-ai-log-v1\\n  (null))\\n";
    let canonical = "(formal-ai-log-v1 (object (field message (string first))))\\n(formal-ai-log-v1 (null))\\n";
    assert_eq!(convert(nested, InputFormat::Lino, LogFormat::Lino).unwrap(), canonical);
    assert!(convert(&nested[..nested.len()-2], InputFormat::Lino, LogFormat::Lino).is_err());
}
'''
p.write_text(s)
p=Path('rust/tests/integration/with_formal_ai/issue_1582.rs')
s=p.read_text().replace('    let settings: serde_json::Value = serde_json::from_str(', '''    let catalog: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(home.join(".codex/formal-ai-model-catalog.json")).expect("Codex catalog"),
    ).expect("JSON");
    assert_eq!(catalog["models"][0]["context_window"], serde_json::json!(1234567));
    let settings: serde_json::Value = serde_json::from_str(''',1)
p.write_text(s)
