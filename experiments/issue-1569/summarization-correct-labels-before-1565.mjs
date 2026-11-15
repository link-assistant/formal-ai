// Issue #1569: run from a checkout of a6b3d3180 (the parent of #1565) to show the
// pre-#1565 summarizer, given the correct English labels for issues 103 and 279,
// already measures 425/778 issue key facts (0.546).
import path from 'node:path';
import { pathToFileURL } from 'node:url';
const root = process.cwd();
const m = await import(pathToFileURL(path.join(root, 'scripts/measure-dependency-summarization.mjs')).href);
const docs = await m.readDocuments();
const { keyFacts, retainedFacts, summarizeByDependency } = await import(pathToFileURL(path.join(root, 'js/agentic/crate/dependency_summarization.mjs')).href);
let f=0,r=0;
for (const d of docs) { if (d.kind!=='issues') continue; if (/issue-(103|279)\.md$/.test(d.name)) d.language='en';
  const s=summarizeByDependency(d.text,d.language); const k=keyFacts(d.text,d.language); f+=k.length; r+=retainedFacts(s,k); }
console.log('issues retained', r, '/', f, Math.floor(r/f*1000)/1000);
