// Finite route and page round-trip investigation; fixtures only, no Rust.
import { readFileSync } from 'node:fs';
import { WorkerHost, evaluate } from '../../js/server/worker-host.mjs';
const host = new WorkerHost();
const context = await host.boot();
const corpus = readFileSync('data/benchmarks/web-formalization/en.lino', 'utf8').split('\n');
const start = corpus.indexOf('  title "Moon"');
const paragraphs = [];
for (let at = start + 1; at < corpus.length && corpus[at].startsWith('  '); at++) {
  const match = /^ {2}paragraph "(.*)"$/u.exec(corpus[at]);
  if (match) paragraphs.push(match[1]);
}
context.fixturePage = paragraphs.join('\n');
const page = evaluate(context, `(() => {
  const p = crateModule('crate/page_formalization.mjs');
  const t = crateModule('crate/text_formalization.mjs');
  const report = p.formalizePage(fixturePage, 'en');
  const statements = report.sentences.flatMap(sentence => sentence.statements);
  return {counts: {...report, sentences: report.sentences.length}, statements: statements.map(statement => {
    const text = t.deformalizeStatement(statement, 'en');
    const reread = t.clauseStatement(text, 'en', null, false);
    return {source: statement.text, text, known: t.knownIds(statement), back: t.knownIds(reread), survives: p.factSurvives(statement)};
  })};
})()`);
console.log(JSON.stringify({page, autonomousAuthorshipCredit: 0}, null, 2));
for (const prompt of ["Brainstorm ideas for a birthday party", "Let's roleplay: you are a medieval knight", "Let's roleplay: you are a ship captain", "Python में एक फ़ंक्शन लिखो जो n का फ़ैक्टोरियल लौटाए"]) {
  context.routePrompt = prompt;
  const diagnostics = evaluate(context, `({
    roleplay: roleplayRequest(routePrompt),
    brainstorm: brainstormRequestTopic(BRAINSTORM_SEEDS, normalizePrompt(routePrompt)),
    program: writeProgramParameters(routePrompt),
  })`);
  console.log(JSON.stringify({prompt, diagnostics, autonomousAuthorshipCredit: 0}));
}

const factorial = await host.solve('Python में एक फ़ंक्शन लिखो जो n का फ़ैक्टोरियल लौटाए');
console.log(JSON.stringify({factorial, autonomousAuthorshipCredit: 0}, null, 2));
