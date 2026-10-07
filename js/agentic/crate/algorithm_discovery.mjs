// Link-native discovery of reusable algorithms from execution traces: the part
// of rust/src/algorithm_discovery.rs (`discover_algorithms`,
// `AlgorithmDiscoveryRun::validated_candidates`,
// `AlgorithmCandidate::{validated, status, links_notation,
// from_links_notation, conformance_links_notation}`,
// `candidate_from_occurrences`, `validate_occurrence`, `subsumes`),
// rust/src/algorithm_discovery/adapters.rs (`traces_from_memory_events`,
// `parse_arguments`) and rust/src/algorithm_discovery/ranking.rs
// (`rank_survivors`, via rust/src/selection_heuristics.rs `LeastActionRanker`
// and `TrizRanker` over data/meta/selection-heuristics.lino; R901-3) that
// `agentic_coding::algorithm_learning` reaches.
//
// Shapes: a trace is `{id, steps: [{operation, arguments: [[name, value]...]}]}`
// with arguments sorted by name (a `BTreeMap`); an `ArgumentPattern` is
// `{constant}` or `{parameter}`.
//
// Rust collects the repeated windows into a `HashMap`, whose iteration order
// is seeded per process, and each surviving candidate's
// `balanced_convert(store, shape)` may append links to the shared store. Here
// windows are visited in first-occurrence order. Every id, step, support and
// held-out verdict is order-independent; only `associative_root` (and the
// `evidence_id` hashing it) can differ, and only when two candidates both
// create new links - a case where two Rust runs disagree with each other too.

import { cached, readText } from '../host.mjs';
import { pushLinoNode } from './links_format.mjs';
import { stableId } from './engine_stable_id.mjs';
import { byteOrder as cmpStr, trim, utf8Len as stringBytes } from './rust_str.mjs';
import { parseRoot, findChildValue } from './seed_parser.mjs';
import { NULL_LINK, SequenceStore, SymbolTable, balancedConvert, compress } from './sequences.mjs';
import { rankWithHeuristic, situationFor } from './selection_heuristics_triz.mjs';

const DEFAULT_MIN_STEPS = 2;
const DEFAULT_SUPPORT_OCCURRENCES = 2;
const DEFAULT_HELD_OUT_OCCURRENCES = 1;
/** Mirrors `MAX_DISCOVERY_INPUT_STEPS` in rust/src/algorithm_discovery.rs. */
export const MAX_DISCOVERY_INPUT_STEPS = 4096;
/** Mirrors `MAX_DISCOVERED_ALGORITHM_STEPS` in rust/src/algorithm_discovery.rs. */
export const MAX_DISCOVERED_ALGORITHM_STEPS = 32;

/**
 * Rust `Ord::cmp` for integers.
 * @param {number} left
 * @param {number} right
 * @returns {number}
 */
const cmpNum = (left, right) => (left < right ? -1 : left > right ? 1 : 0);


// ---- rust/src/algorithm_discovery/adapters.rs -------------------------------

/** A `BTreeMap<String, String>` as sorted `[key, value]` pairs (later inserts win). */
function sortedArguments(pairs) {
  const map = new Map();
  for (const [key, value] of pairs) map.set(key, value);
  return [...map.entries()].sort(([left], [right]) => cmpStr(left, right));
}

/**
 * `serde_json::from_str::<Value>` keeping the number lexemes, so a non-string
 * value re-serializes the way `Value::to_string` does. Returns undefined on any
 * input serde_json rejects (including nesting at its 128-level recursion limit).
 */
