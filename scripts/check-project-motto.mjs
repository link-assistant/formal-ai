#!/usr/bin/env node
// Issue #1212: source publication contract. Released packages and repository
// Settings require separate publication evidence; this check makes no live claim.
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
export const MOTTO="Don't know everything, know just enough to get to know anything";
export const DOCUMENTS=['README.md','VISION.md','GOALS.md'];
export const PACKAGES=['desktop/package.json','vscode/package.json','packages/formal-ai-engine/package.json'];
export const PAGES=['js/index.html',...['app','docs','cli','vscode','download'].map(name=>`js/${name}/index.html`)];
export const FILES=[...DOCUMENTS,'rust/Cargo.toml',...PACKAGES,...PAGES,'docs/architect-notes/2026-10-15-know-just-enough.md'];
// SHA256 of the verbatim primary-issue quote line, including its > prefix.
const ARCHITECT_QUOTE_SHA256='e2e4f4be7f792109b8fa5cf8eaae546a1a512bd03bbc652969420d95536db9a9';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');

/** Inspect supplied source bytes so mutation controls need no project copy. */
export function checkProjectMotto(read=relative=>readFileSync(path.join(root,relative),'utf8')){
 const problems=[];
 const contents=new Map();
 for(const file of FILES){try{contents.set(file,read(file));}catch{problems.push(`${file}: source unavailable`);}}
 for(const file of DOCUMENTS){const text=contents.get(file);if(text!==undefined){
  const top=text.match(/^# [^\n]+\n\s*\n([^\n]+)/u)?.[1];
  if(top!==`**${MOTTO}**`)problems.push(`${file}: exact motto must follow the title`);
 }}
 const cargo=contents.get('rust/Cargo.toml');
 if(cargo!==undefined&&!cargo.split('\n').some(line=>line===`description = "${MOTTO}"`))problems.push('rust/Cargo.toml: package description must equal the motto');
 for(const file of PACKAGES){const text=contents.get(file);if(text!==undefined){try{
  if(JSON.parse(text).description!==MOTTO)problems.push(`${file}: package description must equal the motto`);
 }catch{problems.push(`${file}: invalid package JSON`);}}}
 for(const file of PAGES){const text=contents.get(file);if(text!==undefined){
  const descriptions=[...text.matchAll(/<meta\b[^>]*>/giu)].filter(match=>/\bname\s*=\s*["']description["']/iu.test(match[0]));
  if(!descriptions.some(match=>match[0].includes(`content="${MOTTO}"`)))problems.push(`${file}: description metadata must equal the motto`);
 }}
 const vision=contents.get('VISION.md');
 if(vision!==undefined){
  for(const sentence of vision.split(/[.!?\n]/u)){
   const banned=/\b(?:random(?:\s+or\s+evolutionary)?|evolutionary|exhaustive)\s+search\b/iu.exec(sentence);
   if(banned&&!/\b(?:no|not|never|avoid|reject|forbid)\b/iu.test(sentence.slice(0,banned.index)))problems.push('VISION.md: candidate selection must not prescribe random or exhaustive search');
  }
  if(!vision.includes('docs/architect-notes/2026-09-14-know-how-to-get-to-know-anything.md'))problems.push('VISION.md: retain the historical architect-note link');
 }
 const note=contents.get('docs/architect-notes/2026-10-15-know-just-enough.md');
 if(note!==undefined&&(!note.includes('https://github.com/link-assistant/formal-ai/issues/1212')||createHash('sha256').update(note.split('\n').find(line=>line.startsWith('> Also check that we set motto/slogan'))??'').digest('hex')!==ARCHITECT_QUOTE_SHA256))problems.push('architect note: retain the attributed primary quote');
 return problems;
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const problems=checkProjectMotto();for(const problem of problems)console.error(problem);
 console.log(`project motto: ${FILES.length} source surfaces, ${problems.length} problem(s); release and Settings acceptance remain separate`);
 if(problems.length)process.exitCode=1;
}
