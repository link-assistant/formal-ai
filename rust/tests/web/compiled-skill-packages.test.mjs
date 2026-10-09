import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { compileNaturalLanguageSkill as compile } from '../../../js/agentic/crate/skill_compiler.mjs';
import { parseLino } from '../../../js/server/lino.mjs';
import { symbolicFromWorker } from '../../../js/server/solve.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installTextHost } from '../../../scripts/lib/text-capability-measures.mjs';

installTextHost();
const native = readFileSync(new URL('../unit/specification/natural_language_skill_compilation.rs', import.meta.url), 'utf8');
const constant = name => JSON.parse(native.match(new RegExp('const ' + name + ': &str = ("(?:[^"\\\\]|\\\\.)*");'))[1]);
const skill = constant('SKILL');
const trigger = constant('TRIGGER');
const response = constant('RESPONSE');
const independentId = (prefix, text) => {
  let hash = 14695981039346656037n;
  for (const byte of Buffer.from(text, 'utf8')) hash = ((hash ^ BigInt(byte)) * 1099511628211n) % 18446744073709551616n;
  return prefix + '_' + hash.toString(16).padStart(16, '0');
};

test('native trigger response fixture compiles with independently verified stable IDs', () => {
  const package_ = compile(skill);
  assert.equal(package_.status, 'compiled');
  assert.equal(package_.id, 'compiled_skill_32628c474c664f67');
  assert.equal(package_.rule_id, 'compiled_skill_rule_881391387cf00e11');
  assert.equal(package_.handler_id, 'compiled_skill_handler_aaf62aceed77ddcd');
  assert.equal(package_.legacy_behavior_rule_id, 'behavior_rule_runtime_32628c474c664f67');
  assert.equal(package_.source_description, skill);
  assert.equal(package_.trigger, trigger);
  assert.equal(package_.response, response);
  assert.equal(package_.skill_name, 'trigger_response_skill');
});

test('compiled package codec retains all fourteen ordered native fields', () => {
  const p = compile(skill);
  const record = parseLino(p.linksNotation());
  assert.equal(record.name, p.id);
  assert.deepEqual(record.children.map(node => [node.name, node.value]), [
    ['type', 'compiled_skill_package'], ['schema_version', '0.2.0'],
    ['package_kind', 'associative_package'], ['source', 'natural_language_skill'],
    ['source_description', skill], ['skill_name', 'trigger_response_skill'],
    ['trigger_rule', p.rule_id], ['trigger', trigger], ['normalized_trigger', trigger],
    ['compiled_handler', p.handler_id], ['handler_kind', 'deterministic_response'],
    ['response', response], ['replay_mode', 'exact_normalized_prompt'],
    ['legacy_behavior_rule_id', p.legacy_behavior_rule_id],
  ]);
  assert.equal(p.linksNotation().endsWith('\n'), false);
});

test('three E1 records preserve native field order and independently hashed doublets', () => {
  const p = compile(skill);
  const records = p.linkRecords();
  const schemas = [
    ['CompiledSkillPackage', 'associative_package', independentId('natural_language_skill', skill), [
      ['source_description', skill], ['skill_name', 'trigger_response_skill'],
      ['trigger_rule', p.rule_id], ['compiled_handler', p.handler_id], ['replay_mode', 'exact_normalized_prompt'],
    ]],
    ['CompiledSkillTriggerRule', 'substitution_rule', p.id, [
      ['trigger', trigger], ['normalized_trigger', trigger], ['handler', p.handler_id],
    ]],
    ['CompiledSkillHandler', 'deterministic_response_handler', p.id, [
      ['handler_kind', 'deterministic_response'], ['response', response],
    ]],
  ];
  assert.equal(records.length, 3);
  for (let index = 0; index < records.length; index += 1) {
    const r = records[index];
    const [type, subtype, source, fields] = schemas[index];
    const edges = [[r.stable_id, 'Type'], ['Type', type], [type, 'SubType'],
      ['SubType', subtype], [subtype, 'Value'], [r.stable_id, source]];
    for (const [key, value] of [['schema_version', '0.2.0'], ...fields]) {
      edges.push([r.stable_id, 'field:' + key], ['field:' + key, 'value:' + value]);
    }
    assert.equal(r.record_type, type);
    assert.equal(r.source_id, source);
    assert.equal(r.schema_version, '0.2.0');
    assert.deepEqual(r.links, edges.map(([from, to]) => ({ from, to, index: independentId('doublet', from + '->' + to) })));
  }
});

