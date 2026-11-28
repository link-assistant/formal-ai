// One output obligation shared by both OpenAI APIs.
import { cached, realm } from '../agentic/host.mjs';
import { agenticMessage } from '../agentic/messages.mjs';
import { jsonText } from '../agentic/plan.mjs';
import { RECOMPOSE_BIND_KIND, RENDER_EMIT_KIND, recordFor } from '../agentic/crate/derivation.mjs';
import { sentimentLabel } from './client-functions.mjs';
import { persistRuntimeDerivation } from './runtime-cache.mjs';
import { ensureNodeHost } from './node-host-install.mjs';
import { stableId } from './ids.mjs';
import { isMap } from './protocol-values.mjs';
import { thinkingStep } from './thinking.mjs';
import { stringArrayEvidence, stringArrayItems } from './structured-array.mjs';
import { portableValue, schemaIssue, schemaReference, validOutput } from './structured-schema.mjs';

const JSON_OBJECT = { type: 'object' };
const UNKNOWN = 'unknown';

export function structuredOutputRequested(format) {
  return format != null && format.type !== 'text';
}

export function outputObligation(format, chat) {
  if (!structuredOutputRequested(format)) return null;
  if (format.type === 'json_object') return { format, schema: JSON_OBJECT };
  const specification = chat ? format.json_schema : format;
  if (format.type !== 'json_schema' || !isMap(specification)
      || (Object.hasOwn(specification, 'strict') && typeof specification.strict !== 'boolean')
      || !Object.hasOwn(specification, 'schema')) return { format, schema: null };
  const issue = schemaIssue(specification.schema);
  return { format, schema: issue ? null : specification.schema, issue };
}

function parsed(text) {
  try { return JSON.parse(text); } catch { return undefined; }
}

const NUMBER = String.raw`-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?`;
const KEY = /\b(?:key|property|field)\s+["'`]([A-Za-z_][A-Za-z_0-9-]*)["'`]/iu;
const CITY = /\b(?:in|based in|located in)\s+(\p{Lu}[\p{L}'’-]*(?:[ \t]+\p{Lu}[\p{L}'’-]*)*)(?=$|[,.!?;]|[ \t]+(?:with|is|at|and|where)\b)/gu;
const LOCATION = /\b(?:wanted|hiring|jobs?|openings?|positions?|based|located)\b/iu;
const LABEL = /^[ \t]*([\p{L}\p{N}_-]+(?:[ \t]+[\p{L}\p{N}_-]+)*?)[ \t]*(?::|=|[ \t]+is[ \t]+)[ \t]*(.+)$/iu;
const normalizeKey = name => name
  .replace(/([\p{Ll}\p{N}])([\p{Lu}\p{Lt}])/gu, '$1 $2')
  .replace(/([\p{Lu}\p{Lt}])([\p{Lu}\p{Lt}]\p{Ll})/gu, '$1 $2')
  .toLowerCase().replace(/[^\p{L}\p{N}]+/gu, ' ').trim();

function unique(values) {
  return values.length && values.every(value => Object.is(value, values[0])) ? values[0] : undefined;
}

function labeledEvidence(source, label) {
  return source.split(/[\n;,]/u).map(clause => LABEL.exec(clause.trimStart()))
    .filter(match => match && normalizeKey(match[1]) === label)
    .map(match => {
      const text = match[2].trim().replace(/[.!?]+$/u, '');
      const value = parsed(text);
      return value === undefined ? text : value;
    });
}

function evidenceFor(name, field, description, source, answer, number, key) {
  const normalized = normalizeKey(name);
  const labeled = labeledEvidence(source, normalized);
  if (labeled.length) return unique(labeled);
  const described = () => {
    const label = typeof description === 'string' ? normalizeKey(description) : '';
    return label ? labeledEvidence(source, label) : [];
  };
  if (number !== undefined && (name === key || (!key && ['answer', 'result', 'sum', 'total'].includes(normalized)))) return number;
  if (normalized === 'city') {
    const locations = source.split(/[.!?;\n]/u).filter(clause => LOCATION.test(clause))
      .flatMap(clause => Array.from(clause.matchAll(CITY), match => match[1]));
    return unique(locations.length ? locations : described());
  }
  if (/^(?:min|minimum) (?:years(?: of experience)?|experience years)$/u.test(normalized)) {
    const years = cached('structured-output-experience-pattern', () => new RegExp(
      '\\b' + agenticMessage('structured-output-experience-qualifier') + '\\s+('
      + NUMBER + ')\\s+years?\\s+(?:of\\s+)?experience\\b', 'gu'));
    const matches = Array.from(source.matchAll(years), match => Number(match[1]));
    return unique(matches.length ? matches : described());
  }
  const alternate = described();
  if (alternate.length) return unique(alternate);
  const choices = field?.enum;
  if (Array.isArray(choices)) {
    if (answer && choices.includes(answer.trim())) return answer.trim();
    if (choices.includes(source.trim())) return source.trim();
    return sentimentLabel({}, name, field, source);
  }
  return undefined;
}

function objectFields(schema) {
  return schema.properties ? schema : schemaReference(schema, schema.$ref) ?? schema;
}

function fieldDescription(field, resolved) {
  return typeof field?.description === 'string' ? field.description : resolved?.description;
}

