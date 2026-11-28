import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { installHost } from '../../../js/agentic/host.mjs';
import { chatCompletionJson, chatCompletionSse } from '../../../js/server/openai.mjs';
import { loadRuntimeDerivation } from '../../../js/server/runtime-cache.mjs';
import { anchoredPattern } from '../../../js/server/structured-pattern.mjs';
import { startCli } from './support/structured-log-fixture.mjs';

const object = (properties, required = Object.keys(properties)) => ({
  type: 'object', properties, required, additionalProperties: false,
});
const city = { type: 'string' };
const integer = { type: 'integer' };
const nullableInteger = { type: ['integer', 'null'] };
const nullableString = { type: ['string', 'null'] };
const stringArray = { type: 'array', items: { type: 'string' } };
const skills = ['Python', 'SQL', 'Apache Airflow'];
const experience = 'Backend developer, at least 3 years of experience with Go.';
const jobTitle = 'job title: storekeeper';
// z.toJSONSchema(z.object({ email: z.string().email() }), { target: 'draft-7' }).
const zodEmail = pattern => ({
  $schema: 'http://json-schema.org/draft-07/schema#',
  ...object({ email: { type: 'string', format: 'email', pattern } }),
});
const zodCurrentPattern = String.raw`^(?:[A-Za-z0-9_'+\-]+\.)*[A-Za-z0-9_'+\-]*[A-Za-z0-9_+-]@(?:[A-Za-z0-9][A-Za-z0-9\-]*\.)+[A-Za-z]{2,}$`; // 4.6.7
const zodLegacyPattern = String.raw`^(?!\.)(?!.*\.\.)([A-Za-z0-9_'+\-\.]*)[A-Za-z0-9_+-]@([A-Za-z0-9][A-Za-z0-9\-]*\.)+[A-Za-z]{2,}$`; // 4.0.0, 4.1.13
const zodEmailSchema = zodEmail(zodCurrentPattern);
const zodLegacyEmailSchema = zodEmail(zodLegacyPattern);
const verdictField = { enum: ['met', 'not_met', 'unknown'] };
const verdictCases = [
  ['duration_met', 'at least 3 years of Go', 'Go developer from 2018 to 2024', 'met', 'minimum_duration'],
  ['duration_short', 'at least 3 years of Go', 'Go developer from 2023 to 2024', 'not_met', 'minimum_duration'],
  ['duration_since', 'at least 3 years of Go', 'Go developer since 2018', 'met', 'minimum_duration', '2026-11-01'],
  ['skill_met', 'has worked with PostgreSQL', 'built reporting on PostgreSQL and Redis', 'met', 'skill_use'],
  ['skill_negated', 'has worked with PostgreSQL', 'built reporting on MySQL only; never used PostgreSQL',
    'not_met', 'skill_negation', '', 'never used PostgreSQL'],
  ['skill_missing', 'speaks German', 'Go developer from 2018 to 2024', 'unknown', 'no_criterion_evidence'],
  ['duration_missing_reference', 'at least 3 years of Go', 'Go developer since 2018',
    'unknown', 'minimum_duration_missing_reference'],
  ['skill_excluded', 'has used PostgreSQL', 'Used MySQL and Redis only', 'not_met', 'skill_exclusion'],
  ['skill_in_exclusive_list', 'has used Redis', 'Used only MySQL and Redis', 'met', 'skill_use'],
  ['skill_negation_scope', 'has used Redis', 'Never used MySQL but worked with Redis',
    'met', 'skill_use', '', 'worked with Redis'],
  ['skill_mention', 'has worked with PostgreSQL', 'Read PostgreSQL documentation', 'unknown', 'no_criterion_evidence'],
  ['skill_partial_name', 'has used C', 'Worked with C++', 'unknown', 'no_criterion_evidence'],
  ['skill_contracted_negation', 'has worked with Node.js', "Hasn't worked with Node.js", 'not_met', 'skill_negation'],
  ['skill_conflicting', 'has used Redis', 'Used Redis; never used Redis', 'unknown', 'conflicting_criterion_evidence'],
  ['duration_unrelated', 'at least 3 years of Go', 'Go developer. Rust developer from 2018 to 2024',
    'unknown', 'no_criterion_evidence'],
  ['duration_mixed_claims', 'at least 3 years of Go', 'Go developer and Rust developer from 2018 to 2024',
    'unknown', 'no_criterion_evidence'],
  ['duration_anniversary', 'at least 3 years of C++', 'C++ developer from 2022-06-20 to 2025-06-19',
    'not_met', 'minimum_duration'],
  ['duration_inexact', 'at least 3 years of Go', 'Go developer since 2023',
    'unknown', 'minimum_duration_incomplete', '2026-01-01'],
  ['duration_reversed', 'at least 3 years of Go', 'Go developer from 2024 to 2018',
    'unknown', 'minimum_duration_incomplete'],
].map(([name, criterion, text, verdict, rule, reference = '', evidence = text]) => ({
  name, prompt: `Criterion: ${criterion}. ${reference ? 'Today is ' + reference + '. ' : ''}Resume: ${text}.`,
  schema: object({ verdict: verdictField }), expected: { verdict },
  placeholders: verdict === 'unknown' ? ['/verdict'] : [],
  rules: verdict === 'unknown' ? {} : { '/verdict': rule },
  verdicts: { verdict: { rule, evidence, ...(reference ? { reference } : {}) } },
  stream: name === 'duration_met',
}));
verdictCases.push(
  {
    name: 'duration_ru', prompt: 'Критерий: опыт продаж не менее 3 лет. Резюме: менеджер по продажам с 2015 по 2020 год.',
    schema: object({ verdict: verdictField }), expected: { verdict: 'met' },
    placeholders: [], rules: { '/verdict': 'minimum_duration' },
    verdicts: { verdict: { rule: 'minimum_duration', evidence: 'менеджер по продажам с 2015 по 2020 год' } },
  },
  {
    name: 'skill_ru_negated', prompt: 'Критерий: опыт работы с PostgreSQL. Текст: Без опыта работы с PostgreSQL.',
    schema: object({ verdict: verdictField }), expected: { verdict: 'not_met' },
    placeholders: [], rules: { '/verdict': 'skill_negation' },
    verdicts: { verdict: { rule: 'skill_negation', evidence: 'Без опыта работы с PostgreSQL' } },
  },
  ...[
    ['boolean_met', 'Works with Rust', true, 'skill_use', { type: ['boolean', 'null'] }],
    ['boolean_not_met', 'Never used Rust', false, 'skill_negation', { enum: [true, false, null] }],
    ['boolean_unknown', 'Works with Python', null, 'no_criterion_evidence', { enum: [true, false, null] }],
  ].map(([name, text, verdict, rule, field]) => ({
    name, prompt: `Criterion: has worked with Rust. Evidence: ${text}.`,
    schema: { ...object({ qualified: { $ref: '#/$defs/tri' } }), $defs: { tri: field } },
    expected: { qualified: verdict },
    placeholders: verdict === null ? ['/qualified'] : [],
    rules: verdict === null ? {} : { '/qualified': rule },
    verdicts: { qualified: { rule, evidence: text } },
    stream: verdict === null,
  })),
);
const cases = [
  ...verdictCases,
  {
    name: 'verdict_placeholder', prompt: 'Criterion: at least 3 years of Go. Resume: Go developer since 2018.',
    schema: object({ verdict: { type: 'string', enum: ['met', 'not_met', 'unknown'] } }),
    expected: { verdict: 'unknown' }, placeholders: ['/verdict'], rules: {}, stream: true,
    verdicts: { verdict: { rule: 'minimum_duration_missing_reference', evidence: 'Go developer since 2018' } },
  },
  {
    name: 'seniority_placeholder', prompt: 'We are hiring a warehouse supervisor.',
    schema: object({ seniority: nullableString }), expected: { seniority: null },
    placeholders: ['/seniority'], rules: {}, stream: true,
  },
  {
    name: 'experience_rule', prompt: 'at least 3 years of experience',
    schema: object({ min_years_of_experience: integer }), expected: { min_years_of_experience: 3 },
    placeholders: [], rules: { '/min_years_of_experience': 'experience_years' }, stream: true,
  },
  {
    name: 'explicit_unknown_and_null', prompt: 'verdict: unknown; seniority: null',
    schema: object({ verdict: { enum: ['met', 'unknown'] }, seniority: nullableString }),
    expected: { verdict: 'unknown', seniority: null }, placeholders: [],
    rules: { '/verdict': 'labeled_clause', '/seniority': 'labeled_clause' },
  },
  {
    name: 'explicit_enum', prompt: 'unknown',
    schema: object({ verdict: { enum: ['met', 'unknown'] } }), expected: { verdict: 'unknown' },
    placeholders: [], rules: { '/verdict': 'enum_label' },
  },
  {
    name: 'escaped_pointers', prompt: '{"a/b~c":[{"nested":null}],"verdict":"unknown"}',
    schema: object({ 'a/b~c': { type: 'array' }, verdict: { enum: ['met', 'unknown'] } }),
    expected: { 'a/b~c': [{ nested: null }], verdict: 'unknown' },
    placeholders: [], rules: { '/a~1b~0c/0/nested': 'input_json', '/verdict': 'input_json' },
  },
  {
    name: 'partial_input_json', prompt: '{"verdict":"unknown"}',
    schema: object({ verdict: { enum: ['met', 'unknown'] }, seniority: nullableString }),
    expected: { verdict: 'unknown', seniority: null },
    placeholders: ['/seniority'], rules: { '/verdict': 'input_json' },
  },
  {
    name: 'job', prompt: 'Warehouse supervisor wanted in Springfield. At least 3 years of experience required.',
    instructions: 'Extract the city and the minimum years of experience. Reply with JSON only.',
    schema: object({ city, min_years: integer }), expected: { city: 'Springfield', min_years: 3 },
    placeholders: [], rules: { '/city': 'location_clause', '/min_years': 'experience_years' },
  },
  {
    name: 'new_location', prompt: 'Office manager wanted in New York. Minimum of 7 years of experience.',
    chat: true, schema: object({ city, min_years: integer }), expected: { city: 'New York', min_years: 7 },
  },
  ...['min_years_of_experience', 'minYearsOfExperience', 'MinYears', 'min-years-of-experience']
    .map((key, index) => ({
      name: `experience_${index}`, prompt: experience,
      schema: object({ [key]: nullableInteger }), expected: { [key]: 3 },
    })),
  ...['job_title', 'jobTitle'].map((key, index) => ({
    name: `job_title_${index}`, prompt: jobTitle,
    schema: object({ [key]: nullableString }), expected: { [key]: 'storekeeper' }, stream: true,
  })),
  {
    name: 'acronym', prompt: 'HTTP status: 200; APIErrorCount = 1',
    schema: object({ HTTPStatus: integer, 'api-error-count': integer }),
    expected: { HTTPStatus: 200, 'api-error-count': 1 },
  },
  {
    name: 'description_label', prompt: 'Name of the role in the posting is storekeeper',
    schema: object({ role: { ...nullableString, description: 'Name of the role in the posting' } }),
    expected: { role: 'storekeeper' },
  },
  {
    name: 'description_ref', prompt: 'Position name: storekeeper; Position type: part time',
    schema: {
      ...object({
        role: { $ref: '#/$defs/role', description: 'Position name' },
        contract: { $ref: '#/$defs/contract' },
      }),
      $defs: {
        role: { ...city, description: 'Position type' },
        contract: { ...city, description: 'Position type' },
      },
    },
    expected: { role: 'storekeeper', contract: 'part time' },
  },
  {
    name: 'description_priority',
    prompt: 'Job title: storekeeper; Position name: cashier; Office manager wanted in Boston; Office location: Dover.',
    schema: object({
      jobTitle: { ...nullableString, description: 'Position name' },
      city: { ...nullableString, description: 'Office location' },
    }),
    expected: { jobTitle: 'storekeeper', city: 'Boston' },
  },
  {
    name: 'description_location', prompt: 'Office location: Dover.',
    schema: object({ city: { ...nullableString, description: 'Office location' } }),
    expected: { city: 'Dover' },
  },
  {
    name: 'description_ambiguity', prompt: 'Job title: storekeeper; Job title: cashier; Position name: porter',
    schema: object({ jobTitle: { ...nullableString, description: 'Position name' } }),
    expected: { jobTitle: null },
  },
  {
    name: 'skills_en', prompt: 'We need a data engineer with Python, SQL and Apache Airflow experience.',
    strict: null, schema: { type: 'object', properties: { skills: stringArray }, required: ['skills'] },
    expected: { skills }, placeholders: [], rules: { '/skills': 'string_array' }, stream: true,
  },
  {
    name: 'skills_ru', prompt: 'Нужен инженер данных с опытом Python, SQL и Apache Airflow.',
    schema: object({ skills: stringArray }), expected: { skills },
    placeholders: [], rules: { '/skills': 'string_array' }, stream: true,
  },
  {
    name: 'list_topics',
    prompt: 'Requirements: write reports or mentor teammates. Инструменты:\n- Helm\n- Node.js\nЯзыки: Go или Rust.',
    schema: object({
      requirements: stringArray,
      stack: { ...stringArray, description: 'Development tools named in the text' },
      languages: stringArray,
    }),
    expected: { requirements: ['write reports', 'mentor teammates'], stack: ['Helm', 'Node.js'], languages: ['Go', 'Rust'] },
    placeholders: [], rules: { '/requirements': 'string_array', '/stack': 'string_array', '/languages': 'string_array' },
  },
  {
    name: 'list_concepts',
    prompt: 'We work with vendor services and custom builds. We experiment with Links Notation and Wikipedia.',
    schema: object({ topics: stringArray }), expected: { topics: ['Links Notation', 'Wikipedia'] },
    placeholders: [], rules: { '/topics': 'string_array' },
  },
  {
    name: 'list_concept_boundaries', prompt: 'They have experience in links notation, Wikipedia, and event log as examples.',
    schema: object({ skills: stringArray }), expected: { skills: ['links notation', 'Wikipedia', 'event log'] },
    placeholders: [], rules: { '/skills': 'string_array' },
  },
  {
    name: 'list_enum', prompt: 'Tools: python, Rust, c++, or sql.',
    schema: {
      ...object({ tools: { ...stringArray, items: { $ref: '#/$defs/tool' } } }),
      $defs: { tool: { enum: ['SQL', 'Python', 'C++', 'C#'] } },
    },
    expected: { tools: ['Python', 'C++', 'SQL'] },
    placeholders: [], rules: { '/tools': 'string_array' },
  },
  {
    name: 'list_missing', prompt: 'We need a data engineer who communicates clearly and works independently.',
    schema: object({ skills: stringArray }), expected: { skills: [] },
    placeholders: ['/skills'], rules: {}, noEvidence: ['skills'], stream: true,
  },
  {
    name: 'list_missing_ru', prompt: 'Ищем инженера данных с опытом работы.',
    schema: object({ skills: stringArray }), expected: { skills: [] },
    placeholders: ['/skills'], rules: {}, noEvidence: ['skills'],
  },
  {
    name: 'list_wrong_topic', prompt: 'Benefits: insurance and flexible hours.',
    schema: object({ skills: stringArray }), expected: { skills: [] },
    placeholders: ['/skills'], rules: {}, noEvidence: ['skills'],
  },
  {
    name: 'list_min_items', prompt: 'An experienced data engineer is needed.',
    schema: object({ skills: { ...stringArray, minItems: 1 } }),
  },
  {
    name: 'list_explicit_empty', prompt: '{"skills":[]}',
    schema: object({ skills: stringArray }), expected: { skills: [] },
    placeholders: [], rules: { '/skills': 'input_json' },
  },
  {
    name: 'list_labeled_json', prompt: 'Skills: ["C#", "C++", "Node.js", "Version 1. Beta"]',
    schema: object({ skills: stringArray }), expected: { skills: ['C#', 'C++', 'Node.js', 'Version 1. Beta'] },
    placeholders: [], rules: { '/skills': 'string_array' },
  },
  {
    name: 'list_labeled_empty', prompt: 'Skills: []',
    schema: object({ skills: stringArray }), expected: { skills: [] },
    placeholders: [], rules: { '/skills': 'string_array' },
  },
  {
    name: 'list_field_ref', prompt: 'Benefits: insurance and a pension. Skills: analysis, writing, and coaching.',
    schema: {
      ...object({ focus: { $ref: '#/$defs/list', description: 'Skills' } }),
      $defs: { list: { ...stringArray, description: 'Benefits' } },
    },
    expected: { focus: ['analysis', 'writing', 'coaching'] },
    placeholders: [], rules: { '/focus': 'string_array' },
  },
  {
    name: 'json_mode', prompt: 'Return a JSON object with key "answer" equal to the sum of 2 and 3.',
    chat: true, expected: { answer: 5 },
    placeholders: [], rules: { '/answer': 'arithmetic' },
  },
  {
    name: 'calculation', prompt: 'Return the sum of 2 and 3.',
    chat: false, schema: object({ sum: integer }), expected: { sum: 5 },
  },
  {
    name: 'expression', prompt: '2 + 2',
    chat: false, schema: object({ answer: integer }), expected: { answer: 4 },
  },
  {
    name: 'missing', prompt: 'A warehouse supervisor is needed. At least 3 years of experience required.',
    schema: object({ city, min_years: integer }),
  },
  {
    name: 'nullable', prompt: 'A warehouse supervisor is needed. At least 3 years of experience required.',
    chat: true, schema: object({ city: { anyOf: [city, { type: 'null' }] }, min_years: integer }),
    expected: { city: null, min_years: 3 },
  },
  {
    name: 'optional', prompt: 'A warehouse supervisor is needed. At least 3 years of experience required.',
    chat: false, schema: object({ city, min_years: integer }, ['min_years']), expected: { min_years: 3 },
  },
  {
    name: 'classification', prompt: 'This is excellent.',
    chat: false, schema: object({ sentiment: { type: 'string', enum: ['positive', 'neutral', 'negative'] } }),
    expected: { sentiment: 'positive' }, strict: false,
  },
  {
    name: 'outside_enum', prompt: '{"sentiment":"maybe"}',
    chat: true, schema: object({ sentiment: { enum: ['positive', 'neutral', 'negative'] } }),
  },
  {
    name: 'unknown_enum', prompt: 'The text does not say whether there is an approval.',
    chat: false, schema: object({ verdict: { enum: ['approve', 'reject', 'unknown'] } }), expected: { verdict: 'unknown' },
  },
  {
    name: 'ambiguous_location', prompt: 'Open positions in Springfield and in Boston.',
    chat: false, schema: object({ city }),
  },
  {
    name: 'nested_ref', prompt: '{"record":{"city":"Dover","ignored":true}}',
    chat: true, schema: { ...object({ record: { $ref: '#/$defs/record' } }), $defs: { record: object({ city }) } },
  },
  {
    name: 'project_keys', prompt: '{"record":{"city":"Dover"},"ignored":true}',
    chat: false, schema: { ...object({ record: { $ref: '#/$defs/record' } }), $defs: { record: object({ city }) } },
    expected: { record: { city: 'Dover' } },
  },
  {
    name: 'out', prompt: experience,
    chat: true, strict: null,
    schema: {
      $schema: 'http://json-schema.org/draft-07/schema#',
      type: 'object', properties: { min_years_of_experience: { type: ['number', 'null'] } },
      required: ['min_years_of_experience'],
    },
    expected: { min_years_of_experience: 3 },
  },
  {
    name: 'zod_email', prompt: 'email: jane@example.com',
    schema: zodEmailSchema,
    expected: { email: 'jane@example.com' },
  },
  {
    name: 'zod_email_pattern', prompt: 'email: a.b+c@domain.example.com',
    schema: zodEmailSchema, expected: { email: 'a.b+c@domain.example.com' },
  },
  {
    name: 'zod_email_invalid_pattern', prompt: 'email: jane@example.c',
    schema: zodEmailSchema, refusalDetail: 'Could not derive a valid JSON object',
  },
  {
    name: 'zod_legacy_email', prompt: 'email: a.b+c@domain.example.com',
    schema: zodLegacyEmailSchema, expected: { email: 'a.b+c@domain.example.com' },
  },
  {
    name: 'zod_legacy_email_invalid_pattern', prompt: 'email: jane..doe@example.com',
    schema: zodLegacyEmailSchema, refusalDetail: 'Could not derive a valid JSON object',
  },
  {
    name: 'nested_annotations', prompt: 'email: unverified',
    schema: {
      $schema: 'https://json-schema.org/draft/2019-09/schema',
      $id: 'https://example.com/contact', $comment: 'Contact record',
      ...object({ email: {
        type: 'string', format: 'email', deprecated: true, readOnly: true, writeOnly: true,
        contentMediaType: 'text/plain', contentEncoding: 'base64',
      } }),
    },
    expected: { email: 'unverified' },
  },
  {
    name: 'unsupported_multiple', prompt: '{"count":3}', stream: true,
    schema: object({ count: { type: 'number', multipleOf: 3 } }),
    refusalDetail: 'unsupported keyword "multipleOf" at /properties/count',
  },
  {
    name: 'unsupported_reference', prompt: '{"record":[1]}',
    schema: {
      ...object({ record: { $ref: '#/definitions/line~1items~0raw/anyOf/0' } }),
      definitions: { 'line/items~raw': { anyOf: [{ type: 'array', uniqueItems: true }] } },
    },
    refusalDetail: 'unsupported keyword "uniqueItems" at /definitions/line~1items~0raw/anyOf/0',
  },
  {
    name: 'invalid_keyword', prompt: '{"name":"Ada"}',
    schema: object({ name: { minLength: '2' } }),
    refusalDetail: 'invalid value for keyword "minLength" at /properties/name',
  },
  {
    name: 'invalid_root_keyword', prompt: '{"name":"Ada"}',
    schema: { ...object({ name: city }), $schema: 'https://example.com/unknown-dialect' },
    refusalDetail: 'invalid value for keyword "$schema" at ""',
  },
];

