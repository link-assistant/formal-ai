import {existsSync,readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const request=JSON.parse(readFileSync('/private/tmp/pr1188-ci-T4035/request.json','utf8'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
for(const item of request.changes){if(item.before_sha256===null?existsSync(item.path):hash(readFileSync(item.path))!==item.before_sha256)throw Error('preimage changed:'+item.path);if(hash(item.content)!==item.after_sha256)throw Error('payload changed');}
for(const item of request.changes){writeFileSync(item.path,item.content);if(hash(readFileSync(item.path))!==item.after_sha256)throw Error('effect mismatch');}
console.log(JSON.stringify({exact:true,repositoryEffects:request.changes.length,originalJob:request.original_job}));

console.log("PR1188_FETCH_PARSE_RESULT_CORRECTED");
