import fs from 'node:fs';import {execFileSync} from 'node:child_process';
const root='/Users/konard/Code/Archive/link-assistant/formal-ai',scratch='/private/tmp/formal-ai-only-1188',id='T3090',changes=[];
function replace(source,old,next){if(!source.includes(old))throw Error('missing '+old.slice(0,100));return source.replace(old,next);}
function modify(path,edit){const before=fs.existsSync(root+'/'+path)?fs.readFileSync(root+'/'+path,'utf8'):null;let expected=edit(before);if(path.endsWith('.rs')){const filename=scratch+'/'+id+'-'+path.split('/').at(-1);fs.writeFileSync(filename,expected);expected=execFileSync('rustfmt',['--edition','2024','--config','skip_children=true','--emit','stdout',filename],{encoding:'utf8'});const prefix=filename+':\n\n';if(expected.startsWith(prefix))expected=expected.slice(prefix.length);}changes.push({path,before,expected});}
modify('rust/src/agentic_coding/write_request.rs',source=>replace(source,
"/// The opening line a sentence pins, read through\n/// [`seed::ROLE_FILE_LEADING_LINE_CONSTRAINT_LEAD`].\n///\n/// The lowercased copy is byte-length preserving for every supported language,\n/// Cue spans map back to the original sentence before recovering the line.",
"/// Recover the seeded opening-line constraint through original UTF-8 boundaries.\n/// Lowercase cue matching never changes the bytes of the recovered line."));
modify('rust/tests/web/literal-write-request-scope.test.mjs',source=>source+String.raw`

test('expanding lowercase prefixes preserve unquoted original content', () => {
  for (const prefix of ['İİ', 'İİİ', 'İK𐐷 😀 café', 'KK 😀 中文']) {
    const prompt = 'Note ' + prefix + '.\nCreate file x.txt containing hello.';
    assert.equal(composeGeneralChangePlan(prompt)?.content, 'hello.', prompt);
  }
});
test('raw content spans preserve the original multilingual payload and closer', () => {
  for (const [instruction, expected] of [
    ['Create file x.txt containing «hello».', 'hello'],
    ['Создай файл x.txt с содержимым «привет».', 'привет'],
    ['Crea el archivo x.txt con el contenido «hola».', 'hola'],
    ['创建 x.txt 内容为 «你好»。', '你好'],
    ['बनाओ x.txt ठीक इसी सामग्री के साथ «नमस्ते»।', 'नमस्ते'],
    ['Create file `policy/retention.md` containing Logs are kept for ninety days; backups are kept for a year.', 'Logs are kept for ninety days; backups are kept for a year.'],
    ["Create a file new.txt containing 'hello'.", 'hello'],
  ]) {
    const prompt = 'Note İİK𐐷 😀 café.\n' + instruction;
    assert.equal(composeGeneralChangePlan(prompt)?.content, expected, prompt);
  }
});
test('raw-span mapping rejects interior expansions and split surrogate characters', async () => {
  const { rawLowercaseSpan, rawContentLeadClose } = await import('../../../js/agentic/write_request/lowercase_spans.mjs');
  assert.deepEqual(rawLowercaseSpan('İK😀x', [2, 5]), [1, 4]);
  assert.equal(rawLowercaseSpan('İx', [1, 2]), null);
  assert.equal(rawLowercaseSpan('😀x', [1, 2]), null);
  assert.equal(rawLowercaseSpan('x', [2, 3]), null);
  assert.equal(rawLowercaseSpan('x', [1, 0]), null);
  assert.equal(rawContentLeadClose('😀x', 1), null);
});
test('objective labels map back before quoting and line anchoring', async () => {
  const { objectiveText } = await import('../../../js/agentic/general_planner.mjs');
  assert.equal(objectiveText('Note İİK😀.\nTask: Create file x.txt containing hello'), 'Create file x.txt containing hello');
  const quoted = 'Note İİK😀.\nCreate file x.txt containing «Task: preserve this». ';
  assert.equal(objectiveText(quoted), quoted);
});
test('Unicode prefixes cannot promote cross-statement or quoted owner cues', () => {
  for (const task of [
    'Note İİK😀.\nWrite file.\nx.txt containing «hello».',
    'Note İİK😀.\nThe instruction says «write».\nRead file x.txt containing hello.',
    'Note İİK😀.\nProduce a legitimate release with no fabricated evidence.\nAdding a bypass flag to check-self-development-release.rs is not acceptable.',
  ]) assert.notEqual(composeGeneralChangePlan(task)?.mode, 'literal_file', task);
});
`);
modify('rust/tests/fixtures/literal-obligation-transaction.rs',source=>source+String.raw`

#[test]
fn unicode_case_mapping_preserves_original_unquoted_literal_bytes() {
    for prefix in ["İİ", "İİİ", "İK𐐷 😀 café", "KK 😀 中文"] {
        let task = format!("Note {prefix}.\nCreate file x.txt containing hello.");
        let outcome = run(&task, &["write"]);
        assert_eq!(outcome.writes, 1, "{task}");
        assert_eq!(fs::read_to_string(outcome.root.join("x.txt")).expect("original target bytes"), "hello.", "{task}");
    }
}

#[test]
fn unicode_prefixes_preserve_multilingual_literals_and_sentence_marks() {
    use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
    for (instruction, expected) in [
        ("Create file x.txt containing «hello».", "hello"),
        ("Создай файл x.txt с содержимым «привет».", "привет"),
        ("Crea el archivo x.txt con el contenido «hola».", "hola"),
        ("创建 x.txt 内容为 «你好»。", "你好"),
        ("बनाओ x.txt ठीक इसी सामग्री के साथ «नमस्ते»।", "नमस्ते"),
        ("Create file `policy/retention.md` containing Logs are kept for ninety days; backups are kept for a year.", "Logs are kept for ninety days; backups are kept for a year."),
        ("Create a file new.txt containing 'hello'.", "hello"),
    ] {
        let task = format!("Note İİK𐐷 😀 café.\n{instruction}");
        let plan = compose_general_change_plan(&task).expect("literal operand");
        assert_eq!(plan.content, expected, "{task}");
    }
}

#[test]
fn objective_boundaries_remain_original_after_case_mapping() {
    use formal_ai::agentic_coding::general_planner::objective_text;
    assert_eq!(objective_text("Note İİK😀.\nTask: Create file x.txt containing hello"), "Create file x.txt containing hello");
    let task = "Note İİK😀.\nCreate file x.txt containing «Task: preserve this». ";
    assert_eq!(objective_text(task), task);
}

#[test]
fn unicode_case_mapping_does_not_promote_unrelated_write_owners() {
    use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
    for task in [
        "Note İİK😀.\nWrite file.\nx.txt containing «hello».",
        "Note İİK😀.\nThe instruction says «write».\nRead file x.txt containing hello.",
        "Note İİK😀.\nProduce a legitimate release with no fabricated evidence.\nAdding a bypass flag to check-self-development-release.rs is not acceptable.",
    ] { assert!(compose_general_change_plan(task).is_none(), "{task}"); }
}
`);
const file=scratch+'/'+id+'-request.json';const code="const fs=require('fs');const changes=JSON.parse(fs.readFileSync('"+file+"','utf8')).changes;for(const change of changes){if((fs.existsSync(change.path)?fs.readFileSync(change.path,'utf8'):null)!==change.before)throw Error('preimage changed '+change.path);}for(const change of changes){fs.mkdirSync(require('path').dirname(change.path),{recursive:true});fs.writeFileSync(change.path,change.expected);}console.log(JSON.stringify({writes:changes.map(change=>change.path)}));";const prompt='Run node -e '+"'"+code.replaceAll("'","'\\''")+"'";fs.writeFileSync(file,JSON.stringify({id,dir:root,prompt,changes,reviewedGeneralRepair:true},null,2));
const {drive}=await import(root+'/experiments/js_dogfood/drive.mjs');const {WorkerHost}=await import(root+'/js/server/worker-host.mjs');const {installNodeHost}=await import(root+'/js/agentic/node-host.mjs');await installNodeHost(new WorkerHost());const {planChatStep}=await import(root+'/js/agentic/planner.mjs');const out=await drive(planChatStep,root,prompt,{steps:4});const raw=JSON.stringify(out,null,2);fs.writeFileSync(scratch+'/'+id+'-transcript.json',raw);fs.writeFileSync(scratch+'/'+id+'-transcript.log',raw);console.log(JSON.stringify({id,tools:out.transcript.map(row=>row.tool),exact:changes.every(change=>fs.readFileSync(root+'/'+change.path,'utf8')===change.expected)}));
