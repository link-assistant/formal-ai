// Documentary meaning is independent of lossless comment restoration and executable authority.
import { cached, readText } from '../agentic/host.mjs';
import { parseLinoRoot, findChildValue } from '../agentic/write_lino.mjs';
import { formalizeText, readWords, surfacesIn } from '../agentic/crate/text_formalization.mjs';
import { bestSurface } from '../agentic/crate/round_trip_translation.mjs';
import { meaningFiles, parseLexiconText, containsCjk } from '../agentic/crate/seed_meanings.mjs';
import { isWordBoundary } from '../agentic/crate/source_walk.mjs';
import { parseLinks, field, textOf } from '../logging/lino-codec.mjs';
import { wellFormedText } from '../logging/text-lines.mjs';
import { conceptRelationEndpoints } from './portable/concept-relations.js';
export const COMMENT_GRAMMAR_PATH = 'data/meta/source-model/comment-grammar.lino';
const CONSTRUCTS_PATH = 'data/meta/self-translation/constructs.lino';
const quoted = value => ({
  quoted: value
});
const value = (node, name) => {
  const found = field(node, name);
  return found ? textOf(found[1]) : '';
};
const words = (node, name) => (field(node, name) ?? [name]).slice(1).map(textOf);
const clean = text => text.trim().replace(/[.!?。！？]+$/u, '').trim();
const equal = (left, right) => JSON.stringify(left) === JSON.stringify(right);
function grammar() {
  return cached('source-comment-meaning-grammar', () => {
    const [root] = parseLinks(readText(COMMENT_GRAMMAR_PATH));
    if (root?.[0] !== 'comment-grammar') throw Error('CommentGrammarRequired');
    return root.slice(1).filter(Array.isArray);
  });
}
function observedDefinitions() {
  return cached('source-comment-definition-owners', () => {
    const records = new Map();
    for (const file of meaningFiles()) for (const item of parseLexiconText(readText(file))) {
      const rows = records.get(item.slug) ?? [];
      rows.push({
        file,
        item
      });
      records.set(item.slug, rows);
    }
    return records;
  });
}
function cueMatches(text, cue) {
  if (typeof cue !== 'string' || cue === '') return [];
  const escaped = Array.from(cue).map(character => '\\^$.*+?()[]{}|'.includes(character) ? '\\' + character : character).join('');
  const expression = new RegExp(escaped, 'giu');
  return [...text.matchAll(expression)].filter(match => containsCjk(cue) || (match.index === 0 || isWordBoundary(Array.from(text.slice(0, match.index)).at(-1))) && (match.index + match[0].length === text.length || isWordBoundary(text[match.index + match[0].length])));
}
function grammarWords(kind, language) {
  return grammar().filter(row => row[0] === kind).flatMap(row => words(row, language).map(surface => ({
    surface,
    meaning: value(row, 'meaning'),
    row
  })));
}
function takePrefix(text, candidates) {
  const found = candidates.flatMap(candidate => cueMatches(text, candidate.surface).filter(match => match.index === 0).map(match => ({
    ...candidate,
    rest: text.slice(match[0].length).trim(),
    phrase: match[0]
  })));
  return found.sort((left, right) => right.phrase.length - left.phrase.length)[0] ?? null;
}
function takeSuffix(text, candidates) {
  const found = candidates.flatMap(candidate => cueMatches(text, candidate.surface).filter(match => match.index + match[0].length === text.length).map(match => ({
    ...candidate,
    rest: text.slice(0, match.index).trim(),
    phrase: match[0]
  })));
  return found.sort((left, right) => right.phrase.length - left.phrase.length)[0] ?? null;
}
function issue(kind, phrase, role) {
  return ['semantic-gap', ['kind', kind], ['role', role], ['source-phrase', quoted(phrase)]];
}
function typeMeaning(surface, language) {
  const names = grammarWords('type-word', language).filter(row => row.surface.toLowerCase() === surface.toLowerCase()).map(row => row.meaning);
  if (names.length !== 1) return null;
  const type = parseLinks(readText(CONSTRUCTS_PATH)).find(row => row[0] === 'type' && row[1] === names[0]);
  return type ? ['type', names[0], ['defined-by', ...type.slice(2)], ['source', quoted(CONSTRUCTS_PATH)]] : null;
}
function atom(text, language, role, needs, definitions) {
  const surface = clean(text);
  const typed = typeMeaning(surface, language);
  if (typed) return typed;
  const terms = readWords(surface, language, false).filter(word => ['meaning', 'name', 'number', 'unknown'].includes(word.kind));
  if (terms.length !== 1 || terms[0].kind === 'unknown') {
    needs.push(issue('unresolved-concept', surface, role));
    return ['unresolved-concept', ['phrase', quoted(surface)]];
  }
  const term = terms[0];
  if (term.kind === 'number') return ['number', quoted(term.id)];
  if (term.kind === 'name') {
    needs.push(issue('unbound-name', surface, role));
    return ['unresolved-name', ['phrase', quoted(surface)]];
  }
  const owned = observedDefinitions().get(term.id) ?? [];
  if (!owned.length) {
    needs.push(issue('missing-concept-definition', surface, role));
    return ['unresolved-concept', ['phrase', quoted(surface)]];
  }
  const pending = [term.id],
    visited = new Set();
  while (pending.length && visited.size < 64) {
    const identity = pending.shift();
    if (visited.has(identity)) continue;
    visited.add(identity);
    const owners = observedDefinitions().get(identity) ?? [];
    if (!owners.length) {
      needs.push(issue('missing-definition-link', identity, role));
      continue;
    }
    for (const owner of owners) {
      const definition = ['definition', ['meaning', identity], ['defined-by', ...owner.item.defined_by.map(target => ['meaning', target])], ['roles', ...owner.item.roles], ['source', ['path', quoted(owner.file)], ['owner', ['meaning', identity]]]];
      if (!definitions.some(existing => equal(existing, definition))) definitions.push(definition);
      pending.push(...owner.item.defined_by.filter(target => !visited.has(target)));
    }
  }
  if (pending.length) needs.push(issue('definition-closure-limit', surface, role));
  return ['concept', term.id];
}
function termPhrase(text, language, role, needs, definitions, {
  classification = false
} = {}) {
  let rest = clean(text),
    quantifier = null;
  const prefix = takePrefix(rest, grammarWords('quantifier', language));
  if (prefix) {
    quantifier = prefix.meaning;
    rest = prefix.rest;
  } else {
    const determiner = takePrefix(rest, grammarWords('determiner', language));
    if (determiner) rest = determiner.rest;
  }
  if (classification) quantifier = null;
  return {
    node: atom(rest, language, role, needs, definitions),
    quantifier,
    surface: rest
  };
}
function conceptIdentity(node) {
  return node[0] === 'concept' ? String(node[1]) : node[0] === 'type' ? 'type:' + node[1] : '';
}
function bind(role, term, body) {
  return term.quantifier ? [term.quantifier, ['binding', ['role', role], ['sort', term.node]], body] : body;
}
function relationDefinitions(language) {
  const parsed = parseLinoRoot(readText('data/seed/formalization-relations.lino'));
  const root = (parsed.children ?? []).find(row => row.name === 'relations');
  const inherited = (root?.children ?? []).flatMap(row => (row.children ?? []).filter(item => item.name === 'lexeme' && item.value === language).flatMap(lexeme => (lexeme.children ?? []).filter(item => item.name === 'surface').map(surface => ({
    meaning: row.name.replaceAll('_', '-'),
    kind: findChildValue(row, 'kind'),
    cue: findChildValue(surface, 'text'),
    form: 'finite',
    row: null
  }))));
  const declared = grammar().filter(row => row[0] === 'predicate').flatMap(row => ['finite', 'base'].flatMap(form => words(row, language + '-' + form).map(cue => ({
    meaning: value(row, 'meaning'),
    kind: value(row, 'kind'),
    cue,
    form,
    row
  }))));
  const unique = new Map();
  for (const row of [...inherited, ...declared]) if (row.cue) unique.set(row.meaning + '|' + row.cue + '|' + row.form, row);
  return [...unique.values()];
}
function readClause(text, language, owner, needs, definitions) {
  const source = clean(text);
  const candidates = [];
  for (const predicate of relationDefinitions(language)) for (const match of cueMatches(source, predicate.cue)) {
    let before = source.slice(0, match.index).trim(),
      after = source.slice(match.index + match[0].length).trim();
    const localNeeds = [],
      localDefinitions = [],
      operators = [];
    let suffix;
    if (predicate.form === 'base') {
      while (suffix = takeSuffix(before, [...grammarWords('modal', language), ...grammarWords('negation', language)])) {
        operators.unshift(suffix);
        before = suffix.rest;
      }
      const auxiliary = takeSuffix(before, grammarWords('auxiliary', language));
      if (auxiliary) before = auxiliary.rest;
    }
    if (predicate.form === 'base' && !operators.length) continue;
    const outer = takePrefix(before, grammarWords('negation', language));
    if (outer) before = outer.rest;
    if (!after) continue;
    const impliedOwner = before === value(owner, 'name');
    const subject = before && !impliedOwner ? termPhrase(before, language, 'subject', localNeeds, localDefinitions) : {
      node: ['source-owner', owner],
      quantifier: null,
      surface: ''
    };
    const object = termPhrase(after, language, 'object', localNeeds, localDefinitions, {
      classification: predicate.meaning === 'is-a'
    });
    if (!before && !operators.length) continue;
    const left = subject.quantifier ? ['bound', 'subject'] : subject.node,
      right = object.quantifier ? ['bound', 'object'] : object.node;
    let body = [predicate.kind === 'effect' ? 'effect' : 'relation', ['predicate', predicate.meaning], ['subject', left], ['object', right]];
    body = bind('object', object, body);
    for (const operator of [...operators].reverse()) body = [operator.row[0] === 'negation' ? 'not' : 'modal', ...(operator.row[0] === 'negation' ? [] : [['mode', operator.meaning]]), body];
    body = bind('subject', subject, body);
    if (outer) body = ['not', body];
    const identities = [conceptIdentity(subject.node), conceptIdentity(object.node)];
    if (identities.every(Boolean) && before) {
      const observedLeft = cueMatches(before, subject.surface)[0]?.index ?? -1,
        observedRight = cueMatches(after, object.surface)[0]?.index ?? -1;
      const endpoints = conceptRelationEndpoints(identities, [observedLeft, -1], [-1, observedRight]);
      if (endpoints.$ !== 'grounded') localNeeds.push(issue('unresolved-relation-endpoints', source, 'proposition'));
    }
    if (predicate.row) localDefinitions.push(['predicate-definition', ['meaning', predicate.meaning], ['defined-by', ...(field(predicate.row, 'defined-by') ?? []).slice(1)], ['source', ['path', quoted(COMMENT_GRAMMAR_PATH)], ['owner', ['predicate', predicate.meaning]]]]);
    candidates.push({
      body,
      needs: localNeeds,
      definitions: localDefinitions,
      cueLength: match[0].length,
      source
    });
  }
  const best = candidates.filter(candidate => candidate.cueLength === Math.max(0, ...candidates.map(row => row.cueLength)));
  const distinct = [];
  for (const candidate of best) if (!distinct.some(existing => equal(existing.body, candidate.body))) distinct.push(candidate);
  if (distinct.length !== 1) {
    needs.push(issue(distinct.length ? 'ambiguous-comment-proposition' : 'unresolved-comment-proposition', source, 'proposition'));
    return ['unresolved-proposition'];
  }
  needs.push(...distinct[0].needs);
  for (const definition of distinct[0].definitions) if (!definitions.some(existing => equal(existing, definition))) definitions.push(definition);
  return distinct[0].body;
}
function commentBody(text) {
  if (text.startsWith('//')) return text.slice(text[2] === '/' || text[2] === '!' ? 3 : 2).trim();
  if (text.startsWith('/*') && text.endsWith('*/')) return text.slice(2, -2).split(/\r?\n/u).map(line => line.replace(/^\s*\* ?/u, '')).join('\n').trim();
  throw Error('SourceCommentDelimiterRequired');
}
function ownerLink(owner) {
  if (!owner || typeof owner !== 'object' || typeof owner.kind !== 'string' || typeof owner.name !== 'string') throw Error('StructuralCommentOwnerRequired');
  return [owner.kind, ['name', quoted(owner.name)]];
}
/** Exact bytes are restoration evidence; recursive propositions are separate documentary claims. */
export function formalizeComment(input, {
  language = 'en'
} = {}) {
  if (!input || typeof input !== 'object' || typeof input.text !== 'string' || !wellFormedText(input.text)) throw Error('Utf8CommentRequired');
  if (input.role !== undefined && typeof input.role !== 'string') throw Error('StructuralCommentRoleRequired');
  if (!input.source || typeof input.source.path !== 'string' || typeof input.source.language !== 'string') throw Error('StructuralSourceAttachmentRequired');
  const owner = ownerLink(input.owner),
    attachment = ['attachment', ['source', ['path', quoted(input.source.path)], ['language', quoted(input.source.language)]], ['owner', owner], ['role', input.role ?? 'documentation']];
  const needs = [],
    definitions = [],
    body = commentBody(input.text),
    citations = [];
  const prose = body.replace(/https?:\/\/[^\s<>]+/gu, raw => {
    const target = raw.replace(/[.,;]+$/u, '');
    try {
      new URL(target);
      citations.push(['citation', ['target', quoted(target)], ['authority', 'reference-only']]);
    } catch {
      needs.push(issue('invalid-citation', raw, 'citation'));
    }
    return '';
  }).replace(/\bSee\s*[.!]?$/iu, '').trim();
  const statements = formalizeText(prose, language);
  const clauses = statements.length ? statements.map(statement => statement.text) : prose ? [prose] : [];
  const propositions = clauses.map(clause => readClause(clause, language, owner, needs, definitions));
  return {
    schema: 'source-comment-meaning-v1',
    language,
    attachment,
    verbatim: input.text,
    meaning: ['comment-meaning', ['claims', 'documentary'], ['propositions', ...propositions], ['definitions', ...definitions], ['citations', ...citations]],
    needs
  };
}
/** Ordinary recursive LiNo metadata for a concrete comment node. */
export function commentMeaningLinks(model) {
  if (!validateCommentMeaning(model)) throw Error('CommentMeaningSourceMismatch');
  return ['meaning', ['schema', model.schema], model.attachment, model.meaning, ['needs', ...model.needs]];
}
function conceptSurface(node, source, target, needs) {
  if (node[0] === 'concept') {
    const surface = surfacesIn(String(node[1]), source)[0] ?? '';
    const rendered = bestSurface({
      kind: 'meaning',
      id: String(node[1]),
      surface
    }, source, target);
    if (rendered !== null) return rendered;
  }
  if (node[0] === 'type') {
    const word = grammarWords('type-word', target).find(row => row.meaning === node[1]);
    if (word) return word.surface;
  }
  needs.push(issue('unrenderable-concept', node[0] === 'unresolved-concept' ? textOf(field(node, 'phrase')?.[1] ?? quoted('')) : String(node[1] ?? node[0]), 'naturalization'));
  return null;
}
function renderProposition(node, source, target, needs) {
  const bindings = new Map(),
    operators = [];
  let current = node,
    outerNot = false;
  while (['universal', 'existential', 'modal', 'not'].includes(current[0])) {
    if (current[0] === 'universal' || current[0] === 'existential') {
      const binding = field(current, 'binding');
      const sort = field(binding ?? [], 'sort')?.[1];
      if (!Array.isArray(sort)) {
        needs.push(issue('invalid-binding', '', 'naturalization'));
        return null;
      }
      bindings.set(value(binding, 'role'), {
        quantifier: current[0],
        sort
      });
      current = current.at(-1);
    } else if (current[0] === 'modal') {
      operators.push({
        kind: 'modal',
        meaning: value(current, 'mode')
      });
      current = current.at(-1);
    } else {
      if (!bindings.size && !operators.length) outerNot = true;else operators.push({
        kind: 'not'
      });
      current = current.at(-1);
    }
  }
  if (!['relation', 'effect'].includes(current[0])) {
    needs.push(issue('unrenderable-proposition', '', 'naturalization'));
    return null;
  }
  const predicate = grammar().find(row => row[0] === 'predicate' && value(row, 'meaning') === value(current, 'predicate'));
  const layout = grammar().find(row => row[0] === 'layout' && value(row, 'language') === target);
  if (!predicate || !layout) {
    needs.push(issue('unsupported-rendering-language', target, 'naturalization'));
    return null;
  }
  const renderArgument = role => {
    const raw = field(current, role)?.[1];
    if (!Array.isArray(raw)) return null;
    if (raw[0] === 'source-owner') {
      const owner = raw[1];
      return Array.isArray(owner) ? textOf(field(owner, 'name')?.[1] ?? quoted('')) : null;
    }
    const binding = bindings.get(role),
      term = binding?.sort ?? raw;
    const surface = conceptSurface(term, source, target, needs);
    if (surface === null) return null;
    if (!binding) return surface;
    const quantifier = grammarWords('quantifier', target).find(row => row.meaning === binding.quantifier);
    if (!quantifier) {
      needs.push(issue('unrenderable-quantifier', binding.quantifier, role));
      return null;
    }
    return [quantifier.surface, surface].join(value(layout, 'join'));
  };
  const subject = renderArgument('subject'),
    object = renderArgument('object');
  if (subject === null || object === null) return null;
  const modal = operators.find(row => row.kind === 'modal'),
    negated = operators.some(row => row.kind === 'not') || outerNot && !bindings.size;
  if (outerNot && !bindings.size) outerNot = false;
  if (operators.filter(row => row.kind === 'modal').length > 1 || operators.filter(row => row.kind === 'not').length > 1) {
    needs.push(issue('unsupported-operator-scope', '', 'naturalization'));
    return null;
  }
  const verb = words(predicate, target + '-' + (modal || negated ? 'base' : 'finite'))[0];
  if (!verb) return null;
  const modalWord = modal ? grammarWords('modal', target).find(row => row.meaning === modal.meaning)?.surface : null;
  if (modal && !modalWord) return null;
  const negative = value(layout, 'negative'),
    auxiliary = negated && !modal ? value(layout, 'negative-auxiliary') : '';
  const before = [subject, modalWord, auxiliary, negated ? negative : null].filter(Boolean);
  const pieces = value(layout, 'order') === 'subject-object-predicate' ? [...before, object, verb] : [...before, verb, object];
  if (outerNot) pieces.unshift(negative);
  return pieces.join(value(layout, 'join'));
}
/** Render from meanings, then independently re-read the wording before claiming semantic survival. */
export function naturalizeComment(model, {
  language = 'en'
} = {}) {
  if (!validateCommentMeaning(model)) throw Error('CommentMeaningSourceMismatch');
  const needs = [...model.needs],
    propositions = field(model.meaning, 'propositions')?.slice(1) ?? [];
  const rendered = propositions.map(node => Array.isArray(node) ? renderProposition(node, model.language, language, needs) : null);
  if (rendered.some(text => text === null)) return {
    text: null,
    language,
    needs,
    roundTrip: {
      whole: false,
      parts: false
    }
  };
  const text = rendered.map(clause => clause + '.').join(' ');
  const source = field(model.attachment, 'source'),
    owner = field(model.attachment, 'owner')?.[1];
  if (!Array.isArray(owner)) throw Error('InvalidCommentAttachment');
  const input = {
    source: {
      path: value(source, 'path'),
      language: value(source, 'language')
    },
    owner: {
      kind: owner[0],
      name: value(owner, 'name')
    },
    role: value(model.attachment, 'role'),
    text: '// ' + text
  };
  const checked = formalizeComment(input, {
    language
  });
  const expected = field(model.meaning, 'propositions'),
    actual = field(checked.meaning, 'propositions');
  const whole = equal(expected, actual),
    parts = rendered.every((clause, index) => equal(field(formalizeComment({
      ...input,
      text: '// ' + clause + '.'
    }, {
      language
    }).meaning, 'propositions')?.[1], propositions[index]));
  if (!whole || !parts) needs.push(issue('semantic-round-trip-mismatch', text, 'naturalization'));
  return {
    text,
    language,
    needs,
    roundTrip: {
      whole,
      parts
    }
  };
}
/** Source restoration cannot authenticate a changed semantic claim. */
export function validateCommentMeaning(model) {
  const source = field(model.attachment, 'source'),
    owner = field(model.attachment, 'owner')?.[1];
  if (!Array.isArray(owner)) return false;
  const actual = formalizeComment({
    source: {
      path: value(source, 'path'),
      language: value(source, 'language')
    },
    owner: {
      kind: owner[0],
      name: value(owner, 'name')
    },
    role: value(model.attachment, 'role'),
    text: model.verbatim
  }, {
    language: model.language
  });
  return equal(actual.meaning, model.meaning) && equal(actual.needs, model.needs);
}

