// Structural clone candidates. A clean candidate list is never a semantic proof.
import {createHash} from 'node:crypto';
const ignored = new Set(['start','end','loc','range','extra','leadingComments','trailingComments','innerComments','comments','tokens','errors','typeAnnotation','returnType','typeParameters','typeArguments','accessibility','abstract','declare','readonly','definite']);
const erased = new Set(['TSAsExpression','TSTypeAssertion','TSNonNullExpression','TSInstantiationExpression']);
const admittedProofs = new WeakSet();
const byteHash = value => createHash('sha256').update(value).digest('hex');
const hash = value => createHash('sha256').update(JSON.stringify(value)).digest('hex');

// The parser and scope resolver are injected so tests and callers use the same
// maintained language parser. Unsupported input is a gap, never an empty result.
function syntaxPath(path) {
  const parts=[];
  for(let cursor=path;cursor.parentPath;cursor=cursor.parentPath)parts.unshift(cursor.listKey ? cursor.listKey+'['+cursor.key+']' : String(cursor.key));
  return parts.join('.');
}
export function structuralCallables(source, filename, {parse, traverse}) {
  const typescript = /\.(?:ts|mts|cts|tsx)$/u.test(filename);
  const ast = parse(source, {sourceType:'unambiguous', errorRecovery:false,
    plugins:[...(typescript ? ['typescript'] : []), ...(/\.[jt]sx$/u.test(filename) ? ['jsx'] : [])]});
  const callables = [];
  traverse(ast, {Function(functionPath) {
    const node = functionPath.node;
    const bindings = new Map();
    const nodes = new WeakMap();
    // Only bindings declared within this callable may be alpha-renamed.
    // Captured bindings and global/property identifiers retain their identity.
    functionPath.traverse({Identifier(identifierPath) {
      if (identifierPath.node === node.id) return;
      if (!identifierPath.isBindingIdentifier() && !identifierPath.isReferencedIdentifier()) return;
      const binding = identifierPath.scope.getBinding(identifierPath.node.name);
      if (!binding || binding.identifier.start < node.start || binding.identifier.end > node.end) return;
      if (!bindings.has(binding)) bindings.set(binding, 'local:' + bindings.size);
      nodes.set(identifierPath.node, bindings.get(binding));
    }});
    const canonical = value => {
      if (Array.isArray(value)) return value.map(canonical);
      if (value === null || typeof value !== 'object') return value;
      if (erased.has(value.type)) return canonical(value.expression);
      if (value.type === 'Identifier' && nodes.has(value)) return {type:'Identifier', name:nodes.get(value)};
      const result = {};
      for (const key of Object.keys(value).sort()) {
        if (ignored.has(key) || value[key] === undefined) continue;
        result[key] = canonical(value[key]);
      }
      return result;
    };
    const shape = canonical({params:node.params, body:node.body, async:Boolean(node.async), generator:Boolean(node.generator)});
    callables.push({name:node.id?.name ?? node.key?.name ?? functionPath.parentPath.node.id?.name ?? '<anonymous>',
      line:node.loc.start.line, start:node.start, end:node.end, syntaxPath:syntaxPath(functionPath), fingerprint:hash(shape)});
  }});
  return callables;
}

export function groupClones(records) {
  const groups = new Map();
  for (const record of records) {
    if (!groups.has(record.fingerprint)) groups.set(record.fingerprint, []);
    groups.get(record.fingerprint).push(record);
  }
  return [...groups.entries()].filter(([,sites]) => sites.length > 1)
    .map(([fingerprint,sites]) => ({fingerprint,sites}))
    .sort((a,b) => b.sites.length - a.sites.length || a.fingerprint.localeCompare(b.fingerprint));
}

// Strict acceptance must account for every implementation input and every
// observed clone. There is no baseline or generated-file exclusion mechanism.
export function assessDuplication({files,parsedFiles,clones,gaps}) {
  const parsed = new Set(parsedFiles);
  const unparsed = files.filter(file => !parsed.has(file));
  return {accepted:clones.length === 0 && gaps.length === 0 && unparsed.length === 0,
    cloneGroups:clones.length, gaps:[...gaps,...unparsed.map(file => ({file,reason:'unparsed implementation source'}))]};
}

