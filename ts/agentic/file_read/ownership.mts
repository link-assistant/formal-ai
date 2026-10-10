import { childrenNamed, parseLino, readText } from '../host.mjs';
// Source-owned Read operands and policies; twin: rust/src/agentic_coding/file_read/ownership.rs.
import { containsCjk } from '../crate/coding_catalog.mjs';
import { observedCallableRequest } from '../module_function.mjs';
import { Capability } from '../capability.mjs';
import { classifyTool } from '../capability_router.mjs';
import { quotedSegmentSpans, quoteFault } from '../crate/normal_markov.mjs';
import { callerContextVocabulary, policyLeadClause } from '../crate/seed_caller_context.mjs';
import { mentionsRole, wordsForRole } from '../crate/seed_meanings.mjs';
import { isAlphanumeric, isWhitespace, trim, trimStart } from '../crate/rust_str.mjs';
import { tokens } from '../write_request.mjs';
import { cleanFileToken, looksLikeLocalFilePath } from '../file_read.mjs';
import { sentences } from '../shell_command_policy.mjs';
import { samePath } from './records.mjs';
const FUNCTION_ROLES = ['request_function_word', 'enumeration_cue'];
const OBJECT_ROLE = 'file-read-object-noun';
const READ_ROLE = 'file_read_action_cue';
const escapePattern = value => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
function roleSpans(text, roles) {
  const quoted = quotedSegmentSpans(text);
  const result = [];
  for (const role of roles) for (const surface of wordsForRole(role)) {
    if (!surface || surface.includes('…')) continue;
    for (const match of text.matchAll(new RegExp(escapePattern(surface), 'giu'))) {
      const start = match.index, end = start + match[0].length;
      const before = Array.from(text.slice(0, start)).pop();
      const after = Array.from(text.slice(end))[0];
      if (!containsCjk(surface) && ((before && (isAlphanumeric(before) || before === '_'))
        || (after && (isAlphanumeric(after) || after === '_')))) continue;
      if (quoted.some(span => start >= span.start && end <= span.end)) continue;
      result.push({start, end});
    }
  }
  return result;
}
function covered(text, roles, explicit = []) {
  const remaining = text.split('');
  const spans = roleSpans(text, roles);
  for (const surface of explicit) {
    const expression = new RegExp(escapePattern(surface), 'giu');
    for (const match of text.matchAll(expression)) spans.push({start:match.index,end:match.index+match[0].length});
  }
  for (const span of spans) for (let index=span.start;index<span.end;index+=1) remaining[index]=' ';
  return Array.from(remaining.join('')).every(character => isWhitespace(character) || ',;:.!?"\'`。，；！？।'.includes(character));
}
function ownershipLocalFilePath(path) {
  const normalized = Array.from(path, character => character.codePointAt(0) > 127 && isAlphanumeric(character) ? 'a' : character).join('');
  return looksLikeLocalFilePath(path) || (normalized !== path && looksLikeLocalFilePath(normalized));
}
function pathSpans(text) {
  return tokens(text).flatMap(token => {
    const path=cleanFileToken(token.text);
    if (!ownershipLocalFilePath(path)) return [];
    const relative=token.text.indexOf(path);
    return relative<0?[]:[{path,start:token.start+relative,end:token.start+relative+path.length}];
  });
}
function policyStarts(text) {
  const lower=trimStart(text).toLowerCase();
  return policyLeadClause(lower)!==null || callerContextVocabulary().policy_leads.some(lead=>containsCjk(lead)&&lower.startsWith(lead));
}
function pendingReadCondition(prompt,role=READ_ROLE) {
  const sentences = instructionSentenceTexts(prompt);
  const unresolved = sentences.some(sentence => pathSpans(sentence).some(path => {
    const operandEnd = quotedSegmentSpans(sentence)
      .filter(span => span.text === path.path && path.start >= span.start && path.end <= span.end)
      .reduce((end, span) => Math.max(end, span.end), path.end);
    const tail = sentence.slice(operandEnd);
    return policyStarts(tail)
      && boundReadPaths(sentence.slice(0,operandEnd),role).includes(path.path)
      && !readPrecedesOwnedAuthoring(prompt,tail);
  }));
  if (unresolved) return true;
  if (boundReadPaths(prompt,role).length === 0) return false;
  return sentences.some(sentence => {
    const bound = boundReadPaths(sentence,role);
    if (bound.length > 0) return false;
    const sequence = roleSpans(sentence,['enumeration_cue','file_edit_joiner_cue']);
    return sequence.some(span => covered(sentence.slice(0,span.start),FUNCTION_ROLES)
      && !covered(sentence.slice(span.end),FUNCTION_ROLES));
  });
}
function negativeReadObjects(prompt) {
  const result=[];
  for (const sentence of instructionSentenceTexts(prompt)) for (const action of roleSpans(sentence,[READ_ROLE])) {
    const prefix=sentence.slice(0,action.start).toLowerCase();
    const rest=policyLeadClause(prefix);
    if (rest===null || !mentionsRole('statement_negation_cue',prefix) || !covered(rest,FUNCTION_ROLES)) continue;
    const objects=[...pathSpans(sentence).map(span=>({...span,scope:span.path})),
      ...roleSpans(sentence,[OBJECT_ROLE]).map(span=>({...span,scope:null}))];
    for (const object of objects) {
      if (object.start<action.end || !covered(sentence.slice(action.end,object.start),FUNCTION_ROLES)) continue;
      result.push(object.scope);
    }
  }
  for (const sentence of instructionSentenceTexts(prompt)) for (const action of roleSpans(sentence,['file-read-negative-operation'])) {
    const objects=[...pathSpans(sentence).map(span=>({...span,scope:span.path})),
      ...roleSpans(sentence,[OBJECT_ROLE]).map(span=>({...span,scope:null}))];
    for (const object of objects) {
      const prefix=sentence.slice(0,Math.min(action.start,object.start));
      const gap=object.start>=action.end?sentence.slice(action.end,object.start):object.end<=action.start?sentence.slice(object.end,action.start):null;
      if (gap!==null && (covered(prefix,[...FUNCTION_ROLES,'statement_negation_cue'])||policyStarts(prefix))
        && covered(gap,FUNCTION_ROLES)) result.push(object.scope);
    }
  }
  return result;
}
function unboundEffectOperation(prompt) {
  return instructionSentenceTexts(prompt).some(sentence => {
    const paths=pathSpans(sentence), bound=boundReadPaths(sentence,READ_ROLE);
    return roleSpans(sentence,['file-read-independent-effect-operation']).some(action => {
      if(paths.some(path=>action.start>=path.start&&action.end<=path.end))return false;
      if(bound.length>0 && roleSpans(sentence,[READ_ROLE]).some(span=>action.start>=span.start&&action.end<=span.end))return false;
      const prefix=sentence.slice(0,action.start).split('');
      if(bound.length>0) for(const span of [...paths.filter(path=>bound.includes(path.path)),...roleSpans(sentence,[READ_ROLE])]) {
        if(span.end<=action.start)for(let index=span.start;index<span.end;index+=1)prefix[index]=' ';
      }
      return covered(prefix.join(''),FUNCTION_ROLES);
    });
  });
}
function conflicts(scopes,paths) {
  return scopes.some(scope => scope===null || paths.some(path=>samePath(path,scope)||samePath(scope,path)));
}
/** Mirrors `fn bound_read_paths`: structural operands; callers still preflight every immutable Need. */
export function boundReadPaths(prompt,role) {
  if (role === READ_ROLE) {
    const modePaths = modePathsForClause(prompt);
    if (modePaths !== null) return conflicts(negativeReadObjects(prompt),modePaths) ? [] : modePaths;
  }
  const result=[];
  for (const sentence of instructionSentenceTexts(prompt)) {
    const owned=[];
    for (const object of pathSpans(sentence)) {
      for (const action of roleSpans(sentence,[role])) {
        const before=sentence.slice(0,Math.min(action.start,object.start)).split('');
        for (const prior of owned) if (prior.end<=before.length)
          for(let index=prior.start;index<prior.end;index+=1) before[index]=' ';
        const questionWords=role==='module_export_question'?callerContextVocabulary().question_words:[];
        const prefixRoles=role==='module_export_question'
          ? [...FUNCTION_ROLES,READ_ROLE,'coding_declaration_noun','translation_stop_word']:FUNCTION_ROLES;
        if (!covered(before.join(''),[...prefixRoles,'social_greeting'],questionWords)) continue;
        if (object.start>=action.end) {
          const gap=sentence.slice(action.end,object.start).split('');
          for(const prior of owned) if(prior.pathStart>=action.end && prior.pathEnd<=object.start)
            for(let index=prior.pathStart-action.end;index<prior.pathEnd-action.end;index+=1)gap[index]=' ';
          const gapRoles=role==='module_export_question'
            ? [...FUNCTION_ROLES,OBJECT_ROLE,'coding_declaration_noun','translation_stop_word']: [...FUNCTION_ROLES,OBJECT_ROLE];
          if(!covered(gap.join(''),gapRoles))continue;
        } else if (object.end<=action.start) {
          if(role==='module_export_question' && !questionWords.some(word=>sentence.toLowerCase().startsWith(word)))continue;
          if(!covered(sentence.slice(object.end,action.start),prefixRoles))continue;
          if(!covered(sentence.slice(action.end),[...prefixRoles,role]))continue;
        } else continue;
        owned.push({start:Math.min(action.start,object.start),end:Math.max(action.end,object.end),pathStart:object.start,pathEnd:object.end});
        if(!result.includes(object.path))result.push(object.path);
        break;
      }
    }
    if (owned.length>0 && owned.length!==pathSpans(sentence).length) return [];
  }
  return conflicts(negativeReadObjects(prompt),result)?[]:result;
}
/** Mirrors `fn owned_read_paths` in rust/src/agentic_coding/file_read/ownership.rs. */
export function ownedReadPaths(prompt,role) {
  const paths=boundReadPaths(prompt,role);
  return unboundEffectOperation(prompt)||pendingReadCondition(prompt,role)?[]:paths;
}
/** Refuse planned file reads whose immutable policy scope is violated or unproved. */
export function readPolicyBlocksPlan(prompt,plan) {
  const scopes=negativeReadObjects(prompt);
  const unboundOperation=unboundEffectOperation(prompt)||pendingReadCondition(prompt);
  if(scopes.length===0&&!unboundOperation)return false;
  for(const call of plan.calls??[]) {
    const capability=classifyTool(call.tool);
    if(scopes.length>0&&[Capability.Run,Capability.Grep,Capability.ReadMany].includes(capability))return true;
    if(capability!==Capability.Read)continue;
    if(unboundOperation)return true;
    let argumentsValue;try{argumentsValue=JSON.parse(call.arguments);}catch{return true;}
    if(argumentsValue===null||typeof argumentsValue!=='object'||Array.isArray(argumentsValue))return true;
    const fields=['path','filePath','file_path'].filter(key=>Object.hasOwn(argumentsValue,key)).map(key=>argumentsValue[key]);
    if(fields.length===0||fields.some(field=>typeof field!=='string'||trim(field)===''||field!==fields[0]))return true;
    if(conflicts(scopes,[fields[0]]))return true;
  }
  return false;
}

