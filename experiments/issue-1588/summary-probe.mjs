import {execFileSync} from 'node:child_process';
import {installHost} from '../../js/agentic/host.mjs';
import {readRepoFile,parseLino} from '../../js/server/lino.mjs';
const selected = process.argv.slice(2);
installHost({parseLino,readText(file) {
 if (selected.includes(file) || (selected.includes('all') && file.startsWith('data/seed/')))
  return execFileSync('git',['show','HEAD:'+file],{encoding:'utf8',maxBuffer:8*1024*1024});
 return readRepoFile(file);
}});
const {measureDependencySummarization}=await import('../../scripts/measure-dependency-summarization.mjs');
const result=await measureDependencySummarization();
console.log(JSON.stringify(result));