// Fixture-side assertions deliberately don't depend on the server's validator.
function assertSchema(value, schema, root = schema) {
  if (schema.$ref) {
    return assertSchema(value, schema.$ref.slice(2).split('/').reduce((node, part) => node[part], root), root);
  }
  if (schema.enum) assert.ok(schema.enum.includes(value));
  if (schema.anyOf) {
    assert.ok(schema.anyOf.some(branch => {
      try { assertSchema(value, branch, root); return true; } catch { return false; }
    }));
  }
  if (schema.type === 'null') assert.equal(value, null);
  if (schema.type === 'string') assert.equal(typeof value, 'string');
  if (schema.type === 'integer') assert.ok(Number.isInteger(value));
  if (schema.type === 'array') {
    assert.ok(Array.isArray(value));
    value.forEach(item => assertSchema(item, schema.items ?? {}, root));
  }
  if (schema.type !== 'object') return;
  assert.ok(value && typeof value === 'object' && !Array.isArray(value));
  for (const key of schema.required ?? []) assert.ok(Object.hasOwn(value, key));
  for (const [key, item] of Object.entries(value)) {
    if (schema.additionalProperties === false) assert.ok(Object.hasOwn(schema.properties, key));
    if (schema.properties?.[key]) assertSchema(item, schema.properties[key], root);
  }
}