function parseSerdeJson(text) {
  let index = 0;
  const fail = () => {
    throw new SyntaxError('json');
  };
  const ws = () => {
    while (index < text.length && ' \t\n\r'.includes(text[index])) index += 1;
  };
  const string = () => {
    let out = '';
    index += 1;
    for (;;) {
      if (index >= text.length) fail();
      const ch = text[index];
      if (ch === '"') {
        index += 1;
        return out;
      }
      if (ch.charCodeAt(0) < 0x20) fail();
      if (ch !== '\\') {
        out += ch;
        index += 1;
        continue;
      }
      const escape = text[index + 1];
      index += 2;
      const simple = { '"': '"', '\\': '\\', '/': '/', b: '\b', f: '\f', n: '\n', r: '\r', t: '\t' };
      if (escape in simple) {
        out += simple[escape];
        continue;
      }
      if (escape !== 'u') fail();
      const unit = () => {
        const hex = text.slice(index, index + 4);
        if (!/^[0-9a-fA-F]{4}$/.test(hex)) fail();
        index += 4;
        return parseInt(hex, 16);
      };
      const high = unit();
      if (high >= 0xdc00 && high <= 0xdfff) fail();
      if (high >= 0xd800 && high <= 0xdbff) {
        if (text[index] !== '\\' || text[index + 1] !== 'u') fail();
        index += 2;
        const low = unit();
        if (low < 0xdc00 || low > 0xdfff) fail();
        out += String.fromCharCode(high, low);
      } else {
        out += String.fromCharCode(high);
      }
    }
  };
  const value = (depth) => {
    ws();
    const ch = text[index];
    if (ch === '{' || ch === '[') {
      if (depth >= 127) fail();
      index += 1;
      const object = ch === '{';
      const entries = object ? new Map() : [];
      ws();
      if (text[index] === (object ? '}' : ']')) {
        index += 1;
        return object ? { type: 'object', entries } : { type: 'array', items: entries };
      }
      for (;;) {
        if (object) {
          ws();
          if (text[index] !== '"') fail();
          const key = string();
          ws();
          if (text[index] !== ':') fail();
          index += 1;
          entries.set(key, value(depth + 1));
        } else {
          entries.push(value(depth + 1));
        }
        ws();
        if (text[index] === ',') {
          index += 1;
          continue;
        }
        if (text[index] !== (object ? '}' : ']')) fail();
        index += 1;
        return object ? { type: 'object', entries } : { type: 'array', items: entries };
      }
    }
    if (ch === '"') return { type: 'string', value: string() };
    for (const [word, literal] of [['true', true], ['false', false], ['null', null]]) {
      if (text.startsWith(word, index)) {
        index += word.length;
        return { type: 'literal', value: literal };
      }
    }
    const match = /^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?/.exec(text.slice(index));
    if (!match) fail();
    index += match[0].length;
    return { type: 'number', lexeme: match[0], integer: !match[2] && !match[3] };
  };
  try {
    const parsed = value(0);
    ws();
    if (index !== text.length) return undefined;
    return parsed;
  } catch {
    return undefined;
  }
}

/** ryu's shortest `f64` rendering, as serde_json prints a float. */
function ryuFloat(number) {
  if (number === 0) return Object.is(number, -0) ? '-0.0' : '0.0';
  const sign = number < 0 ? '-' : '';
  const [mantissa, exponent] = Math.abs(number).toExponential().split('e');
  const digits = mantissa.replace('.', '');
  const kk = Number(exponent) + 1;
  const k = kk - digits.length;
  let body;
  if (k >= 0 && kk <= 16) body = `${digits}${'0'.repeat(k)}.0`;
  else if (kk > 0 && kk <= 16) body = `${digits.slice(0, kk)}.${digits.slice(kk)}`;
  else if (kk > -5 && kk <= 0) body = `0.${'0'.repeat(-kk)}${digits}`;
  else if (digits.length === 1) body = `${digits}e${kk - 1}`;
  else body = `${digits[0]}.${digits.slice(1)}e${kk - 1}`;
  return `${sign}${body}`;
}

/** serde_json's number model: u64, i64 (negative, non-zero), else f64. */
function serdeNumber(lexeme, integer) {
  if (integer) {
    const big = BigInt(lexeme);
    if (big >= 0n && big <= 0xffffffffffffffffn && !lexeme.startsWith('-')) return big.toString();
    if (big < 0n && big >= -0x8000000000000000n) return big.toString();
  }
  const parsed = Number(lexeme);
  if (!Number.isFinite(parsed)) return null;
  return ryuFloat(parsed);
}

