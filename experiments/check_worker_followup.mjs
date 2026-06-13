import fs from 'node:fs';
import vm from 'node:vm';

const src = fs.readFileSync('src/web/formal_ai_worker.js', 'utf8');
const sandbox = {
  self: { location: { search: '' } },
  console,
  fetch: undefined,
  postMessage: () => {},
  TextEncoder, TextDecoder,
};
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename: 'formal_ai_worker.js' });

const g = sandbox;
function check(name, cond) {
  console.log(`${cond ? 'PASS' : 'FAIL'}  ${name}`);
  if (!cond) process.exitCode = 1;
}

// 1. lexicon resolves the new role to surfaces
const norm = g.normalizePrompt;
check('elab: english "can you give me specific instructions?"',
  g.isProceduralElaborationRequest(norm('Can you give me specific instructions?')));
check('elab: russian "дай конкретные инструкции"',
  g.isProceduralElaborationRequest(norm('дай конкретные инструкции')));
check('elab: chinese "给我具体步骤"',
  g.isProceduralElaborationRequest(norm('给我具体步骤')));
check('elab: negative "what is npm"',
  !g.isProceduralElaborationRequest(norm('what is npm')));

// 2. followup gate requires prior how-to dialogue
const history = [
  { role: 'user', content: 'how to publish to npm' },
  { role: 'assistant', content: 'Procedural discovery for `publish to npm` ...' },
];
const dlg = g.priorProceduralHowToDialogue(history);
check('dialogue recovers prior task', dlg && dlg.task.task === 'publish to npm');
check('dialogue null without history', g.priorProceduralHowToDialogue([]) === null);

// 3. full followup handler (fetch undefined -> web_search fallback path)
const res = await g.tryProceduralHowToFollowup('Can you give me specific instructions?', 'en', history);
check('followup returns procedural_how_to', res && res.intent === 'procedural_how_to');
check('followup content mentions task', res && res.content.includes('publish to npm'));
check('followup evidence has followup marker',
  res && res.evidence.some(e => e.startsWith('procedural_how_to:followup:')));
check('followup evidence has request', res && res.evidence.some(e => e === 'procedural_how_to:request:publish to npm'));

// 4. no rebind when there is no prior how-to
const noHist = await g.tryProceduralHowToFollowup('Can you give me specific instructions?', 'en', []);
check('no rebind without prior how-to', noHist === null);