function formatFor(row, chat) {
  if (!row.schema) return { type: 'json_object' };
  const { strict = true } = row;
  const spec = { name: row.name, ...(strict === null ? {} : { strict }), schema: row.schema };
  return chat ? { type: 'json_schema', json_schema: spec } : { type: 'json_schema', ...spec };
}

function checkResult(row, format, chat, envelope, content, refusal, steps, env) {
  assert.deepEqual(chat ? envelope.response_format : envelope.text.format, format);
  const metadata = chat ? (envelope.choices[0].message ?? envelope.choices[0].delta).formal_ai : envelope.formal_ai;
  assert.equal(metadata.status, row.expected ? (metadata.placeholders.length ? 'unanswered' : 'answered') : 'refused');
  assert.deepEqual(metadata.provenance, ['structured_derivation']);
  if (row.rules) {
    assert.deepEqual(metadata.placeholders, row.placeholders);
    assert.deepEqual(metadata.rules, row.rules);
  }
  if (row.expected) {
    assert.equal(refusal, undefined);
    const value = JSON.parse(content);
    assertSchema(value, row.schema ?? { type: 'object' });
    assert.deepEqual(value, row.expected);
  } else {
    assert.ok(!content);
    assert.match(refusal, /Could not derive|unsupported constraints/u);
    if (row.refusalDetail) assert.ok(refusal.includes(row.refusalDetail), refusal);
  }
  const step = steps.find(step => step.step === 'structured_output');
  const detail = JSON.parse(step.detail);
  const noEvidence = row.noEvidence ? { no_evidence: row.noEvidence } : {};
  const verdicts = row.verdicts ? { verdicts: row.verdicts } : {};
  assert.deepEqual(detail, row.expected
    ? { format, value: row.expected, placeholders: metadata.placeholders, rules: metadata.rules, ...noEvidence, ...verdicts }
    : { format, refusal });
  assert.match(envelope.derivation_id, /^answer_[a-f0-9]{16}$/u);
  const locator = envelope.evidence_links.find(link => link.startsWith('derivation_record:'));
  assert.ok(locator, JSON.stringify(envelope.evidence_links));
  const record = loadRuntimeDerivation(locator.slice('derivation_record:'.length), { env });
  assert.equal(record.answer_id, envelope.derivation_id);
  assert.deepEqual(JSON.parse(record.recomposition), detail);
}

