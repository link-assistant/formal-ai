import test from 'node:test';
import assert from 'node:assert/strict';
import { installTextHost } from '../../../scripts/lib/text-capability-measures.mjs';
import { formalizeComment, naturalizeComment, commentMeaningLinks, validateCommentMeaning } from '../../../js/meta/comment-meaning.js';
import { field } from '../../../js/logging/lino-codec.mjs';
import { conceptRelationEndpoints } from '../../../js/meta/portable/concept-relations.js';
installTextHost();
const input = (text, language = 'javascript') => ({
  source: {
    path: 'src/held-out.' + (language === 'rust' ? 'rs' : language === 'typescript' ? 'ts' : 'js'),
    language
  },
  owner: {
    kind: 'function',
    name: 'decodeFrame'
  },
  role: 'contract',
  text
});
const formal = text => formalizeComment(input('// ' + text));
const propositions = model => field(model.meaning, 'propositions').slice(1);
const walk = (node, predicate) => Array.isArray(node) && (predicate(node) || node.some(child => walk(child, predicate)));
const has = (model, head) => walk(model.meaning, node => node[0] === head);
for (const language of ['javascript', 'typescript', 'rust']) test(language + ' exact comment bytes and structural owner remain separate from semantics', () => {
  const source = language === 'rust' ? '//! The result is a string.\r\n' : '/* The result is a string.\n * 保留 exact layout. */';
  const model = formalizeComment(input(source, language));
  assert.equal(model.verbatim, source);
  assert.deepEqual(field(model.attachment, 'owner'), ['owner', ['function', ['name', {
    quoted: 'decodeFrame'
  }]]]);
  assert.equal(validateCommentMeaning(model), true);
  assert.equal(commentMeaningLinks(model)[0], 'meaning');
  assert.equal(model.meaning.includes(source), false);
});
test('held-out type classification is grounded in the actual shared type constructor', () => {
  const model = formal('The result is a string.');
  assert(has(model, 'type'));
  assert(has(model, 'definition'));
  assert(has(model, 'predicate-definition'));
  assert(walk(model.meaning, node => node[0] === 'rust-owned' && node[1] === 'String'));
  const rendered = naturalizeComment(model);
  assert.equal(rendered.text, 'result is a string.');
  assert.deepEqual(rendered.roundTrip, {
    whole: true,
    parts: true
  });
});
test('modality and both existential binders survive actual independent re-formalization', () => {
  const model = formal('Some function may throw an error.');
  assert(has(model, 'existential'));
  assert(has(model, 'modal'));
  assert(has(model, 'effect'));
  assert(walk(model.meaning, node => node[0] === 'mode' && node[1] === 'permission'));
  assert(!JSON.stringify(model.meaning).includes('calendar_month'));
  assert(!JSON.stringify(model.meaning).includes('identifier_reserved_word'));
  const rendered = naturalizeComment(model);
  assert.deepEqual(rendered.roundTrip, {
    whole: true,
    parts: true
  });
  assert.match(rendered.text, /may throw/u);
  assert.doesNotMatch(rendered.text, /False|May/u);
});
test('quantifier-negation scopes are ordered and cannot collapse into a bag of terms', () => {
  const outer = propositions(formal('Not every function writes files.'))[0];
  const inner = propositions(formal('Every function does not write files.'))[0];
  assert.equal(outer[0], 'not');
  assert.equal(outer[1][0], 'universal');
  assert.equal(inner[0], 'universal');
  assert.equal(inner.at(-1)[0], 'not');
  assert.notDeepEqual(outer, inner);
  for (const model of [formal('Not every function writes files.'), formal('Every function does not write files.')]) {
    assert.deepEqual(naturalizeComment(model).roundTrip, {
      whole: true,
      parts: true
    });
  }
});
test('held-out obligations attach implicit subjects to the actual structural source owner', () => {
  const model = formal('must return a string.');
  assert(walk(model.meaning, node => node[0] === 'source-owner'));
  const rendered = naturalizeComment(model);
  assert.match(rendered.text, /decodeFrame must return/u);
  assert.deepEqual(rendered.roundTrip, {
    whole: true,
    parts: true
  });
});
test('negated write is an effect claim and not execution or authority', () => {
  const model = formal('The function does not write files.');
  assert(has(model, 'not'));
  assert(has(model, 'effect'));
  assert.deepEqual(field(model.meaning, 'claims'), ['claims', 'documentary']);
  assert.deepEqual(naturalizeComment(model).roundTrip, {
    whole: true,
    parts: true
  });
});
test('whole document and every independently re-formalized part retain meanings', () => {
  const model = formal('The result is a string. Some function may throw an error.');
  assert.equal(propositions(model).length, 2);
  assert.deepEqual(naturalizeComment(model).roundTrip, {
    whole: true,
    parts: true
  });
});
test('unknown terms remain explicit needs and cannot be rendered by copying source prose', () => {
  const model = formal('Every value must preserve identity.');
  assert(model.needs.some(node => field(node, 'kind')?.[1] === 'unresolved-concept'));
  const rendered = naturalizeComment(model);
  assert.equal(rendered.text, null);
  assert.deepEqual(rendered.roundTrip, {
    whole: false,
    parts: false
  });
});
test('unsupported language and unsupported clause remain explicit semantic gaps', () => {
  const model = formalizeComment(input('// Every function writes files.'), {
    language: 'zz'
  });
  assert(model.needs.length > 0);
  assert.equal(naturalizeComment(model, {
    language: 'zz'
  }).text, null);
  const opaque = formal('Blue triangles quietly shimmer.');
  assert(has(opaque, 'unresolved-proposition'));
  assert.equal(naturalizeComment(opaque).text, null);
});
test('citation retains an ordinary URL and never becomes source truth or executable authority', () => {
  const model = formal('The result is a string. See https://example.org/types.');
  const citation = field(model.meaning, 'citations')[1];
  assert.deepEqual(field(citation, 'target'), ['target', {
    quoted: 'https://example.org/types'
  }]);
  assert.deepEqual(field(citation, 'authority'), ['authority', 'reference-only']);
});
test('changed meanings and changed source fail independent source-to-meaning validation', () => {
  const model = formal('The result is a string.');
  const changed = structuredClone(model);
  field(changed.meaning, 'propositions')[1][1][1] = 'writes';
  assert.equal(validateCommentMeaning(changed), false);
  assert.throws(() => naturalizeComment(changed), /CommentMeaningSourceMismatch/u);
  const source = structuredClone(model);
  source.verbatim = '// The result is a number.';
  assert.equal(validateCommentMeaning(source), false);
});
test('malformed source attachment, delimiter and role refuse rather than infer authority', () => {
  assert.throws(() => formalizeComment({
    ...input('plain prose')
  }), /SourceCommentDelimiterRequired/u);
  assert.throws(() => formalizeComment({
    ...input('// text'),
    owner: null
  }), /StructuralCommentOwnerRequired/u);
  assert.throws(() => formalizeComment({
    ...input('// text'),
    role: {}
  }), /StructuralCommentRoleRequired/u);
  assert.throws(() => formalizeComment(input('// \ud800')), /Utf8CommentRequired/u);
});
test('canonical relation endpoint selection preserves nearest stable ties and malformed-column refusal', () => {
  assert.deepEqual(conceptRelationEndpoints(['left', 'near', 'right'], [0, 4, -1], [-1, -1, 2]), {
    $: 'grounded',
    subject: 'near',
    object: 'right'
  });
  assert.deepEqual(conceptRelationEndpoints(['left', 'tie', 'first', 'second'], [4, 4, -1, -1], [-1, -1, 2, 2]), {
    $: 'grounded',
    subject: 'tie',
    object: 'first'
  });
  assert.deepEqual(conceptRelationEndpoints(['left'], [], []), {
    $: 'unresolved'
  });
  assert.deepEqual(conceptRelationEndpoints([''], [0], [1]), {
    $: 'unresolved'
  });
});

test('CST attachment retains explicit needs and rejects inconsistent documentary meaning', () => {
  const model = formal('Every value must preserve identity.');
  assert.deepEqual(field(commentMeaningLinks(model), 'needs'), ['needs', ...model.needs]);
  const changed = structuredClone(model);
  changed.verbatim = '// The result is a string.';
  assert.throws(() => commentMeaningLinks(changed), /CommentMeaningSourceMismatch/u);
});
