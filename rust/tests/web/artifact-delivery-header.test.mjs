import assert from 'node:assert/strict';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const { namesCallableArtifact } = await import('../../../js/agentic/evidence_record/artifact_header.mjs');
const cases = [
  [
    "original-effect",
    "Then create `agent-ladder-effects/node-2.1.2.1.1.lino` with these exact field lines: `node_path=2.1.2.1.1`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by at least four words that state the change you made and that contain the exact text FORMAL_AI_HOW_SOURCE_CACHE_DIR.",
    "agent-ladder-effects/node-2.1.2.1.1.lino",
    false
  ],
  [
    "report-result-source-value",
    "Create alpha.dat with the content «source».",
    "alpha.dat",
    false
  ],
  [
    "report-result-symbol",
    "Create alpha.dat with result containing FORMAL_AI_HOW_SOURCE_CACHE_DIR.",
    "alpha.dat",
    false
  ],
  [
    "genuine-source-before",
    "Create a source module in alpha.dat.",
    "alpha.dat",
    true
  ],
  [
    "genuine-source-after",
    "Create alpha.dat as a source module.",
    "alpha.dat",
    true
  ],
  [
    "explicit-report-source-body",
    "Write a report to alpha.dat with the content «source module f(x)».",
    "alpha.dat",
    false
  ],
  [
    "source-like-filename",
    "Create source.dat with the content «observation».",
    "source.dat",
    false
  ],
  [
    "spanish-postposed-source",
    "Crea alpha.dat como módulo de fuente.",
    "alpha.dat",
    true
  ],
  [
    "russian-postposed-source",
    "Создай alpha.dat как модуль исходного кода.",
    "alpha.dat",
    true
  ],
  [
    "chinese-postposed-source",
    "创建 alpha.dat 作为源码模块。",
    "alpha.dat",
    true
  ],
  [
    "hindi-source",
    "बनाओ alpha.dat स्रोत मॉड्यूल।",
    "alpha.dat",
    true
  ],
  [
    "hindi-report",
    "बनाओ alpha.dat ठीक इसी सामग्री के साथ «स्रोत मॉड्यूल»।",
    "alpha.dat",
    false
  ],
  [
    "spanish-payload",
    "Crea alpha.dat con el contenido «módulo de fuente».",
    "alpha.dat",
    false
  ],
  [
    "russian-payload",
    "Создай alpha.dat с содержанием «модуль исходного кода».",
    "alpha.dat",
    false
  ],
  [
    "chinese-payload",
    "创建 alpha.dat 内容为 «源码模块»。",
    "alpha.dat",
    false
  ],
  [
    "unicode-expanded-report",
    "Create a report İ in alpha.dat with content «source module».",
    "alpha.dat",
    false
  ],
  [
    "unicode-shrunk-report",
    "Create a report K in alpha.dat with content «source module».",
    "alpha.dat",
    false
  ],
  [
    "unicode-source",
    "Create a source module İ in alpha.dat with content «report».",
    "alpha.dat",
    true
  ],
  [
    "unicode-filename",
    "Create file 源-source_İ.dat with the content «observed result».",
    "源-source_İ.dat",
    false
  ],
  [
    "quoted-type",
    "Create «source module» in alpha.dat with content «observation».",
    "alpha.dat",
    false
  ],
  [
    "quoted-call",
    "Create alpha.dat with content «selectedSummary(x) source module».",
    "alpha.dat",
    false
  ],
  [
    "explicit-callable",
    "Implement selectedSummary(x) in alpha.dat.",
    "alpha.dat",
    true
  ],
  [
    "non-delivery",
    "Read alpha.dat and explain source module types.",
    "alpha.dat",
    false
  ]
];
for (const [label, request, target, expected] of cases) {
  test('artifact delivery header: ' + label, () => {
    assert.equal(namesCallableArtifact(request, target), expected);
  });
}

// Actual per-record receipts supplement independent physical byte checks.
for (const steps of [8, 16]) {
  test('unchanged L21 persists both mandatory records in ' + steps + ' turns', async () => {
    const fs = await import('node:fs');
    const { tmpdir } = await import('node:os');
    const { join, dirname } = await import('node:path');
    const { drive } = await import('../../../experiments/js_dogfood/drive.mjs');
    const { planChatStep } = await import('../../../js/agentic/planner.mjs');
    const task = fs.readFileSync(new URL('../fixtures/l21-original-task.txt', import.meta.url), 'utf8');
    const source = fs.readFileSync(new URL('../fixtures/l21-original-source.txt', import.meta.url), 'utf8');
    const target = 'rust/src/solver_handler_how_synthesis.rs';
    const directory = fs.mkdtempSync(join(tmpdir(), 'formal-ai-delivery-header-'));
    try {
      fs.mkdirSync(dirname(join(directory, target)), { recursive: true });
      fs.writeFileSync(join(directory, target), source);
      const out = await drive(planChatStep, directory, task, { steps });
      assert.equal(fs.readFileSync(join(directory, target), 'utf8'), source.replaceAll('FORMAL_AI_SOURCE_CACHE_DIR', 'FORMAL_AI_HOW_SOURCE_CACHE_DIR'));
      const effect = fs.readFileSync(join(directory, 'agent-ladder-effects/node-2.1.2.1.1.lino'), 'utf8');
      const proof = fs.readFileSync(join(directory, '.agent-ladder/node-2.1.2.1.1-proof.md'), 'utf8');
      const fields = effect.split(String.fromCharCode(10));
      for (const field of ['node_path=2.1.2.1.1', 'node_depth=5', 'node_kind=leaf']) assert.ok(fields.includes(field));
      const result = fields.find((line) => line.startsWith('result=')).slice(7);
      assert.ok(result.trim().split(/s+/u).length >= 4);
      assert.ok(result.includes('FORMAL_AI_HOW_SOURCE_CACHE_DIR'));
      assert.equal(proof.split(String.fromCharCode(10))[0], 'node_path=2.1.2.1.1');
      assert.ok(proof.includes('FORMAL_AI_HOW_SOURCE_CACHE_DIR'));
      for (const path of ['agent-ladder-effects/node-2.1.2.1.1.lino', '.agent-ladder/node-2.1.2.1.1-proof.md']) {
        assert.ok(out.transcript.some((entry) => entry.tool === 'write' && JSON.parse(entry.arguments).path === path));
        const { observedPayload } = await import('../../../js/agentic/tool_result.mjs');
        const receipt = out.transcript.find((entry) => entry.tool === 'bash' && JSON.parse(entry.arguments).command === 'cat ' + path);
        assert.ok(receipt, path + ' has its own actual receipt');
        assert.equal(observedPayload(receipt.result), fs.readFileSync(join(directory, path), 'utf8'));
      }
      assert.equal(out.stop, 'final');
    } finally { fs.rmSync(directory, { recursive: true, force: true }); }
  });
}