function modePathsForClause(prompt) {
  const root = parseLino(readText('data/seed/meanings-file-write.lino'));
  const contract = childrenNamed(root, 'file-read-mode-contract')[0];
  if (!contract) return null;
  for (const form of childrenNamed(contract, 'form')) {
    const template = childrenNamed(form, 'pattern')[0]?.value;
    if (!template) continue;
    let missing = false;
    const pattern = template.replace(/\{([a-z]+(?:-[a-z]+)*)\}/gu, (_, name) => {
      if (name === 'source' || name === 'sources') return `(?<${name}>.+?)`;
      const exact = wordsForRole(name);
      const legacy = wordsForRole(name.replaceAll('-', '_'));
      if (exact.length > 0 && legacy.length > 0 && JSON.stringify(exact) !== JSON.stringify(legacy)) { missing = true; return ''; }
      const surfaces = exact.length > 0 ? exact : legacy;
      if (surfaces.length === 0) { missing = true; return ''; }
      return '(?:'+surfaces.map(escapePattern).join('|')+')';
    });
    if (missing) continue;
    let match;
    try { match = new RegExp(pattern, 'diu').exec(prompt); } catch { continue; }
    if (!match || match[0].length !== prompt.length) continue;
    const domains = childrenNamed(form, 'capture-domain');
    if (!domains.every(domain => {
      const value = match.groups[domain.value];
      const domainPattern = childrenNamed(domain, 'pattern')[0]?.value;
      if (value === undefined || !domainPattern) return false;
      try {
        const observed = new RegExp(domainPattern, 'u').exec(value);
        return observed !== null && observed[0].length === value.length;
      } catch { return false; }
    })) continue;
    const source = match.groups.source ?? match.groups.sources;
    if (!source || quoteFault(source) !== null) continue;
    const paths = pathSpans(source);
    if (paths.length === 0 || match.groups.source !== undefined && paths.length !== 1) continue;
    const remaining = source.split('');
    for (const path of paths) for (let index=path.start; index<path.end; index+=1) remaining[index]=' ';
    if (!covered(remaining.join(''), [...FUNCTION_ROLES,OBJECT_ROLE,'file-read-mode-joiner'])) continue;
    return [...new Set(paths.map(path => path.path))];
  }
  return null;
}