// Proofs are recomputed from source and output bytes, never admitted from headers.
export function proveDerivedCallables({sourcePath,targetPath,source,target,sourceCallables,targetCallables,translate,tokenize,generator}) {
 const refuse=reason=>({accepted:false,reason,source:sourcePath,target:targetPath});
 if(target!==translate(source))return refuse('target drift from exact maintained regeneration');
 const sourceSha256=byteHash(source),targetSha256=byteHash(target);
 const seenSource=new Set(),seenTarget=new Set(),nodes=[];
 for(const row of sourceCallables){const key=row.start+':'+row.end;if(seenSource.has(key))return refuse('ambiguous source span mapping');seenSource.add(key);}
 for(const row of targetCallables){const key=row.start+':'+row.end;if(seenTarget.has(key))return refuse('ambiguous target span mapping');seenTarget.add(key);}
 if(sourceCallables.length!==targetCallables.length)return refuse('unmapped target callable');
 for(const row of sourceCallables){
  const targets=targetCallables.filter(candidate=>candidate.start===row.start&&candidate.end===row.end&&candidate.syntaxPath===row.syntaxPath);
  if(targets.length!==1||targets[0].fingerprint!==row.fingerprint)return refuse('missing or ambiguous canonical callable mapping');
  const canonicalNode={source:sourcePath,sourceSha256,syntaxPath:row.syntaxPath,start:row.start,end:row.end,metaTreeSha256:hash(tokenize(source.slice(row.start,row.end)))};
  nodes.push({canonicalId:hash(canonicalNode),canonicalNode,sourceSpan:[row.start,row.end],targetSpan:[targets[0].start,targets[0].end]});
 }
 const proof={accepted:true,source:sourcePath,target:targetPath,sourceSha256,targetSha256,generator:structuredClone(generator),nodes};
 const freeze=value=>{if(value&&typeof value==='object'){for(const child of Object.values(value))freeze(child);Object.freeze(value);}return value;};
 freeze(proof);admittedProofs.add(proof);return proof;
}
export function canonicalCloneGroups(rawGroups,proofs) {
 const parents=new Map(),key=(file,start,end)=>file+':'+start+':'+end;
 const find=site=>{if(!parents.has(site))parents.set(site,site);if(parents.get(site)!==site)parents.set(site,find(parents.get(site)));return parents.get(site);};
 const union=(a,b)=>{a=find(a);b=find(b);if(a!==b)parents.set(a<b?b:a,a<b?a:b);};
 for(const proof of proofs){if(!admittedProofs.has(proof))continue;for(const node of proof.nodes)union(key(proof.source,...node.sourceSpan),key(proof.target,...node.targetSpan));}
 const violations=[];
 for(const group of rawGroups){const definitions=new Map();for(const site of group.sites){const identity='definition:'+find(key(site.file,site.start,site.end));if(!definitions.has(identity))definitions.set(identity,[]);definitions.get(identity).push(site);}
 if(definitions.size>1)violations.push({...group,independentDefinitionCount:definitions.size,definitions:[...definitions].map(([identity,sites])=>({identity,sites}))});}
 return violations;
}

// A factory introduces one wrapper, then preserves the source callables' token
// trees and order. Only exact regeneration plus that unique correspondence grants
// identity. The outer wrapper remains a scanned, unmapped implementation node.
export function proveFactoryCallables({sourcePath,targetPath,source,target,sourceCallables,targetCallables,regenerate,tokenize,generator}) {
 const refuse=reason=>({accepted:false,reason,source:sourcePath,target:targetPath});
 if(target!==regenerate(source))return refuse('factory drift from exact maintained regeneration');
 const unique=rows=>new Set(rows.map(row=>row.start+':'+row.end)).size===rows.length;
 if(!unique(sourceCallables)||!unique(targetCallables))return refuse('ambiguous factory callable span mapping');
 const wrappers=targetCallables.filter(row=>row.syntaxPath==='body[0].expression.right');
 if(wrappers.length!==1)return refuse('missing or ambiguous generator-owned outer factory');
 const inner=targetCallables.filter(row=>row!==wrappers[0]);
 if(inner.length!==sourceCallables.length)return refuse('unmapped factory callable');
 const sorted=rows=>[...rows].sort((a,b)=>a.start-b.start||b.end-a.end);
 const from=sorted(sourceCallables),to=sorted(inner),nodes=[];
 for(let index=0;index<from.length;index++){
  const a=from[index],b=to[index],tree=tokenize(source.slice(a.start,a.end));
  if(a.fingerprint!==b.fingerprint||JSON.stringify(tree)!==JSON.stringify(tokenize(target.slice(b.start,b.end))))return refuse('factory callable correspondence differs');
  const canonicalNode={source:sourcePath,sourceSha256:byteHash(source),syntaxPath:a.syntaxPath,start:a.start,end:a.end,metaTreeSha256:hash(tree)};
  nodes.push({canonicalId:hash(canonicalNode),canonicalNode,sourceSpan:[a.start,a.end],targetSpan:[b.start,b.end]});
 }
 const proof={accepted:true,kind:'worker_factory',source:sourcePath,target:targetPath,sourceSha256:byteHash(source),targetSha256:byteHash(target),generator:structuredClone(generator),nodes};
 const freeze=value=>{if(value&&typeof value==='object'){for(const child of Object.values(value))freeze(child);Object.freeze(value);}return value;};freeze(proof);admittedProofs.add(proof);return proof;
}