function events(text) {
  return text.split('\n').filter(line => line.startsWith('data: ') && line !== 'data: [DONE]')
    .map(line => JSON.parse(line.slice(6)));
}

test('Chat Completions preserves an empty refusal in JSON and SSE', () => {
  const message = {
    role: 'assistant', content: '', refusal: '',
    formal_ai: { status: 'unanswered', provenance: ['local_rules'] },
  };
  const completion = { choices: [{ index: 0, message, finish_reason: 'stop' }] };
  assert.equal(chatCompletionJson(completion).choices[0].message.refusal, '');
  const deltas = events(chatCompletionSse(completion, false).body).map(frame => frame.choices[0].delta);
  assert.equal(deltas.find(delta => typeof delta.refusal === 'string')?.refusal, '');
});

test('anchored structured patterns bound repetition without changing matches', () => {
  for (const source of ['^(?:a?)*b$', '^[ab]{2,}(?:ab+)?x?$', zodCurrentPattern, zodLegacyPattern]) {
    const original = new RegExp(source, 'u');
    const bounded = anchoredPattern(source);
    assert.ok(bounded, source);
    for (const input of ['', 'b', 'aa', 'aab', 'ababbbx', 'a.b+c@domain.example.com',
      'jane@example.c', '.jane@example.com', 'jane..doe@example.com']) {
      assert.equal(bounded.test(input), original.test(input), `${source} on ${input}`);
    }
  }
  assert.equal(anchoredPattern('^(?:a+)*b$').test('a'.repeat(16_384)), false);
  assert.equal(anchoredPattern('^(?=abc)abc$'), null);
});