test('all unchanged native multilingual rows compile and replay exact response bytes', async () => {
  const rows = [...native.matchAll(/Case\s*\{\s*language:\s*"([^"\n]+)",\s*skill:\s*"([^"\n]+)",\s*trigger:\s*"([^"\n]+)",\s*response:\s*"([^"\n]+)",\s*\}/gu)];
  assert.equal(rows.length, 4);
  const host = new WorkerHost();
  for (const [, language, description, prompt, expected] of rows) {
    const p = compile(description);
    assert.equal(p.status, 'compiled', language);
    assert.equal(p.trigger, prompt, language);
    assert.equal(p.response, expected, language);
    assert.equal(p.replay(prompt).answer, expected, language);
    assert.equal(p.replay(prompt + ' extra'), null, language);
    const answer = await host.solve(prompt, [{ role: 'user', content: description }]);
    assert.equal(answer.intent, 'behavior_rule_custom', language);
    assert.equal(answer.content, expected, language);
    assert.equal(answer.solverEvents.find(event => event.kind === 'cache_hit').payload, p.id, language);
  }
});

test('held-out quoted Unicode and unquoted seeded teaching preserve response bytes', async () => {
  const description = 'When `Ω status` then `Bob\'s "ready"; 状态!`';
  const p = compile(description);
  assert.equal(p.response, 'Bob\'s "ready"; 状态!');
  assert.equal(p.replay(' Ω STATUS!!! ').answer, p.response);
  assert.equal(p.replay('Ω state'), null);
  assert.equal(parseLino(p.linksNotation()).children.find(node => node.name === 'response').value, p.response);
  const host = new WorkerHost();
  const teaching = await host.solve(description);
  const body = teaching.content.split('```links\n')[1].split('\n```')[0];
  assert.equal(parseLino(body).children.find(node => node.name === 'answer').value, p.response);
  const unquoted = compile('When I say pebble 928 answer with arbitrary response.');
  assert.equal(unquoted.status, 'compiled');
  assert.equal(unquoted.trigger, 'pebble 928');
  assert.equal(unquoted.response, 'arbitrary response');
});

test('unsupported structured IR never becomes a trigger response package or replay cache hit', async () => {
  const structured = native.match(/const TYPED_PROCEDURE_SKILL: &str = r"([\s\S]*?)";/u)[1];
  assert.equal(compile(structured).status, 'unsupported-structured-skill');
  for (const description of ['ordinary unrelated text', 'When `` then ``', 'Skill: `unknown`\nWhen `a` then `b`']) {
    assert.notEqual(compile(description).status, 'compiled');
  }
  const result = await new WorkerHost().solve('triage TCK-7 urgent', [{ role: 'user', content: structured }]);
  assert.equal(result.solverEvents.some(event => event.kind === 'cache_hit'), false);
});

// Keep every original native assertion in solver_prefers_compiled_skill_from_history_and_records_cache_hit.
test('original native history replay retains all five assertions through production WorkerHost', async () => {
  const result = symbolicFromWorker(await new WorkerHost().solve(trigger, [{ role: 'user', content: skill }]));
  assert.equal(result.intent, 'behavior_rule_custom');
  assert.equal(result.answer, response);
  assert.equal(result.evidence_links.some(link => link.startsWith('cache_hit:compiled_skill_')), true);
  assert.equal(result.links_notation.includes('compiled_skill:replay'), true);
  assert.equal(result.links_notation.includes('behavior_rule:match'), false);
});

