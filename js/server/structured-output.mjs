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
import { criterionVerdict } from './structured-verdict.mjs';
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

function evidence(value, rule) {
  return value === undefined ? undefined : { value, rule };
}

function fieldPointer(parent, key) {
  return parent + '/' + String(key).replaceAll('~', '~0').replaceAll('/', '~1');
}

function recordRules(rules, pointer, value, rule) {
  const fields = value !== null && typeof value === 'object' ? Object.entries(value) : [];
  if (!fields.length) rules.set(pointer, rule);
  for (const [key, item] of fields) recordRules(rules, fieldPointer(pointer, key), item, rule);
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
  if (labeled.length) return evidence(unique(labeled), 'labeled_clause');
  const described = () => {
    const label = typeof description === 'string' ? normalizeKey(description) : '';
    return label ? labeledEvidence(source, label) : [];
  };
  if (number !== undefined && (name === key || (!key && ['answer', 'result', 'sum', 'total'].includes(normalized)))) {
    return evidence(number, 'arithmetic');
  }
  if (normalized === 'city') {
    const locations = source.split(/[.!?;\n]/u).filter(clause => LOCATION.test(clause))
      .flatMap(clause => Array.from(clause.matchAll(CITY), match => match[1]));
    return locations.length ? evidence(unique(locations), 'location_clause') : evidence(unique(described()), 'labeled_clause');
  }
  if (/^(?:min|minimum) (?:years(?: of experience)?|experience years)$/u.test(normalized)) {
    const years = cached('structured-output-experience-pattern', () => new RegExp(
      '\\b' + agenticMessage('structured-output-experience-qualifier') + '\\s+('
      + NUMBER + ')\\s+years?\\s+(?:of\\s+)?experience\\b', 'gu'));
    const matches = Array.from(source.matchAll(years), match => Number(match[1]));
    return matches.length ? evidence(unique(matches), 'experience_years') : evidence(unique(described()), 'labeled_clause');
  }
  const alternate = described();
  if (alternate.length) return evidence(unique(alternate), 'labeled_clause');
  const choices = field?.enum;
  if (Array.isArray(choices)) {
    if (answer.trim() && choices.includes(answer.trim())) return evidence(answer.trim(), 'enum_label');
    if (source.trim() && choices.includes(source.trim())) return evidence(source.trim(), 'enum_label');
    return evidence(sentimentLabel({}, name, field, source), 'sentiment_label');
  }
  return undefined;
}

function objectFields(schema) {
  return schema.properties ? schema : schemaReference(schema, schema.$ref) ?? schema;
}

function fieldDescription(field, resolved) {
  return typeof field?.description === 'string' ? field.description : resolved?.description;
}

function verdictProjection(schema, fields, source) {
  const candidates = [];
  const labels = ['met', 'not_met', UNKNOWN];
  for (const [name, field] of Object.entries(fields.properties ?? {})) {
    const accepts = value => validOutput(value, field, schema);
    const resolved = schemaReference(schema, field?.$ref) ?? field;
    if (labels.every(label => resolved?.enum?.includes(label) && accepts(label))) {
      candidates.push({ name, choices: labels });
    } else if ([true, false, null].every(accepts) && !['', 0, [], {}].some(accepts)) {
      candidates.push({ name, choices: [true, false, null] });
    }
  }
  const named = candidates.filter(({ name }) => normalizeKey(name) === 'verdict');
  const selected = candidates.length === 1 ? candidates[0] : named.length === 1 ? named[0] : undefined;
  if (!selected) return undefined;
  const verdict = criterionVerdict(source);
  if (!verdict) return undefined;
  const value = selected.choices[verdict.result === null ? 2 : verdict.result ? 0 : 1];
  return { name: selected.name, value, trace: verdict.trace, conclusive: verdict.result !== null };
}

function recordVerdict(verdict, rules, placeholders) {
  if (!verdict) return;
  const pointer = fieldPointer('', verdict.name);
  if (verdict.conclusive) rules.set(pointer, verdict.trace.rule);
  else {
    rules.delete(pointer);
    placeholders.push(pointer);
  }
}

function fillObject(schema, source, answer, number, supplied, suppliedRule, verdict) {
  const fields = objectFields(schema);
  const key = KEY.exec(source)?.[1];
  const result = new Map();
  const fieldRules = new Map();
  const placeholders = [];
  const noEvidence = [];
  for (const [name, field] of Object.entries(fields.properties ?? {})) {
    if (verdict?.name === name) {
      result.set(name, verdict.value);
      continue;
    }
    const resolved = schemaReference(schema, field?.$ref) ?? field;
    const description = fieldDescription(field, resolved);
    const items = supplied ? undefined : stringArrayItems(resolved, schema);
    const fieldEvidence = supplied ? (Object.hasOwn(supplied, name) ? evidence(supplied[name], suppliedRule) : undefined)
      : items ? evidence(stringArrayEvidence(source, name, description, items, normalizeKey), 'string_array')
        : evidenceFor(name, resolved, description, source, answer, number, key);
    let value = fieldEvidence?.value;
    if (fieldEvidence) {
      fieldRules.set(name, fieldEvidence.rule);
    } else {
      if (items && validOutput([], field, schema)) {
        value = [];
        noEvidence.push(name);
      } else if ((fields.required ?? []).includes(name)) {
        if (validOutput(null, field, schema)) value = null;
        else if (resolved?.enum?.includes(UNKNOWN) && validOutput(UNKNOWN, field, schema)) value = UNKNOWN;
      }
      if (value !== undefined) placeholders.push(fieldPointer('', name));
    }
    if (value !== undefined) result.set(name, value);
  }
  if (!result.size && number !== undefined && !fields.properties) {
    const name = key ?? 'answer';
    result.set(name, number);
    fieldRules.set(name, 'arithmetic');
  }
  const value = Object.fromEntries(result);
  return result.size ? { value, placeholders, fieldRules, noEvidence } : undefined;
}

