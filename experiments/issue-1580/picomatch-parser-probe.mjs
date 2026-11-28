import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
if (!process.argv[2]) throw new Error('Supply the integrity-checked upstream package path');
const original=require(process.argv[2]);
const candidate=require('../../packages/picomatch-compat');
// Finite subjects (at most 12 characters) and 1,092 comparisons; no stress input.
const patterns=['+([*])','+([a*])','+([^*])','+([!]*])','+([]*])','+([^]*])',
 '+([[:alpha:]*])','+([[]*)','+("a*")','+(\\*)','+(ab)','+(*(a)|*(b))'];
const options=[{}, {posix:true}, {posix:false}, {literalBrackets:true},
 {literalBrackets:false}, {windows:true}, {unescape:true}];
const subjects=['a','b','aa','ab','***','[]','*',']','[a','a*a*','!*','[[','+([!]*])'];
const differences=[];
for (const pattern of patterns) for(const opts of options) {
 const before=original.makeRe(pattern,opts),after=candidate.makeRe(pattern,opts);
 for(const subject of subjects) {
  if(before.test(subject)!==after.test(subject))
   differences.push({pattern,options:opts,subject,before:before.source,after:after.source});
 }
}
console.log(JSON.stringify({comparisons:patterns.length*options.length*subjects.length,differences},null,2));