// Keep every original native assertion in behavior_rules_count_includes_dialog_local_runtime_rules.
test('original native count has exact full answer including closing newline and all six assertions', async () => {
  const source = readFileSync(new URL('../unit/specification/behavior_rules.rs', import.meta.url), 'utf8');
  const body = source.split('fn behavior_rules_count_includes_dialog_local_runtime_rules()')[1].split('\n#[test]')[0];
  const quoted = [...body.matchAll(/"(?:[^"\\]|\\.)*"/gu)].map(match => JSON.parse(match[0]));
  const teaching = quoted.find(text => text.startsWith('When `'));
  const query = quoted.find(text => text.startsWith('How many '));
  const expected = quoted.find(text => text.startsWith('Total behavior rules: 9') && text.includes('\n'));
  assert.ok(teaching && query && expected);
  const result = await new WorkerHost().solve(query, [{ role: 'user', content: teaching }]);
  assert.equal(result.intent, 'behavior_rules_count');
  assert.equal(result.content, expected);
  assert.equal(result.content.includes('Total behavior rules: 9'), true);
  assert.equal(result.content.includes('built_in_rules "8"'), true);
  assert.equal(result.content.includes('dialog_local_rules "1"'), true);
  assert.equal(result.content.includes('total_rules "9"'), true);
});

test('true user assistant history teaches counts lists and replays the same package with ordered events', async () => {
  const host = new WorkerHost();
  const p = compile(skill);
  const history = [];
  const answers = [];
  for (const prompt of [skill, 'How many behavior rules are there?', 'Show rules', trigger]) {
    const answer = await host.solve(prompt, history);
    answers.push(answer);
    history.push({ role: 'user', content: prompt }, { role: 'assistant', content: answer.content });
  }
  const [teaching, count, list, replay] = answers;
  assert.equal(teaching.intent, 'behavior_rule_update');
  const record = parseLino(teaching.content.split('```links\n')[1].split('\n```')[0]);
  assert.equal(record.name, p.id);
  assert.deepEqual(record.children.map(node => [node.name, node.value]).filter(([name]) => name !== 'when_then'), [
    ['type', 'compiled_skill_package'], ['legacy_behavior_rule_id', p.legacy_behavior_rule_id],
    ['match_prompt', p.trigger], ['answer', p.response], ['compiled_handler', p.handler_id],
    ['replay_mode', 'exact_normalized_prompt'], ['source', 'user_message'],
  ]);
  assert.deepEqual(teaching.solverEvents.filter(event => ['skill_compile:package', 'behavior_rule:update'].includes(event.kind))
    .map(event => [event.kind, event.payload]), [['skill_compile:package', p.id], ['behavior_rule:update', p.legacy_behavior_rule_id]]);
  assert.equal(count.solverEvents.find(event => event.kind === 'behavior_rules:runtime_count').payload, '1');
  assert.equal(list.content.includes('`' + p.id + '` (`' + p.legacy_behavior_rule_id + '`)'), true);
  assert.equal(replay.content, p.response);
  assert.deepEqual(replay.solverEvents.filter(event => ['compiled_skill:package', 'compiled_skill:replay', 'cache_hit', 'response'].includes(event.kind))
    .map(event => [event.kind, event.payload]), [['compiled_skill:package', p.linksNotation()],
    ['compiled_skill:replay', p.rule_id], ['cache_hit', p.id], ['response', 'response:' + p.id]]);
});

test('canonical package dedup and latest user response ignore assistant impostors', async () => {
  const host = new WorkerHost();
  const old = 'When `quartz 631` then `older value`';
  const latest = 'When `quartz 631` then `new value`';
  const equivalent = 'When `QUARTZ 631!!!` then `new value`';
  const history = [{ role: 'user', content: old }, { role: 'user', content: latest },
    { role: 'user', content: equivalent }, { role: 'assistant', content: 'When `quartz 631` then `impostor`' }];
  const count = await host.solve('How many behavior rules are there?', history);
  assert.equal(count.solverEvents.find(event => event.kind === 'behavior_rules:runtime_count').payload, '2');
  const replay = await host.solve('quartz 631', history);
  assert.equal(replay.content, 'new value');
  assert.equal(replay.solverEvents.find(event => event.kind === 'cache_hit').payload, compile(equivalent).id);
  const nonmatch = await host.solve('quartz 631', [history[3]]);
  assert.equal(nonmatch.solverEvents.some(event => event.kind === 'cache_hit'), false);
  const baseline = await host.solve('4 + 4');
  assert.deepEqual(await host.solve('4 + 4', history), baseline);
});
