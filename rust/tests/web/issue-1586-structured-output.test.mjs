import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { installHost } from '../../../js/agentic/host.mjs';
import { loadRuntimeDerivation } from '../../../js/server/runtime-cache.mjs';
import { startCli } from './support/structured-log-fixture.mjs';

const object = (properties, required = Object.keys(properties)) => ({
  type: 'object', properties, required, additionalProperties: false,
});
const city = { type: 'string' };
const integer = { type: 'integer' };
const nullableInteger = { type: ['integer', 'null'] };
const nullableString = { type: ['string', 'null'] };
const experience = 'Backend developer, at least 3 years of experience with Go.';
const jobTitle = 'job title: storekeeper';
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
    name: 'unsupported', prompt: '{"city":"Dover"}',
    chat: true, schema: object({ city: { ...city, format: 'hostname' } }),
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
  const spec = { name: row.name, strict: row.strict ?? true, schema: row.schema };
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

test('OpenAI requests constrain final JSON across endpoint and streaming shapes', { timeout: 120_000 }, async t => {
  const parserContext = { self: {} };
  for (const script of ['parser-cache-primordials.js', 'seed-record-readers.js', 'seed_loader.js']) {
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