test('both OpenAI endpoints report network retrieval for the request that used it', { timeout: 40_000 }, async t => {
  const paths = [];
  const source = createServer((request, response) => {
    paths.push(request.url);
    response.writeHead(200, { 'content-type': 'text/plain' });
    response.end('Fixture source body.');
  });
  await new Promise(resolve => source.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => source.close(resolve)));
  const run = await startCli(t, {
    binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
    env: { PATH: process.env.PATH, FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false' },
    lifetime: 40_000,
  });
  await run.ready();
  for (const [endpoint, chat] of [['chat/completions', true], ['responses', false]]) {
    const path = '/' + endpoint;
    for (const [prompt, provenance] of [
      [`Fetch http://127.0.0.1:${source.address().port}${path}`, ['local_rules', 'network_retrieval']],
      ['2 + 2', ['local_rules']],
    ]) {
      const body = chat ? { messages: [{ role: 'user', content: prompt }] } : { input: prompt };
      const response = await fetch(run.url + '/api/openai/v1' + path, {
        method: 'POST', headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ ...body, model: 'formal-ai' }),
      });
      assert.equal(response.status, 200);
      const payload = await response.json();
      const metadata = chat ? payload.choices[0].message.formal_ai : payload.formal_ai;
      assert.deepEqual(metadata, { status: 'answered', provenance }, prompt);
      const content = chat ? payload.choices[0].message.content : payload.output[0].content[0].text;
      if (provenance.includes('network_retrieval')) assert.ok(content.includes('Fixture source body.'), prompt);
    }
  }
  assert.deepEqual(paths, ['/chat/completions', '/responses']);
});