/** `serde_json::Value::to_string` (compact, object keys in `BTreeMap` order). */
function serdeToString(node) {
  if (node.type === 'string') return JSON.stringify(node.value);
  if (node.type === 'literal') return String(node.value);
  if (node.type === 'number') return serdeNumber(node.lexeme, node.integer);
  if (node.type === 'array') return `[${node.items.map(serdeToString).join(',')}]`;
  const keys = [...node.entries.keys()].sort(cmpStr);
  return `{${keys.map((key) => `${JSON.stringify(key)}:${serdeToString(node.entries.get(key))}`).join(',')}}`;
}

function hasOutOfRangeNumber(node) {
  if (node.type === 'number') return serdeNumber(node.lexeme, node.integer) === null;
  if (node.type === 'array') return node.items.some(hasOutOfRangeNumber);
  if (node.type === 'object') return [...node.entries.values()].some(hasOutOfRangeNumber);
  return false;
}

/** Mirrors rust/src/algorithm_discovery/adapters.rs `parse_arguments`. */
function parseArguments(input) {
  const parsed = parseSerdeJson(input);
  if (parsed && parsed.type === 'object' && !hasOutOfRangeNumber(parsed)) {
    return sortedArguments([...parsed.entries.entries()]
      .map(([key, value]) => [key, value.type === 'string' ? value.value : serdeToString(value)]));
  }
  const pairs = [];
  for (const part of input.split(/[;,]/)) {
    const trimmed = trim(part);
    const equals = trimmed.indexOf('=');
    if (equals < 0) continue;
    const key = trim(trimmed.slice(0, equals));
    if (key) pairs.push([key, trim(trimmed.slice(equals + 1))]);
  }
  return sortedArguments(pairs);
}

/** Mirrors rust/src/algorithm_discovery/adapters.rs `traces_from_memory_events`. */
export function tracesFromMemoryEvents(events) {
  const grouped = new Map();
  for (const event of events) {
    if (event.kind === 'algorithm_learning_candidate') continue;
    const operation = event.tool ?? event.kind ?? null;
    if (operation === null) continue;
    const id = event.conversation_id ?? 'ungrouped';
    const args = event.inputs === null || event.inputs === undefined ? [] : parseArguments(event.inputs);
    if (!grouped.has(id)) grouped.set(id, []);
    grouped.get(id).push({ operation, arguments: args });
  }
  return [...grouped.entries()].sort(([left], [right]) => cmpStr(left, right))
    .map(([id, steps]) => ({ id, steps }));
}

// ---- rust/src/algorithm_discovery.rs ----------------------------------------

/** Mirrors `AlgorithmCandidate::validated`. */
export function candidateValidated(candidate) {
  return candidate.held_out.length > 0 && candidate.held_out.every((test) => test.passed);
}

/** Mirrors `push_identity` (lengths are UTF-8 byte counts). */
function pushIdentity(name, value) {
  return `${stringBytes(name)}:${name}${stringBytes(value)}:${value}`;
}

/** Mirrors `candidate_identity`. */
function candidateIdentity(steps) {
  let canonical = '';
  for (const step of steps) {
    canonical += pushIdentity('operation', step.operation);
    for (const [key, pattern] of step.arguments) {
      canonical += pushIdentity('argument', key);
      canonical += pushIdentity(pattern.constant !== undefined ? 'constant' : 'parameter',
        pattern.constant !== undefined ? pattern.constant : pattern.parameter);
    }
  }
  return canonical;
}

/** Mirrors `candidate_evidence_identity`. */
function candidateEvidenceIdentity(steps, support, heldOut, associativeRoot) {
  let canonical = candidateIdentity(steps) + pushIdentity('associative_root', String(associativeRoot));
  for (const traceId of support) canonical += pushIdentity('support', traceId);
  for (const test of heldOut) {
    canonical += pushIdentity('held_out_trace', test.trace_id);
    canonical += pushIdentity('start_step', String(test.start_step));
    canonical += pushIdentity('passed', String(test.passed));
    for (const failure of test.failures) canonical += pushIdentity('failure', failure);
  }
  return canonical;
}

/** Mirrors `structured_diagnostic`. */
function diagnostic(kind, fields) {
  return [kind, ...fields.map(([name, value]) => `${name}=${value}`)].join(' ');
}