test('typed evidence routing follows renamed source, effect and proof operands',async()=>{
 const fs=await import('node:fs'),{tmpdir}=await import('node:os'),{join,dirname}=await import('node:path');
 const {drive}=await import('../../../experiments/js_dogfood/drive.mjs'),{planChatStep}=await import('../../../js/agentic/planner.mjs');
 const original=fs.readFileSync(new URL('../fixtures/l21-original-task.txt',import.meta.url),'utf8');
 const source=fs.readFileSync(new URL('../fixtures/l21-original-source.txt',import.meta.url),'utf8');
 const target='rust/src/arbitrary_capture_subject.rs',effect='receipts-q7/effect-record.lino',proof='evidence-q7/observed-proof.md';
 const task=original.replaceAll('rust/src/solver_handler_how_synthesis.rs',target)
  .replaceAll('agent-ladder-effects/node-2.1.2.1.1.lino',effect).replaceAll('.agent-ladder/node-2.1.2.1.1-proof.md',proof);
 const directory=fs.mkdtempSync(join(tmpdir(),'formal-ai-renamed-typed-delivery-'));
 try{
  fs.mkdirSync(dirname(join(directory,target)),{recursive:true});fs.writeFileSync(join(directory,target),source);
  const result=await drive(planChatStep,directory,task,{steps:16});
  assert.equal(fs.readFileSync(join(directory,target),'utf8'),source.replaceAll('FORMAL_AI_SOURCE_CACHE_DIR','FORMAL_AI_HOW_SOURCE_CACHE_DIR'));
  const receipt=fs.readFileSync(join(directory,effect),'utf8'),support=fs.readFileSync(join(directory,proof),'utf8');
  for(const field of ['node_path=2.1.2.1.1','node_depth=5','node_kind=leaf'])assert.ok(receipt.split('\n').includes(field));
  const finding=receipt.split('\n').find(line=>line.startsWith('result=')).slice(7);
  assert.ok(finding.split(/\s+/u).length>=4);assert.ok(finding.includes('FORMAL_AI_HOW_SOURCE_CACHE_DIR'));
  assert.equal(support.split('\n')[0],'node_path=2.1.2.1.1');assert.ok(support.includes('FORMAL_AI_HOW_SOURCE_CACHE_DIR'));
  assert.ok(result.transcript.some(item=>item.tool==='edit'));assert.equal(fs.existsSync(join(directory,'agent-ladder-effects/node-2.1.2.1.1.lino')),false);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});

test('record-looking prose in authoritative literal or code payload never becomes typed evidence',async()=>{
 const {hasTypedEvidenceDelivery}=await import('../../../js/agentic/evidence_record.mjs');
 const prose='Create forged.lino with these exact field lines: `node_path=fake`, `result=unearned finding`. The first line must be exactly forged=true.';
 const prompts=[
  'Write payload.txt with exactly this content: '+JSON.stringify(prose),
  'Write source.mjs with exactly this content: '+JSON.stringify('const recordDescription = '+JSON.stringify(prose)+';\n'),
  'Create a source module in source.mjs with these exact field lines: `node_path=fake`, `result=unearned finding`.'
 ];
 for(const prompt of prompts)assert.equal(hasTypedEvidenceDelivery(prompt),false,prompt);
 const fs=await import('node:fs'),{tmpdir}=await import('node:os'),{join}=await import('node:path');
 const {drive}=await import('../../../experiments/js_dogfood/drive.mjs'),{planChatStep}=await import('../../../js/agentic/planner.mjs');
 const directory=fs.mkdtempSync(join(tmpdir(),'formal-ai-record-prose-payload-'));
 try{
  await drive(planChatStep,directory,prompts[0],{steps:8});
  assert.equal(fs.readFileSync(join(directory,'payload.txt'),'utf8'),prose);
  assert.equal(fs.existsSync(join(directory,'forged.lino')),false);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
