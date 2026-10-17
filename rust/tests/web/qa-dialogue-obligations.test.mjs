import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {formalizeText,statementTerms} from '../../../js/agentic/crate/text_formalization.mjs';
import {formalizeDialogue,answerCompatibility} from '../../../js/agentic/crate/dialogue_formalization.mjs';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const worker=new WorkerHost();
await installNodeHost(worker);
if(process.env.FORMAL_AI_QA_CANDIDATE_ROOT) {
  // Private sparse qualification runs actual candidate factories in the maintained JS host.
  const {factoryOf}=await import(path.join(process.env.FORMAL_AI_QA_MAINTAINED_ROOT,
    'scripts/generate-worker-crate-modules.mjs'));
  for(const name of ['dialogue_formalization','text_formalization']) {
    const source=fs.readFileSync(path.join(root,'js/agentic/crate',name+'.mjs'),'utf8');
    await worker.run(factoryOf('crate/'+name+'.mjs',source)+'\n;null;');
  }
  const seed=fs.readFileSync(path.join(root,'data/seed/text-formalization.lino'),'utf8');
  await worker.run(`SEED_RAW=Object.fromEntries(Object.entries(SEED_RAW).filter(([name])=>seedFileBaseName(name)!=='text-formalization.lino'));SEED_RAW['seed/text-formalization.lino']=__qaSeed;CRATE_SEED_RAW={...CRATE_SEED_RAW};crateInstances.delete('crate/text_formalization.mjs');`,{__qaSeed:seed});
  for(const [file,names] of [
    ['formal_ai_worker_capabilities_and_runtime_rules.js',['isConcretePropertyQuestion','tryFeatureCapabilityStatus','tryCapabilities']],
    ['formal_ai_worker_equations_and_word_problems.js',['isIdentityPrompt']],
    ['formal_ai_worker_solver_events.js',['solverRouteForPrompt']],
  ]) {
    const source=fs.readFileSync(path.join(root,'js/worker',file),'utf8');
    for(const name of names) {
      const expression=new RegExp('function '+name+'\\([^]*?\\n}');
      const definition=source.match(expression)?.[0];
      assert(definition,'actual maintained function '+name);
      await worker.run(definition+'\n;null;');
    }
  }
  const current=host();
  installHost({...current,readText:relative=>relative==='data/seed/text-formalization.lino'?seed:current.readText(relative)});
}

test('unseen bilingual property questions carry typed obligations',()=>{
  for(const [text,language,kind,topic] of [
    ['Where do you reside now?','en','where','location'],
    ['Где вы сейчас живёте?','ru','where','location'],
    ['How many years of experience with Elixir?','en','how-many','experience'],
    ['Какой у вас стаж с GraphQL?','ru','which','experience'],
    ['Could you describe what compensation you expect in EUR?','en','how-many','compensation'],
  ]) {
    const actual=formalizeText(text,language)[0].dialogue.question;
    assert.equal(actual.kind,kind);assert.equal(actual.topic,topic);
  }
});

test('negative-only answers survive statement formalization and cannot answer where',()=>{
  for(const [text,language,polarity] of [['Нет.','ru','denied'],['No','en','denied'],
    ['не готова','ru','denied'],['YES!','en','affirmed'],['Да','ru','affirmed']]) {
    const statement=formalizeText(text,language)[0];
    assert(statement);assert.equal(statement.dialogue.answer.polarity,polarity);
    if(polarity==='denied')assert.equal(statement.polarity,'denied');
  }
  const question=formalizeDialogue('Where are you located?','en').question;
  assert.equal(answerCompatibility(question,formalizeDialogue('No','en').answer).status,'incompatible');
});

test('grouped numbers remain exact single terms with currency and recurrence',()=>{
  for(const [surface,value,language,currency] of [
    ['От 987 654 ₽ в месяц на руки.','987654','ru','RUB'],
    ['From $8\u202f765 net per month.','8765','en','USD'],
    ['At least €9,007,199,254,740,993 net per month.','9007199254740993','en','EUR'],
  ]) {
    const statements=formalizeText(surface,language);
    const actual=statements.flatMap(statement=>statement.dialogue.quantities);
    assert.equal(actual.length,1);assert.equal(actual[0].value,value);
    assert.equal(actual[0].currency,currency);assert.equal(actual[0].comparison,'at-least');
    assert.equal(actual[0].period,'month');assert.equal(actual[0].net,true);
    const numbers=statements.flatMap(statementTerms).filter(term=>term.kind==='number');
    assert.equal(numbers.length,1);assert.equal(numbers[0].id,'number:'+value);
  }
});

test('typed matching rejects cross-currency answers and missing units stays unknown',()=>{
  const salary=formalizeDialogue('What salary expectations in USD?','en').question;
  assert.equal(answerCompatibility(salary,formalizeDialogue('450000 RUB','en').answer).status,'incompatible');
  assert.equal(answerCompatibility(salary,formalizeDialogue('450000','en').answer).status,'unknown');
  const years=formalizeDialogue('How many years of experience?','en').question;
  assert.equal(answerCompatibility(years,formalizeDialogue('3','en').answer).status,'unknown');
  assert.equal(formalizeDialogue('До 3 лет','ru').quantities[0].comparison,'at-most');
});

test('malformed groups are never silently combined and unknown geography stays literal',()=>{
  const invalid=formalizeDialogue('12 34 USD','en').quantities;
  assert(!invalid.some(quantity=>quantity.value==='1234'));
  const place=formalizeDialogue('In Riverton, Coastland.','en').answer;
  assert.equal(place.kind,'location');assert(place.locations.includes('Riverton'));
  assert(!JSON.stringify(place).includes('wikidata'));
});

test('actual solver rejects applicant property hijacks while retaining self-description',async()=>{
  for(const prompt of [
    'Есть ли у Вас опыт работы с API, чтение JSON-структур и использование инструментов вроде Postman для отправки и анализа запросов?',
    'Could you please tell us what are your salary expectations? (in USD, NET/ month)',
    'Do you have experience with GraphQL and YAML?',
    'Please tell me what are your compensation expectations in EUR?',
  ]) {
    const actual=await worker.solve(prompt);
    assert.equal(actual.intent,'unknown',prompt+': '+actual.intent);
  }
  assert.equal((await worker.solve('Tell me about yourself')).intent,'identity');
  assert.equal((await worker.solve('Кто ты?')).intent,'identity');
  assert.equal((await worker.solve('Can you run JavaScript?')).intent,'capabilities');
});