test('OpenAI requests constrain final JSON across endpoint and streaming shapes', { timeout: 120_000 }, async t => {
  const parserContext = { self: {} };
  for (const script of ['parser-cache-primordials.js', 'seed-record-readers.js', 'seed_loader.js']) {
    runInNewContext(readFileSync(new URL('../../../js/' + script, import.meta.url), 'utf8'), parserContext);
  }
  installHost({ parseLino: parserContext.self.FormalAiSeed.parse });
  const run = await startCli(t, {
    binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
    env: { FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false', FORMAL_AI_OFFLINE: '1', FORMAL_AI_NETWORK: 'denied' },
    lifetime: 120_000,
  });
  await run.ready();
  const models = await (await fetch(run.url + '/v1/models')).json();
  assert.ok(models.data[0].capabilities.includes('json_schema'));
  for (const chat of [true, false]) {
    for (const row of cases.filter(row => row.chat === undefined || row.chat === chat)) {
      const format = formatFor(row, chat);
      for (const stream of row.stream || row.name === 'job' || row.name === 'missing' ? [false, true] : [false]) {
        const payload = chat ? {
          response_format: format,
          messages: [
            ...(row.instructions ? [{ role: 'system', content: row.instructions }] : []),
            { role: 'user', content: row.prompt },
          ],
        } : { input: row.prompt, instructions: row.instructions, text: { format } };
        const http = await fetch(run.url + (chat ? '/v1/chat/completions' : '/v1/responses'), {
          method: 'POST', headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ ...payload, model: 'formal-ai', stream }),
        });
        if (http.status !== 200) assert.fail(`${row.name}: HTTP ${http.status} ${await http.text()}\n${run.stderr}`);
        try {
          if (!stream) {
            const envelope = await http.json();
            const message = chat ? envelope.choices[0].message : envelope.output.find(item => item.type === 'message');
            const part = chat ? message : message.content[0];
            checkResult(row, format, chat, envelope, chat ? part.content : part.text, part.refusal, message.thinking_steps, run.env);
          } else {
            const frames = events(await http.text());
            const last = frames.at(-1);
            if (chat) {
              const deltas = frames.map(frame => frame.choices[0].delta);
              checkResult(row, format, chat, last, deltas.map(delta => delta.content).join(''),
                deltas.find(delta => delta.refusal)?.refusal, last.choices[0].delta.thinking_steps, run.env);
            } else {
              const envelope = last.response;
              const text = frames.find(frame => frame.type === 'response.output_text.done')?.text;
              const refusal = frames.find(frame => frame.type === 'response.refusal.done')?.refusal;
              checkResult(row, format, chat, envelope, text, refusal, envelope.thinking_steps, run.env);
            }
          }
        } catch (error) {
          throw new Error(`${chat ? 'Chat' : 'Responses'} ${row.name} (stream=${stream})`, { cause: error });
        }
      }
    }
  }
});