const argumentValue = (step, key) => {
  const found = step.arguments.find(([name]) => name === key);
  return found ? found[1] : undefined;
};

/** Mirrors `validate_occurrence`. */
function validateOccurrence(schema, occurrence, traces) {
  const trace = traces[occurrence.trace_index];
  const bindings = new Map();
  const failures = [];
  schema.forEach((expected, offset) => {
    const observed = trace.steps[occurrence.start + offset];
    const step = String(offset);
    if (!observed) {
      failures.push(diagnostic('missing_step', [['step', step], ['expected', expected.operation]]));
      return;
    }
    if (observed.operation !== expected.operation) {
      failures.push(diagnostic('operation_mismatch', [['step', step], ['expected', expected.operation], ['observed', observed.operation]]));
    }
    for (const [key, pattern] of expected.arguments) {
      const value = argumentValue(observed, key);
      if (value === undefined) {
        failures.push(diagnostic('missing_argument', [['step', step], ['name', key]]));
        continue;
      }
      if (pattern.constant !== undefined) {
        if (value !== pattern.constant) {
          failures.push(diagnostic('constant_mismatch', [['step', step], ['name', key], ['expected', pattern.constant], ['observed', value]]));
        }
      } else if (bindings.has(pattern.parameter)) {
        const bound = bindings.get(pattern.parameter);
        if (bound !== value) {
          failures.push(diagnostic('parameter_mismatch', [
            ['step', step], ['name', key], ['parameter', pattern.parameter], ['expected', bound], ['observed', value],
          ]));
        }
      } else {
        bindings.set(pattern.parameter, value);
      }
    }
    for (const [key] of observed.arguments) {
      if (!expected.arguments.some(([name]) => name === key)) {
        failures.push(diagnostic('unexpected_argument', [['step', step], ['name', key]]));
      }
    }
  });
  return { trace_id: trace.id, start_step: occurrence.start, passed: failures.length === 0, failures };
}

/** Mirrors `non_overlapping_occurrences`. */
function nonOverlappingOccurrences(occurrences, length) {
  const sorted = occurrences.slice().sort((left, right) => cmpNum(left.trace_index, right.trace_index)
    || cmpNum(left.start, right.start));
  const ends = new Map();
  const selected = [];
  for (const occurrence of sorted) {
    if (occurrence.start >= (ends.get(occurrence.trace_index) ?? 0)) {
      ends.set(occurrence.trace_index, occurrence.start + length);
      selected.push(occurrence);
    }
  }
  return selected;
}

const sameKeys = (left, right) => left.length === right.length
  && left.every(([key], index) => key === right[index][0]);

