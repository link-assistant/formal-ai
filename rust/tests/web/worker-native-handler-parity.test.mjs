import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { specificationCases, testFunctions, tokenize } from '../../../scripts/lib/rust-specification-cases.mjs';
import { typedProgramOf, executeTypedProgram } from '../../../scripts/lib/rust-specification-programs.mjs';

const root = new URL('../../../', import.meta.url).pathname;
const originals = [
  ['rust/tests/unit/specification/code_generation/follow_up.rs', 'english_follow_up_modification_emits_substitution_plan_trace', 4, 4],
  ['rust/tests/unit/specification/natural_language_access.rs', 'natural_language_code_execution_gate_is_stable_across_supported_language_contexts', 3, 12],
  ['rust/tests/unit/specification/pronoun_topic_follow_up.rs', 'pronoun_followup_resolves_prior_rust_topic_for_creator_question', 6, 6],
];
for (const [file, name, macros, executions] of originals) {
  test('whole original native assertion program: ' + name, async () => {
    const source = readFileSync(root + file, 'utf8');
    const native = testFunctions(tokenize(source)).find(item => item.name === name);
    assert.ok(native);
    const parsed = typedProgramOf(native.body, { source, file: root + file, root });
    assert.ok(parsed.program, parsed.reason);
    assert.equal(parsed.program.nativeAssertions, macros);
    const result = await executeTypedProgram(new WorkerHost(), parsed.program);
    assert.equal(result.status, 'passed', result.failure);
    assert.equal(result.assertions, executions);
    assert.equal(result.nativeAssertions, macros);
    assert.ok(specificationCases(root).cases.find(item => item.id === file + '::' + name)?.program);
    const final = result.observations.at(-1).answer;
    if (name.includes('substitution')) {
      const event = final.solver_events.find(item => item.kind === 'write_program_plan');
      assert.ok(event?.payload.includes('list_files_arg'));
    } else if (name.includes('execution_gate')) {
      for (const observation of result.observations) {
        const events = observation.answer.solver_events.filter(item => item.kind === 'policy:agent_mode_required_for_tools');
        assert.equal(events.length, 1);
        assert.equal(events[0].payload, 'tool:javascript_execution');
      }
    } else {
      assert.ok(final.solver_events.some(item => item.kind === 'coreference:resolved' && item.payload === 'it=Rust'));
      assert.ok(final.solver_events.some(item => item.kind === 'coreference:rewrite' && item.payload === 'who created rust'));
      assert.ok(final.solver_events.some(item => item.kind === 'wikidata' && item.payload === 'Q575650'));
    }
  });
}

test('loaded seeded pronoun contexts survive terminal punctuation and Unicode boundaries', async () => {
  const host = new WorkerHost();
  const cases = [
    ['Who created it?', 'it'], ['Compare it.', 'it'], ['Ask about results!', 'results'],
    ['Explain the program?', 'program'], ['Объясни результаты?', 'результаты'],
    ['Объясни программу.', 'программа'], ['इन परिणामों?', 'परिणाम'],
    ['यह प्रोग्राम?', 'प्रोग्राम'], ['解释结果。', '结果'], ['解释程序？', '程序'],
  ];
  for (const [prompt, token] of cases) {
    const actual = await host.run('matchingCoreferencePronoun(normalizePrompt(__contextPrompt))?.token ?? null', { __contextPrompt: prompt });
    assert.equal(actual, token, prompt);
  }
  for (const prompt of ['Who created with?', 'Explain programming?', 'xрезультатыx', 'xपरिणामोंx', 'itself']) {
    const actual = await host.run('matchingCoreferencePronoun(normalizePrompt(__contextPrompt))?.token ?? null', { __contextPrompt: prompt });
    assert.equal(actual, null, prompt);
  }
});

test('actual matching seed is required before any coreference event is recorded', async () => {
  const host = new WorkerHost();
  const first = await host.solve('What is Rust?');
  const history = [{ role: 'user', content: 'What is Rust?' }, { role: 'assistant', content: first.content }];
  const answer = await host.solve('Who created with?', history);
  assert.ok(Array.isArray(answer.solverEvents));
  assert.ok(!answer.solverEvents.some(item => item.kind.startsWith('coreference:')));
});


test('embedded antecedent words do not fabricate a recognized prior user topic',async()=>{
  const host=new WorkerHost();
  for(const content of ['Rustic furniture is handmade.','A programming textbook.','xпрограммаx']){
    const reply=await host.solve('Explain the program?',[{role:'user',content}]);
    assert.ok(!reply.solverEvents.some(event=>event.kind==='coreference:resolved'),content);
    assert.equal(await host.run('nearestCoreferenceAntecedent(__turns)',{__turns:[{role:'user',content}]}),null,content);
  }
});


test('real seeded antecedents defer meta reasoning only for command-head references', async()=>{
  const host=new WorkerHost();
  const original='Write me a Rust program that lists the files in the current directory';
  const first=await host.solve(original);
  assert.equal(first.intent,'write_program');
  const history=[{role:'user',content:original},{role:'assistant',content:first.content}];
  for(const prompt of ['Explain the program?','Объясни результаты?','Объясни программу.','इन परिणामों?','यह प्रोग्राम?','解释结果。','解释程序？']){
    const reply=await host.solve(prompt,history);
    assert.equal(reply.intent,'coreference_program_artifact',prompt);
    assert.ok(reply.solverEvents.some(event=>event.kind==='coreference:resolved'),prompt);
  }
  for(const [prompt,turns]of [
    ['Explain the program?',[]],['Explain the program?',[{role:'user',content:'I enjoy fresh apples.'}]],
    ['Explain programming?',history],['itself',history],['xрезультатыx',history],['xपरिणामोंx',history],
    ['Summarize: Explain the program? It has three steps.',history],
    ['Rewrite "Explain the program? It has three steps."',history],
  ]){
    const reply=await host.solve(prompt,turns);
    assert.ok(!reply.solverEvents.some(event=>event.kind==='coreference:resolved'),prompt);
  }
});