test('both OpenAI endpoints distinguish unsupported replies from local answers', { timeout: 60_000 }, async t => {
  const run = await startCli(t, {
    binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
    env: { FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false', FORMAL_AI_OFFLINE: '1', FORMAL_AI_NETWORK: 'denied' },
    args: ['--ephemeral'],
    lifetime: 60_000,
  });
  await run.ready();
  const prompts = [
    'Summarise in one sentence: the applicant answered every screening question and asked about remote work.',
    'Кратко перескажи в одном предложении: соискатель ответил на все вопросы и спросил про удалённую работу.',
    '用一句话总结：申请人回答了所有问题，并询问了远程工作。',
    'Resume en una frase: el solicitante respondió a todas las preguntas y preguntó por el trabajo remoto.',
    'Brainstorm ideas',
    'Refactor the code',
    'Convert this to JSON',
  ];
  for (const chat of [true, false]) {
    for (const [prompt, status] of [...prompts.map(prompt => [prompt, 'unanswered']), ['2 + 2', 'answered']]) {
      for (const stream of [false, true]) {
        const body = chat ? { messages: [{ role: 'user', content: prompt }] } : { input: prompt };
        const result = await fetch(run.url + (chat ? '/v1/chat/completions' : '/v1/responses'), {
          method: 'POST', headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ ...body, model: 'formal-ai', stream }),
        });
        assert.equal(result.status, 200);
        let metadata, content, refusal;
        if (!stream) {
          const payload = await result.json();
          if (chat) {
            ({ formal_ai: metadata, content, refusal } = payload.choices[0].message);
          } else {
            metadata = payload.formal_ai;
            ({ text: content, refusal } = payload.output.find(item => item.type === 'message').content[0]);
          }
        } else {
          const frames = events(await result.text());
          if (chat) {
            const deltas = frames.map(frame => frame.choices[0].delta);
            metadata = deltas.at(-1).formal_ai;
            content = deltas.map(delta => delta.content).join('');
            refusal = deltas.find(delta => typeof delta.refusal === 'string')?.refusal;
          } else {
            metadata = frames.at(-1).response.formal_ai;
            content = frames.find(frame => frame.type === 'response.output_text.done')?.text;
            refusal = frames.find(frame => frame.type === 'response.refusal.done')?.refusal;
          }
        }
        assert.deepEqual(metadata, { status, provenance: ['local_rules'] }, prompt);
        if (status === 'unanswered') {
          assert.equal(typeof refusal, 'string', prompt);
          assert.ok(!content, prompt);
        } else {
          assert.ok(content.includes('4'), prompt);
          assert.equal(refusal, undefined, prompt);
        }
      }
    }
  }
});