/** Mirrors `candidate_from_occurrences`. */
function candidateFromOccurrences(store, shape, occurrences, traces) {
  const support = occurrences.slice(0, DEFAULT_SUPPORT_OCCURRENCES);
  const stepAt = (occurrence, offset) => traces[occurrence.trace_index].steps[occurrence.start + offset];
  const locations = [];
  for (let offset = 0; offset < shape.length; offset += 1) {
    const first = stepAt(support[0], offset);
    for (const [key] of first.arguments) {
      const values = support.map((occurrence) => argumentValue(stepAt(occurrence, offset), key));
      if (values.some((value) => value === undefined)) return null;
      locations.push([offset, key, values]);
    }
    if (support.some((occurrence) => !sameKeys(stepAt(occurrence, offset).arguments, first.arguments))) return null;
  }
  const parameterVectors = new Map();
  const steps = traces[support[0].trace_index].steps.slice(support[0].start, support[0].start + shape.length)
    .map((step) => ({ operation: step.operation, arguments: [] }));
  for (const [offset, key, values] of locations) {
    let pattern;
    if (values.every((value) => value === values[0])) {
      pattern = { constant: values[0] };
    } else {
      const vectorKey = JSON.stringify(values);
      if (!parameterVectors.has(vectorKey)) parameterVectors.set(vectorKey, `parameter_${parameterVectors.size + 1}`);
      pattern = { parameter: parameterVectors.get(vectorKey) };
    }
    const target = steps[offset].arguments;
    const existing = target.findIndex(([name]) => name === key);
    if (existing >= 0) target[existing] = [key, pattern];
    else target.push([key, pattern]);
    target.sort(([left], [right]) => cmpStr(left, right));
  }
  const heldOut = occurrences.slice(DEFAULT_SUPPORT_OCCURRENCES)
    .map((occurrence) => validateOccurrence(steps, occurrence, traces));
  const exactHeldOut = new Set(occurrences.slice(DEFAULT_SUPPORT_OCCURRENCES).map((occurrence) => occurrence.trace_index));
  const supportTraces = new Set(support.map((occurrence) => occurrence.trace_index));
  traces.forEach((trace, traceIndex) => {
    if (supportTraces.has(traceIndex) || exactHeldOut.has(traceIndex)) return;
    const start = trace.steps.findIndex((step) => step.operation === steps[0].operation);
    if (start < 0) return;
    heldOut.push(validateOccurrence(steps, { trace_index: traceIndex, start }, traces));
  });
  if (heldOut.length < DEFAULT_HELD_OUT_OCCURRENCES) return null;
  const id = stableId('algorithm', candidateIdentity(steps));
  const supportTraceIds = support.map((occurrence) => traces[occurrence.trace_index].id);
  const associativeRoot = balancedConvert(store, shape);
  const evidenceId = stableId('algorithm_evidence', candidateEvidenceIdentity(steps, supportTraceIds, heldOut, associativeRoot));
  return {
    id,
    evidence_id: evidenceId,
    steps,
    support_trace_ids: supportTraceIds,
    held_out: heldOut,
    associative_root: associativeRoot,
  };
}

/** Mirrors `subsumes`. */
function subsumes(longer, shorter) {
  if (longer.steps.length <= shorter.steps.length) return false;
  const evidence = (candidate) => new Set([...candidate.support_trace_ids, ...candidate.held_out.map((test) => test.trace_id)]);
  const longerEvidence = evidence(longer);
  if (![...evidence(shorter)].every((id) => longerEvidence.has(id))) return false;
  const wanted = shorter.steps.map((step) => step.operation);
  for (let start = 0; start + wanted.length <= longer.steps.length; start += 1) {
    if (wanted.every((operation, offset) => longer.steps[start + offset].operation === operation)) return true;
  }
  return false;
}

/**
 * Mirrors rust/src/algorithm_discovery/ranking.rs `rank_survivors` (R901-3):
 * resolves the ranker through `heuristics_for(HeuristicRole::Rank, situation)`
 * so `TrizRanker` is selected when a contradiction is detected.
 */
function rankSurvivors(candidates) {
  const scores = candidates.map((candidate, index) => ({
    candidate_id: candidate.id,
    checks: [
      candidate.held_out.filter((t) => t.passed).length,
      candidate.held_out.length,
    ],
    cost: {
      steps: candidate.steps.length,
      code_size: candidate.steps.reduce((sum, step) => sum + stringBytes(step.operation), 0),
      resource_units: 0,
      leaf_count: 1,
    },
  }));
  const situation = situationFor(scores);
  const ranked = rankWithHeuristic(scores, situation);
  const taken = new Set(ranked);
  const remaining = candidates.filter((candidate, index) => !taken.has(index))
    .sort((left, right) => cmpStr(left.id, right.id));
  return [...ranked.map((index) => candidates[index]), ...remaining];
}



/**
 * Mirrors `fn discover_algorithms` in rust/src/algorithm_discovery.rs: the
 * `AlgorithmDiscoveryRun` (snake_case fields) over `traces`.
 * @param {Array<object>} traces
 */