async function derive(ctx, schema, source, answer, supplied) {
  const satisfies = candidate => isMap(candidate) && portableValue(candidate) && validOutput(candidate, schema);
  const formal = parsed(answer);
  const fromSolver = isMap(formal);
  const observed = fromSolver ? formal : supplied;
  const suppliedRule = fromSolver ? 'solver_json' : 'input_json';
  const canReadText = !isMap(supplied) && Buffer.byteLength(source) <= 8192;
  let verdict;
  if (canReadText) {
    await ensureNodeHost(ctx);
    verdict = verdictProjection(schema, objectFields(schema), source);
  }
  if (satisfies(observed)) {
    const value = verdict ? { ...observed, [verdict.name]: verdict.value } : observed;
    if (verdict && !satisfies(value)) return undefined;
    const rules = new Map();
    const placeholders = [];
    recordRules(rules, '', value, suppliedRule);
    recordVerdict(verdict, rules, placeholders);
    return { value, placeholders, rules: Object.fromEntries(rules), noEvidence: [], verdict };
  }
  let result;
  if (isMap(observed)) {
    result = fillObject(schema, '', '', undefined, observed, suppliedRule, verdict);
  } else if (canReadText) {
    const [sumPattern, arithmeticPattern] = cached('structured-output-arithmetic-patterns', () =>
      ['structured-output-sum-pattern', 'structured-output-arithmetic-pattern'].map(key =>
        new RegExp(agenticMessage(key, { number: NUMBER }), 'iu')));
    const terms = sumPattern.exec(source.trim());
    const expression = terms ? terms[1] + ' + ' + terms[2] : arithmeticPattern.exec(source.trim())?.[1];
    let number;
    if (expression && expression.length <= 80) {
      try { number = Number(realm().evaluateArithmetic(expression)); } catch { /* Underivable. */ }
    }
    result = fillObject(schema, source, answer, number, undefined, undefined, verdict);
  }
  if (!result || !satisfies(result.value)) return undefined;
  const { value, placeholders, fieldRules, noEvidence } = result;
  const rules = new Map();
  for (const [name, rule] of fieldRules) {
    const pointer = fieldPointer('', name);
    if (rule === 'string_array') rules.set(pointer, rule);
    else recordRules(rules, pointer, value[name], rule);
  }
  recordVerdict(verdict, rules, placeholders);
  placeholders.sort((left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right)));
  return { value, placeholders, rules: Object.fromEntries(rules), noEvidence: noEvidence.sort(), verdict };
}

export async function structuredAnswerFromSource(ctx, format, chat, source) {
  const obligation = outputObligation(format, chat);
  if (!obligation) return null;
  const supplied = parsed(source);
  const derived = obligation.schema && await derive(ctx, obligation.schema, source, '', supplied);
  const value = derived?.value;
  if (!value && obligation.schema && !isMap(supplied)) return null;
  return {
    // The final projection reads the original source so inferred values cannot
    // be mistaken for JSON supplied as evidence.
    intent: 'structured_output', answer: '', confidence: 1,
    evidence_links: [], thinking_steps: [], links_notation: '',
  };
}

export async function projectStructuredOutput(ctx, obligation, source, answer, refusal) {
  const { schema, issue } = obligation;
  const derived = !refusal && schema ? await derive(ctx, schema, source, answer, parsed(source)) : undefined;
  if (!refusal && !derived) {
    await ensureNodeHost(ctx);
    refusal = agenticMessage(schema ? 'structured-output-underivable' : 'structured-output-schema-unsupported');
    if (issue) {
      const key = issue.unsupported ? 'structured-output-schema-unsupported-keyword' : 'structured-output-schema-invalid-keyword';
      refusal += ' ' + agenticMessage(key, { keyword: JSON.stringify(issue.keyword), pointer: issue.pointer || '""' });
    }
  }
  const content = derived ? jsonText(derived.value) : '';
  const verdict = derived?.verdict;
  const detail = jsonText({
    format: obligation.format,
    ...(derived ? {
      value: derived.value, placeholders: derived.placeholders, rules: derived.rules,
      ...(derived.noEvidence.length ? { no_evidence: derived.noEvidence } : {}),
      ...(verdict ? { verdicts: { [verdict.name]: verdict.trace } } : {}),
    } : { refusal }),
  });
  const formal_ai = derived ? {
    status: derived.placeholders.length ? 'unanswered' : 'answered',
    provenance: ['structured_derivation'],
    placeholders: derived.placeholders,
    rules: derived.rules,
  } : { status: 'refused', provenance: ['structured_derivation'] };
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
  return { content, refusal, detail, id: identifier, evidence, formal_ai };
}

export function structuredThinkingStep(steps, result) {
  return thinkingStep(steps.length, 'structured_output', result.detail, 'high', RECOMPOSE_BIND_KIND);
}