/** Canonical installed grammar; task metadata cannot substitute documentation syntax. */
export function parameterDocumentationGrammar(){
 const rows=grammar().filter(row=>row[0]==='parameter-documentation');
 if(rows.length!==1)throw Error('UniqueParameterDocumentationGrammarRequired');
 const row=rows[0],tags=words(row,'tags'),forbidden=words(row,'owner-overrides'),passive=words(row,'passive-tags');
 if(!tags.length||!forbidden.length||!passive.length||value(row,'syntax')!=='jsdoc-simple-param')throw Error('UnsupportedParameterDocumentationGrammar');
 return {network:structuredClone(row),tags,forbidden,passive};
}
function scanType(line,start){
 if(line[start]!=='{')return start;
 let depth=0,quote=null,escape=false;
 for(let index=start;index<line.length;index+=1){
  const character=line[index];
  if(quote!==null){if(escape)escape=false;else if(character==='\\')escape=true;else if(character===quote)quote=null;continue;}
  if(character==='`')throw Error('UnsupportedDocumentationTemplateType');
  if(character==='"'||character==="'"){quote=character;continue;}
  if(character==='{'){depth+=1;if(depth>32)throw Error('DocumentationTypeDepthRefused');}
  if(character==='}'){depth-=1;if(depth===0)return index+1;}
 }
 throw Error('UnbalancedDocumentationType');
}
/** Syntax binds a tag to an already observed owner; descriptions remain documentary unknowns. */
export function formalizeParameterDocumentation(input){
 const grammar=parameterDocumentationGrammar();
 if(!input.text.startsWith('/**')||!input.text.endsWith('*/'))throw Error('JSDocCommentRequired');
 const base=formalizeComment(input),bindings=[],needs=[];
 const parameterOccurrences=[...input.text.matchAll(/(?:^|[ \t])@([A-Za-z]+)\b/gu)].filter(match=>grammar.tags.includes(match[1]));
 for(const match of input.text.matchAll(/(?:^|[ \t])@([A-Za-z]+)\b/gu))if(!grammar.tags.includes(match[1])&&!grammar.passive.includes(match[1]))needs.push({kind:'unsupported-documentation-tag-attachment',tag:match[1]});
 const body=input.text.slice(3,-2),lines=body.match(/[^\r\n]*(?:\r\n|\n|\r|$)/gu).filter(Boolean);
 let offset=3;
 for(const rawLine of lines){
  const line=rawLine.replace(/(?:\r\n|\n|\r)$/u,'');
  const prefix=/^[ \t]*(?:\*[ \t]?)?/u.exec(line)[0];
  const opening=/^@([A-Za-z]+)\b/u.exec(line.slice(prefix.length));
  if(opening){
   const tag=opening[1];
   if(grammar.forbidden.includes(tag))needs.push({kind:'documentation-owner-override',tag});
   if(grammar.tags.includes(tag)){
    let cursor=prefix.length+opening[0].length,typeStart=null,typeEnd=null;
    if(!/[ \t]/u.test(line[cursor]??''))throw Error('ParameterTagSpacingRequired');
    while(/[ \t]/u.test(line[cursor]??''))cursor+=1;
    if(line[cursor]==='{'){typeStart=cursor;cursor=scanType(line,cursor);typeEnd=cursor;if(!/[ \t]/u.test(line[cursor]??''))throw Error('ParameterTagSpacingRequired');while(/[ \t]/u.test(line[cursor]??''))cursor+=1;}
    const name=/^[A-Za-z_$][A-Za-z0-9_$]*/u.exec(line.slice(cursor));
    if(!name||(!/^[ \t]*$/u.test(line.slice(cursor+name[0].length))&&!/^[ \t]/u.test(line.slice(cursor+name[0].length))))needs.push({kind:'unsupported-parameter-tag-name',tag});
    else bindings.push({tag,type:typeStart===null?null:line.slice(typeStart,typeEnd),binding:name[0],start:offset+cursor,end:offset+cursor+name[0].length,meaning:['parameter-reference',['owner',input.owner.kind,['name',quoted(input.owner.name)]],['binding',quoted(name[0])],['syntax',quoted(tag)]],descriptionMeaning:'not-certified'});
   }
  }
  offset+=rawLine.length;
 }
 if(parameterOccurrences.length!==bindings.length)needs.push({kind:'unclassified-parameter-tag'});
 const duplicated=bindings.filter((binding,index)=>bindings.findIndex(candidate=>candidate.binding===binding.binding)!==index);
 for(const duplicate of duplicated)needs.push({kind:'duplicate-parameter-tag',binding:duplicate.binding});
 return {schema:'source-comment-parameter-bindings/v1',base,grammar:grammar.network,bindings,needs};
}
export function validateParameterDocumentation(model){
 const grammar=parameterDocumentationGrammar();
 if(!validateCommentMeaning(model.base)||!equal(model.grammar,grammar.network))return false;
 const attachment=field(model.base.attachment,'source'),owner=field(model.base.attachment,'owner')?.[1];
 if(!owner)return false;
 const actual=formalizeParameterDocumentation({text:model.base.verbatim,source:{path:value(attachment,'path'),language:value(attachment,'language')},owner:{kind:owner[0],name:value(owner,'name')},role:value(model.base.attachment,'role')});
 return equal(actual,model);
}
/** Preserve all concrete segments except exactly proven name positions, then independently re-formalize. */
export function naturalizeParameterDocumentation(model,binding,target){
 if(!validateParameterDocumentation(model))throw Error('DocumentationMeaningPreimageMismatch');
 if(model.needs.length)throw Error('DocumentationBindingNeeds');
 if(!/^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(target)||target===binding)throw Error('UnsupportedDocumentationTarget');
 if(model.bindings.some(item=>(item.type?.match(/[A-Za-z_$][A-Za-z0-9_$]*/gu)??[]).includes(binding)))throw Error('DocumentationTypeReferenceNeeds');
 const selected=model.bindings.filter(item=>item.binding===binding);
 if(selected.length!==1||model.bindings.some(item=>item.binding===target))throw Error('AmbiguousDocumentationParameter');
 const item=selected[0],text=model.base.verbatim.slice(0,item.start)+target+model.base.verbatim.slice(item.end);
 const attachment=field(model.base.attachment,'source'),owner=field(model.base.attachment,'owner')[1];
 const checked=formalizeParameterDocumentation({text,source:{path:value(attachment,'path'),language:value(attachment,'language')},owner:{kind:owner[0],name:value(owner,'name')},role:value(model.base.attachment,'role')});
 const actual=checked.bindings.find(candidate=>candidate.binding===target);
 if(checked.needs.length||!actual||text.slice(0,actual.start)!==model.base.verbatim.slice(0,item.start)||text.slice(actual.end)!==model.base.verbatim.slice(item.end))throw Error('DocumentationResidualMismatch');
 const residual=model.base.verbatim.slice(0,item.start)+model.base.verbatim.slice(item.end);
 const needs=(residual.match(/[A-Za-z_$][A-Za-z0-9_$]*/gu)??[]).includes(binding)?[{kind:'unresolved-prose-parameter-reference',binding}]:[];
 return {text,model:checked,needs,claim:'parameter-binding-only'};
}