export function discoverAlgorithms(traces) {
  const inputSteps = traces.reduce((sum, trace) => sum + trace.steps.length, 0);
  if (inputSteps > MAX_DISCOVERY_INPUT_STEPS) {
    return {
      trace_count: traces.length,
      input_steps: inputSteps,
      observation_limit_exceeded: true,
      distinct_operations: new Set(traces.flatMap((trace) => trace.steps.map((step) => step.operation))).size,
      associative_root: NULL_LINK,
      associative_compression_steps: 0,
      associative_compression_lossless: false,
      compression_ratio_basis_points: 10000,
      candidates: [],
    };
  }
  const store = new SequenceStore();
  const symbols = new SymbolTable();
  const encoded = traces.map((trace) => trace.steps.map((step) => symbols.marker(store, `operation:${step.operation}`)));
  const flattened = [];
  encoded.forEach((sequence, index) => {
    flattened.push(symbols.marker(store, `trace_boundary:${index}`));
    flattened.push(...sequence);
  });
  const associativeRoot = balancedConvert(store, flattened);
  const compression = compress(store, flattened);

  const occurrences = new Map();
  encoded.forEach((sequence, traceIndex) => {
    const longest = Math.min(sequence.length, MAX_DISCOVERED_ALGORITHM_STEPS);
    for (let length = DEFAULT_MIN_STEPS; length <= longest; length += 1) {
      for (let start = 0; start + length <= sequence.length; start += 1) {
        const shape = sequence.slice(start, start + length);
        const key = shape.join(',');
        if (!occurrences.has(key)) occurrences.set(key, { shape, list: [] });
        occurrences.get(key).list.push({ trace_index: traceIndex, start });
      }
    }
  });
  const candidates = [];
  for (const { shape, list } of occurrences.values()) {
    const selected = nonOverlappingOccurrences(list, shape.length);
    if (selected.length < DEFAULT_SUPPORT_OCCURRENCES) continue;
    const candidate = candidateFromOccurrences(store, shape, selected, traces);
    if (candidate) candidates.push(candidate);
  }
  candidates.sort((left, right) => cmpNum(right.steps.length, left.steps.length) || cmpStr(left.id, right.id));
  const maximal = [];
  for (const candidate of candidates) {
    if (maximal.some((retained) => (candidateValidated(retained) || !candidateValidated(candidate))
      && subsumes(retained, candidate))) continue;
    maximal.push(candidate);
  }
  const ratio = flattened.length === 0 ? 10000 : Math.floor((compression.sequence.length * 10000) / flattened.length);
  return {
    trace_count: traces.length,
    input_steps: inputSteps,
    observation_limit_exceeded: false,
    distinct_operations: new Set(encoded.flat()).size,
    associative_root: associativeRoot,
    associative_compression_steps: compression.steps.length,
    associative_compression_lossless: compression.isLossless(store),
    compression_ratio_basis_points: Math.min(ratio, 65535),
    candidates: rankSurvivors(maximal),
  };
}

/** Mirrors `AlgorithmDiscoveryRun::validated_candidates`. @param {object} run */
export function validatedCandidates(run) {
  return run.candidates.filter(candidateValidated);
}

/** Mirrors `AlgorithmCandidate::status`. */
export function candidateStatus(candidate) {
  return candidateValidated(candidate) ? 'held_out_validated' : 'held_out_validation_failed';
}

/** Mirrors `AlgorithmCandidate::links_notation`. */
export function candidateLinksNotation(candidate) {
  let out = '';
  out = pushLinoNode(out, 0, 'algorithm_candidate', candidate.id);
  out = pushLinoNode(out, 2, 'evidence_id', candidate.evidence_id);
  out = pushLinoNode(out, 2, 'mode', 'proposal_only');
  out = pushLinoNode(out, 2, 'human_gated', 'true');
  out = pushLinoNode(out, 2, 'status', candidateStatus(candidate));
  out = pushLinoNode(out, 2, 'associative_root', String(candidate.associative_root));
  out = pushLinoNode(out, 2, 'steps', null);
  candidate.steps.forEach((step, index) => {
    out = pushLinoNode(out, 4, 'step', String(index));
    out = pushLinoNode(out, 6, 'operation', step.operation);
    for (const [name, pattern] of step.arguments) {
      out = pushLinoNode(out, 6, 'argument', name);
      out = pattern.constant !== undefined
        ? pushLinoNode(out, 8, 'constant', pattern.constant)
        : pushLinoNode(out, 8, 'parameter', pattern.parameter);
    }
  });
  out = pushLinoNode(out, 2, 'support', null);
  for (const traceId of candidate.support_trace_ids) out = pushLinoNode(out, 4, 'trace', traceId);
  out = pushLinoNode(out, 2, 'held_out', null);
  for (const test of candidate.held_out) {
    out = pushLinoNode(out, 4, 'test', test.trace_id);
    out = pushLinoNode(out, 6, 'start_step', String(test.start_step));
    out = pushLinoNode(out, 6, 'passed', test.passed ? 'true' : 'false');
    for (const failure of test.failures) out = pushLinoNode(out, 6, 'failure', failure);
  }
  return out;
}