test('unanswered replies retain the exchange in memory on both OpenAI endpoints', { timeout: 40_000 }, async t => {
  const run = await startCli(t, {
    binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
    env: { FORMAL_AI_OFFLINE: '1', FORMAL_AI_NETWORK: 'denied', FORMAL_AI_RECORD_CHAT: '1' },
    lifetime: 40_000,
  });
  await run.ready();
  const chatPrompt = 'Summarise in one sentence: the applicant asked about remote work.';
  const responsesPrompt = 'Summarise in one sentence: the applicant asked about flexible hours.';
  for (const [endpoint, body, prompt] of [
    ['chat/completions', { messages: [{ role: 'user', content: chatPrompt }] }, chatPrompt],
    ['responses', { input: responsesPrompt }, responsesPrompt],
  ]) {
    const http = await fetch(`${run.url}/v1/${endpoint}`, {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ ...body, model: 'formal-ai' }),
    });
    assert.equal(http.status, 200);
    const payload = await http.json();
    const metadata = payload.formal_ai ?? payload.choices[0].message.formal_ai;
    assert.equal(metadata.status, 'unanswered', endpoint);
    const memory = await fetch(`${run.url}/v1/memory`);
    assert.equal(memory.status, 200);
    assert.ok((await memory.text()).includes(prompt), endpoint);
  }
});

test('Offline ephemeral Chat Completions extracts skills and derives criterion verdicts', { timeout: 120_000 }, async t => {
  const run = await startCli(t, {
    binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
    args: ['--ephemeral'],
    env: {
      FORMAL_AI_EPHEMERAL: '1', FORMAL_AI_OFFLINE: '1', FORMAL_AI_NETWORK: 'denied',
      FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false',
    },
    lifetime: 120_000,
  });
  await run.ready();
  const rows = ['skills_en', 'skills_ru', 'list_missing'].map(name => cases.find(row => row.name === name));
  rows.push(...verdictCases);
  for (const row of rows) {
    const format = formatFor(row, true);
    const http = await fetch(run.url + '/v1/chat/completions', {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        model: 'formal-ai', response_format: format, messages: [{ role: 'user', content: row.prompt }],
      }),
    });
    if (http.status !== 200) assert.fail(`${row.name}: HTTP ${http.status} ${await http.text()}\n${run.stderr}`);
    const reply = await http.json();
    assert.equal(reply.choices[0].finish_reason, 'stop', row.name);
    const message = reply.choices[0].message;
    assert.equal(message.refusal, undefined, row.name);
    assert.deepEqual(JSON.parse(message.content), row.expected, row.name);
    assert.deepEqual(message.formal_ai, {
      status: row.placeholders.length ? 'unanswered' : 'answered',
      provenance: ['structured_derivation'], placeholders: row.placeholders, rules: row.rules,
    }, row.name);
    const step = message.thinking_steps.find(step => step.step === 'structured_output');
    assert.deepEqual(JSON.parse(step.detail).no_evidence, row.noEvidence, row.name);
    assert.deepEqual(JSON.parse(step.detail).verdicts, row.verdicts, row.name);
    assert.ok(reply.evidence_links.every(link => !link.startsWith('derivation_record:')));
  }
});