/** Paired literal quotations keep global authority before clause splitting. */
function instructionSentenceTexts(prompt) {
  const characters = prompt.split('');
  for (const span of quotedSegmentSpans(prompt)) {
    const path = cleanFileToken(span.text);
    if (path === span.text && ownershipLocalFilePath(path)) continue;
    for (let index=span.start; index<span.end; index+=1) characters[index]=' ';
    characters[span.end-1]='.';
  }
  const text = characters.join('');
  const protectedCharacters = text.split('');
  for (const span of pathSpans(text)) for (let index=span.start;index<span.end;index+=1)
    if (protectedCharacters[index] === '.') protectedCharacters[index] = '_';
  return sentences(protectedCharacters.join('')).map(sentence => trim(text.slice(sentence.span.start,sentence.span.end)));
}


/** A declared source goal can require Read before its deferred authoring. */
function readPrecedesOwnedAuthoring(prompt, tail) {
  if (observedCallableRequest(prompt) === null) return false;
  const root = parseLino(readText('data/seed/meanings-file-write.lino'));
  const contract = childrenNamed(root, 'file-read-order-contract')[0];
  if (!contract || childrenNamed(contract, 'requires-goal')[0]?.value !== 'observed-callable-authoring') return false;
  return childrenNamed(contract, 'form').some(form => {
    const pattern = childrenNamed(form, 'pattern')[0]?.value;
    if (!pattern) return false;
    try {
      const match = new RegExp(pattern, 'iu').exec(tail);
      return match !== null && match[0].length === tail.length;
    } catch { return false; }
  });
}