/** An `AlgorithmDiscoveryError::InvalidArtifact` (thrown; `.ok()` callers catch it). */
class InvalidArtifact extends Error {}

/** Mirrors `fn find_node` in rust/src/algorithm_discovery.rs (depth-first). */
function findNode(node, name) {
  for (const child of node.children || []) {
    if (child.name === name) return child;
    const nested = findNode(child, name);
    if (nested) return nested;
  }
  return null;
}

/** Mirrors `fn child` in rust/src/algorithm_discovery.rs. */
function child(node, name) {
  const found = (node.children || []).find((entry) => entry.name === name);
  if (!found) throw new InvalidArtifact(diagnostic('missing_field', [['name', name]]));
  return found;
}

/** Mirrors `fn child_value` in rust/src/algorithm_discovery.rs. */
function childValue(node, name) {
  const value = findChildValue(node, name);
  if (value === '') throw new InvalidArtifact(diagnostic('missing_value', [['name', name]]));
  return value;
}

/** Rust `str::parse::<usize>()` that throws `InvalidArtifact`. */
function parseUsizeOr(text, reason) {
  if (!/^\+?[0-9]+$/.test(text)) throw new InvalidArtifact(reason);
  return Number(text.replace(/^\+/, ''));
}

/**
 * Mirrors `AlgorithmCandidate::from_links_notation`: the integrity-checked
 * candidate, or null for any `Err` (the planner only reads `.ok()`).
 * @param {string} text
 * @returns {object|null}
 */
export function candidateFromLinksNotation(text) {
  try {
    return parseCandidate(text);
  } catch (error) {
    if (error instanceof InvalidArtifact) return null;
    throw error;
  }
}

/** The fallible body of `AlgorithmCandidate::from_links_notation`. */
function parseCandidate(text) {
  const root = findNode(parseRoot(text), 'algorithm_candidate');
  if (!root) throw new InvalidArtifact('missing_algorithm_candidate');
  if (childValue(root, 'mode') !== 'proposal_only' || childValue(root, 'human_gated') !== 'true') {
    throw new InvalidArtifact('not_human_gated_proposal');
  }
  const steps = [];
  for (const node of (child(root, 'steps').children || []).filter((entry) => entry.name === 'step')) {
    const operation = childValue(node, 'operation');
    const args = new Map();
    for (const argument of (node.children || []).filter((entry) => entry.name === 'argument')) {
      if (argument.value === '') throw new InvalidArtifact('argument_name_empty');
      const constant = (argument.children || []).find((entry) => entry.name === 'constant');
      const parameter = (argument.children || []).find((entry) => entry.name === 'parameter');
      let pattern;
      if (constant) {
        pattern = { constant: constant.value };
      } else if (parameter) {
        if (parameter.value === '') throw new InvalidArtifact('parameter_name_empty');
        pattern = { parameter: parameter.value };
      } else {
        throw new InvalidArtifact('argument_without_pattern');
      }
      args.set(argument.value, pattern);
    }
    steps.push({ operation, arguments: [...args.entries()].sort(([left], [right]) => cmpStr(left, right)) });
  }
  if (steps.length < DEFAULT_MIN_STEPS) throw new InvalidArtifact('too_few_steps');
  if (steps.length > MAX_DISCOVERED_ALGORITHM_STEPS) throw new InvalidArtifact('too_many_steps');
  const support = (child(root, 'support').children || []).filter((entry) => entry.name === 'trace').map((entry) => entry.value);
  if (support.length < DEFAULT_SUPPORT_OCCURRENCES) throw new InvalidArtifact('insufficient_support');
  const heldOut = (child(root, 'held_out').children || []).filter((entry) => entry.name === 'test').map((node) => {
    const failures = (node.children || []).filter((entry) => entry.name === 'failure').map((entry) => entry.value);
    const passedText = childValue(node, 'passed');
    if (passedText !== 'true' && passedText !== 'false') throw new InvalidArtifact('invalid_passed');
    const passed = passedText === 'true';
    if (passed !== (failures.length === 0)) throw new InvalidArtifact('passed_contradicts_failures');
    return {
      trace_id: node.value,
      start_step: parseUsizeOr(childValue(node, 'start_step'), 'invalid_start_step'),
      passed,
      failures,
    };
  });
  if (heldOut.length === 0) throw new InvalidArtifact('no_held_out_evidence');
  const candidate = {
    id: root.value,
    evidence_id: childValue(root, 'evidence_id'),
    steps,
    support_trace_ids: support,
    held_out: heldOut,
    associative_root: parseUsizeOr(childValue(root, 'associative_root'), 'invalid_associative_root'),
  };
  if (candidate.id !== stableId('algorithm', candidateIdentity(candidate.steps))) {
    throw new InvalidArtifact('identity_check_failed');
  }
  if (childValue(root, 'status') !== candidateStatus(candidate)) throw new InvalidArtifact('status_check_failed');
  const expectedEvidence = stableId('algorithm_evidence', candidateEvidenceIdentity(
    candidate.steps, candidate.support_trace_ids, candidate.held_out, candidate.associative_root,
  ));
  if (candidate.evidence_id !== expectedEvidence) throw new InvalidArtifact('evidence_check_failed');
  return candidate;
}

