// A deliberately finite checker for literal declarations; it makes no integration-role inference.
import {createHash} from 'node:crypto';
const sha=value=>createHash('sha256').update(value).digest('hex');
export function literalManifestFacts(files){
  const facts=[];
  for(const file of files){
    if(file.path.endsWith('.json')){
      let value;try{value=JSON.parse(file.content);}catch{continue;}
      for(const section of ['dependencies','devDependencies','optionalDependencies']){
        if(!value[section]||typeof value[section]!=='object'||Array.isArray(value[section]))continue;
        for(const [name,version] of Object.entries(value[section]))if(typeof version==='string')
          facts.push({kind:'manifest-declaration',manifest:file.path,sourceIdentity:sha(file.content),section,name,version});
      }
      continue;
    }
    if(!file.path.endsWith('.toml'))continue;
    let section='';
    for(const line of file.content.split(/\r?\n/u)){
      const header=/^\s*\[([^\n]+)\]\s*(?:#.*)?$/u.exec(line);if(header){section=header[1];continue;}
      if(section!=='dependencies'&&!section.endsWith('.dependencies'))continue;
      const declaration=/^\s*([A-Za-z0-9_-]+)\s*=\s*("(?:[^"\\]|\\.)*"|\{[^\n]+\})\s*(?:#.*)?$/u.exec(line);
      if(!declaration)continue;
      const [,name,literal]=declaration;
      let version=null,options={};
      if(literal.startsWith('"')){try{version=JSON.parse(literal);}catch{continue;}}
      else{
        // Accept only flat literal Cargo tables used by these frozen snapshots; unsupported syntax stays unknown.
        const versionToken=/\bversion\s*=\s*("(?:[^"\\]|\\.)*")/u.exec(literal);
        if(!versionToken)continue;
        try{version=JSON.parse(versionToken[1]);}catch{continue;}
        for(const key of ['optional','default-features']){
          const match=new RegExp('(?:^|[, {])'+key+'\\s*=\\s*(true|false)(?=[, }])','u').exec(literal);
          if(match)options[key]=match[1]==='true';
        }
        const features=/\bfeatures\s*=\s*(\[(?:\s*"[^"\\]*"\s*,?)*\])/u.exec(literal);
        if(features){try{options.features=JSON.parse(features[1]);}catch{continue;}}
      }
      facts.push({kind:'manifest-declaration',manifest:file.path,sourceIdentity:sha(file.content),section,name,version,...options});
    }
  }
  return facts;
}
export function supportedManifestClaims(markdown,facts){
  const checked=[];
  const blocks=[...markdown.matchAll(/```formal-ai-source-claim\s*\n([\s\S]*?)\n```/gu)];
  if(blocks.length>32)return [{status:'unresolved',reason:'claim-budget-exceeded'}];
  for(const block of blocks){
    let claim;try{claim=JSON.parse(block[1]);}catch{checked.push({status:'unresolved',reason:'invalid-claim-json'});continue;}
    const fields=claim&&typeof claim==='object'&&!Array.isArray(claim)?Object.keys(claim):[];
    const allowed=['kind','manifest','sourceIdentity','section','name','version','optional','default-features','features'];
    const required=['kind','manifest','sourceIdentity','section','name','version'];
    const fact=fields.length?facts.find(row=>row.manifest===claim.manifest&&row.section===claim.section&&row.name===claim.name):null;
    const valid=required.every(key=>fields.includes(key))&&fields.every(key=>allowed.includes(key))&&claim.kind==='manifest-declaration'&&fact
      &&fields.every(key=>Object.hasOwn(fact,key)&&JSON.stringify(fact[key])===JSON.stringify(claim[key]));
    checked.push({claim,status:valid?'verified-literal-declaration':'unresolved',
      semantics:'only exact manifest fields; consumer behavior and integration roles are not verified'});
  }
  return checked;
}
