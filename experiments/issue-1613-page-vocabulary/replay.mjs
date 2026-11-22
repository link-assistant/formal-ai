import { readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const checkout = resolve(process.argv[2] ?? process.cwd());
const workerHost = pathToFileURL(join(checkout, 'js/server/worker-host.mjs'));
const { createWorkerContext, evaluate } = await import(workerHost);
const corpus = await readFile(join(checkout, 'data/benchmarks/web-formalization/en.lino'), 'utf8');
const moon = corpus.split(/^web-formalization-page$/mu).find(section => section.includes('\n  title "Moon"\n'));
const paragraphs = [...moon.matchAll(/^  paragraph "(.*)"$/gmu)].map(match => match[1]);
const worker = createWorkerContext({ probeParagraphs: paragraphs });
await evaluate(worker, 'loadSeed()');

const report = evaluate(worker, `(() => {
  const { formalizePage, factSurvives, statementNotation } = crateModule(PAGE_FORMALIZATION_MODULE);
  const { clauseStatement, deformalizeStatement } = crateModule("crate/text_formalization.mjs");
  const html = "<html><body><h1>Moon</h1>"
    + probeParagraphs.map(paragraph => "<p>" + paragraph + "</p>").join("") + "</body></html>";
  const prose = pageProse(html, "https://en.wikipedia.org/wiki/Moon");
  const { sentences, ...counts } = formalizePage(prose, "en");
  const statements = sentences.flatMap(sentence => sentence.statements).map(statement => {
    const rendered = deformalizeStatement(statement, "en");
    return {
      text: statement.text,
      notation: statementNotation(statement),
      rendered,
      reparsed: statementNotation(clauseStatement(rendered, "en", null, false)),
      survived: factSurvives(statement),
    };
  });
  return { counts: { sentences: sentences.length, ...counts }, statements };
})()`);
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