/**
 * Mirrors `AlgorithmCandidate::conformance_links_notation`: the side-effect-free
 * materialization record, or null for `Err` (not validated / missing binding).
 * @param {object} candidate
 * @param {string} trigger
 * @param {Map<string, string>} bindings
 * @returns {string|null}
 */
export function conformanceLinksNotation(candidate, trigger, bindings) {
  if (!candidateValidated(candidate)) return null;
  let out = '';
  out = pushLinoNode(out, 0, 'algorithm_conformance', candidate.id);
  out = pushLinoNode(out, 2, 'mode', 'proposal_conformance');
  out = pushLinoNode(out, 2, 'side_effects', 'false');
  out = pushLinoNode(out, 2, 'trigger', trigger);
  for (const [index, step] of candidate.steps.entries()) {
    out = pushLinoNode(out, 2, 'step', String(index));
    out = pushLinoNode(out, 4, 'operation', step.operation);
    for (const [name, pattern] of step.arguments) {
      let value;
      if (pattern.constant !== undefined) {
        value = pattern.constant;
      } else {
        if (!bindings.has(pattern.parameter)) return null;
        value = bindings.get(pattern.parameter);
      }
      out = pushLinoNode(out, 4, 'argument', name);
      out = pushLinoNode(out, 6, 'value', value);
    }
  }
  return pushLinoNode(out, 2, 'result', 'passed');
}

/** Mirrors `#[derive(PartialEq)]` on `AlgorithmCandidate`. */
export function candidatesEqual(left, right) {
  if (!left || !right) return left === right;
  const stepsEqual = left.steps.length === right.steps.length && left.steps.every((step, index) => {
    const other = right.steps[index];
    return step.operation === other.operation && step.arguments.length === other.arguments.length
      && step.arguments.every(([name, pattern], position) => {
        const [otherName, otherPattern] = other.arguments[position];
        return name === otherName && pattern.constant === otherPattern.constant && pattern.parameter === otherPattern.parameter;
      });
  });
  const listEqual = (a, b) => a.length === b.length && a.every((item, index) => item === b[index]);
  return left.id === right.id && left.evidence_id === right.evidence_id && stepsEqual
    && listEqual(left.support_trace_ids, right.support_trace_ids)
    && left.associative_root === right.associative_root
    && left.held_out.length === right.held_out.length
    && left.held_out.every((test, index) => {
      const other = right.held_out[index];
      return test.trace_id === other.trace_id && test.start_step === other.start_step
        && test.passed === other.passed && listEqual(test.failures, other.failures);
    });
}