function fillObject(schema, source, answer, number, supplied) {
  const fields = objectFields(schema);
  const key = KEY.exec(source)?.[1];
  const result = new Map();
  for (const [name, field] of Object.entries(fields.properties ?? {})) {
    const resolved = schemaReference(schema, field?.$ref) ?? field;
    const description = fieldDescription(field, resolved);
    const items = stringArrayItems(resolved, schema);
    let value = supplied ? (Object.hasOwn(supplied, name) ? supplied[name] : undefined)
      : items ? stringArrayEvidence(source, name, description, items, normalizeKey)
        : evidenceFor(name, resolved, description, source, answer, number, key);
    if (!supplied && items && value === undefined && validOutput([], field, schema)) value = [];
    if (value === undefined && (fields.required ?? []).includes(name)) {
      if (validOutput(null, field, schema)) value = null;
      else if (resolved?.enum?.includes(UNKNOWN) && validOutput(UNKNOWN, field, schema)) value = UNKNOWN;
    }
    if (value !== undefined) result.set(name, value);
  }
  if (!result.size && number !== undefined && !fields.properties) result.set(key ?? 'answer', number);
  const candidate = Object.fromEntries(result);
  return result.size && validOutput(candidate, schema) ? candidate : undefined;
}

async function derive(ctx, schema, source, answer, supplied) {
  const satisfies = candidate => isMap(candidate) && portableValue(candidate) && validOutput(candidate, schema);
  const formal = parsed(answer);
  const observed = isMap(formal) ? formal : supplied;
  if (satisfies(observed)) return observed;
  let value;
  if (isMap(observed)) {
    value = fillObject(schema, '', '', undefined, observed);
  } else if (Buffer.byteLength(source) <= 8192) {
    await ensureNodeHost(ctx);
    const [sumPattern, arithmeticPattern] = cached('structured-output-arithmetic-patterns', () =>
      ['structured-output-sum-pattern', 'structured-output-arithmetic-pattern'].map(key =>
        new RegExp(agenticMessage(key, { number: NUMBER }), 'iu')));
    const terms = sumPattern.exec(source.trim());
    const expression = terms ? terms[1] + ' + ' + terms[2] : arithmeticPattern.exec(source.trim())?.[1];
    let number;
    if (expression && expression.length <= 80) {
      try { number = Number(realm().evaluateArithmetic(expression)); } catch { /* Underivable. */ }
    }
    value = fillObject(schema, source, answer, number);
  }
  return satisfies(value) ? value : undefined;
}

export async function structuredAnswerFromSource(ctx, format, chat, source) {
  const obligation = outputObligation(format, chat);
  if (!obligation) return null;
  const supplied = parsed(source);
  const value = obligation.schema && await derive(ctx, obligation.schema, source, '', supplied);
  if (!value && obligation.schema && !isMap(supplied)) return null;
  return {
    intent: 'structured_output', answer: value ? jsonText(value) : '', confidence: 1,
    evidence_links: [], thinking_steps: [], links_notation: '',
  };
}

export async function projectStructuredOutput(ctx, obligation, source, answer, refusal) {
  const { schema, issue } = obligation;
  const supplied = parsed(source);
  const value = !refusal && schema ? await derive(ctx, schema, source, answer, supplied) : undefined;
  if (!refusal && value === undefined) {
    await ensureNodeHost(ctx);
    refusal = agenticMessage(schema ? 'structured-output-underivable' : 'structured-output-schema-unsupported');
    if (issue) {
      const key = issue.unsupported ? 'structured-output-schema-unsupported-keyword' : 'structured-output-schema-invalid-keyword';
      refusal += ' ' + agenticMessage(key, { keyword: JSON.stringify(issue.keyword), pointer: issue.pointer || '""' });
    }
  }
  const content = refusal ? '' : jsonText(value);
  const noEvidence = value ? await missingArrayEvidence(ctx, schema, source, supplied, value) : [];
  const detail = jsonText({ format: obligation.format, ...(refusal ? { refusal } : { value }),
    ...(noEvidence.length ? { no_evidence: noEvidence } : {}) });
  const identifier = stableId('answer', refusal ? detail : content);
  const events = [
    { kind: RECOMPOSE_BIND_KIND, payload: detail },
    { kind: RENDER_EMIT_KIND, payload: 'answer_id=' + identifier + ';format=json' },
  ];
  const evidence = ['derivation:' + identifier];
  if (!ctx.ephemeral) {
    try {
      await ensureNodeHost(ctx);
      const location = persistRuntimeDerivation(recordFor(events, identifier), { env: ctx.env ?? process.env });
      evidence.push(location.path, 'derivation_record:' + location.locator);
    } catch (error) {
      evidence.push('derivation:persistence_failed:' + error.message);
    }
  }
  return { content, refusal, detail, id: identifier, evidence };
}

async function missingArrayEvidence(ctx, schema, source, supplied, value) {
  if (isMap(supplied) || Buffer.byteLength(source) > 8192) return [];
  const missing = [];
  for (const [name, field] of Object.entries(objectFields(schema).properties ?? {})) {
    if (!Array.isArray(value[name]) || value[name].length) continue;
    const resolved = schemaReference(schema, field?.$ref) ?? field;
    const items = stringArrayItems(resolved, schema);
    if (!items) continue;
    await ensureNodeHost(ctx);
    if (stringArrayEvidence(source, name, fieldDescription(field, resolved), items, normalizeKey) === undefined) {
      missing.push(name);
    }
  }
  return missing.sort();
}

export function structuredThinkingStep(steps, result) {
  return thinkingStep(steps.length, 'structured_output', result.detail, 'high', RECOMPOSE_BIND_KIND);
}
