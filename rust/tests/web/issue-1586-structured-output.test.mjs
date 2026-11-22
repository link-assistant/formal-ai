import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { installHost } from '../../../js/agentic/host.mjs';
import { loadRuntimeDerivation } from '../../../js/server/runtime-cache.mjs';
import { anchoredPattern } from '../../../js/server/structured-pattern.mjs';
import { startCli } from './support/structured-log-fixture.mjs';

const object = (properties, required = Object.keys(properties)) => ({
  type: 'object', properties, required, additionalProperties: false,
});
const city = { type: 'string' };
const integer = { type: 'integer' };
// z.toJSONSchema(z.object({ email: z.string().email() }), { target: 'draft-7' }).
const zodEmail = pattern => ({
  $schema: 'http://json-schema.org/draft-07/schema#',
  ...object({ email: { type: 'string', format: 'email', pattern } }),
});
const zodCurrentPattern = String.raw`^(?:[A-Za-z0-9_'+\-]+\.)*[A-Za-z0-9_'+\-]*[A-Za-z0-9_+-]@(?:[A-Za-z0-9][A-Za-z0-9\-]*\.)+[A-Za-z]{2,}$`; // 4.6.7
const zodLegacyPattern = String.raw`^(?!\.)(?!.*\.\.)([A-Za-z0-9_'+\-\.]*)[A-Za-z0-9_+-]@([A-Za-z0-9][A-Za-z0-9\-]*\.)+[A-Za-z]{2,}$`; // 4.0.0, 4.1.13
const zodEmailSchema = zodEmail(zodCurrentPattern);
const zodLegacyEmailSchema = zodEmail(zodLegacyPattern);
const cases = [
  {
    name: 'job', prompt: 'Warehouse supervisor wanted in Springfield. At least 3 years of experience required.',
    instructions: 'Extract the city and the minimum years of experience. Reply with JSON only.',
    schema: object({ city, min_years: integer }), expected: { city: 'Springfield', min_years: 3 },
  },
  {
    name: 'new_location', prompt: 'Office manager wanted in New York. Minimum of 7 years of experience.',
    chat: true, schema: object({ city, min_years: integer }), expected: { city: 'New York', min_years: 7 },
  },
  {
    name: 'json_mode', prompt: 'Return a JSON object with key "answer" equal to the sum of 2 and 3.',
    chat: true, expected: { answer: 5 },
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
    name: 'out', prompt: 'Backend developer, at least 3 years of experience with Go.',
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
  assert.deepEqual(detail, row.expected ? { format, value: row.expected } : { format, refusal });
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

test('OpenAI requests constrain final JSON across endpoint and streaming shapes', { timeout: 120_000 }, async t => {
  const parserContext = { self: {} };
  for (const script of ['parser-cache-primordials.js', 'seed_loader.js']) {
    runInNewContext(readFileSync(new URL('../../../js/' + script, import.meta.url), 'utf8'), parserContext);
  }
  installHost({ parseLino: parserContext.self.FormalAiSeed.parse });
  const run = await startCli(t, {
    binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
    env: { FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false' },
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
        assert.equal(http.status, 200, row.name);
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
