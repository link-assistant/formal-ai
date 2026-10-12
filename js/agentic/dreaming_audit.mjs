// Agent-CLI recipe for auditing and generalizing the issue-#540 dreaming loop
// (rust/src/agentic_coding/dreaming_audit.rs).
//
// The audit is derived at runtime from the grounded dreaming recipe
// (data/meta/dreaming-recipe.lino): the analyzer parses the recipe's
// `meta_step`, `meta_function` and `meta_test` records, cross-references which
// functions ground each stage and which test suites pin it, and reports any
// stage that is not grounded as an open gap.
//
// `crate::dreaming::lexicon::load_data_document` prefers a
// `FORMAL_AI_DATA_DIR` override and falls back to the embedded copy; the
// JavaScript host has no environment, so this reads the repository copy (the
// embedded one).

import { readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { rustLines } from './content.mjs';
import { splitOnce, trim, trimEnd, trimMatches, trimStart } from './crate/rust_str.mjs';

/** Mirrors `DREAMING_AUDIT_PATH` in rust/src/agentic_coding/dreaming_audit.rs. */
export const DREAMING_AUDIT_PATH = 'dreaming-gap-analysis.lino';

/** Mirrors `DREAMING_AUDIT_TASK` in rust/src/agentic_coding/dreaming_audit.rs. */
export function dreamingAuditTask() {
  return agenticMessage('dreaming_audit_task');
}

/** Mirrors `RecipeRecord::field`: the first value under `key`, or null. */
const recordField = (record, key) => record.find(([name]) => name === key)?.[1] ?? null;

/** Mirrors `fn parse_recipe` in rust/src/agentic_coding/dreaming_audit.rs. */
function parseRecipe(text) {
  const records = [];
  for (const line of rustLines(text)) {
    if (trim(line) === '') continue;
    const indent = line.length - trimStart(line).length;
    const trimmed = trim(line);
    if (indent === 0) {
      records.push([]);
      continue;
    }
    const record = records[records.length - 1];
    if (!record) continue;
    const split = splitOnce(trimmed, ' ');
    if (split) record.push([split[0], trimMatches(trim(split[1]), (character) => character === '"')]);
  }
  return records;
}

/** Mirrors `fn analyze_recipe` in rust/src/agentic_coding/dreaming_audit.rs. */
function analyzeRecipe(records) {
  const recordType = (record) => recordField(record, 'record_type') ?? '';
  const functions = records.filter((record) => recordType(record) === 'meta_function');
  const tests = records.filter((record) => recordType(record) === 'meta_test');
  const stages = records.filter((record) => recordType(record) === 'meta_step').map((step) => {
    const id = recordField(step, 'id') ?? '';
    const detail = (recordField(step, 'detail') ?? '').toLowerCase();
    const groundingFunctions = functions.map((record) => recordField(record, 'function'))
      .filter((name) => name !== null && detail.includes(name.toLowerCase()));
    const pinningSuites = tests.filter((test) => {
      const pins = (recordField(test, 'pins') ?? '').toLowerCase();
      const file = recordField(test, 'test_file');
      return pins.includes(id.split('_').join(' '))
        || groundingFunctions.some((name) => pins.includes(name.toLowerCase()))
        || (file !== null && detail.includes(file.toLowerCase()));
    }).map((test) => recordField(test, 'suite')).filter((suite) => suite !== null);
    const orderText = recordField(step, 'order');
    const order = orderText !== null && /^\+?[0-9]+$/.test(orderText) ? Number(orderText) : Infinity;
    return {
      order,
      stage: {
        id,
        title: recordField(step, 'title') ?? '',
        grounding_functions: groundingFunctions,
        pinning_suites: pinningSuites,
        source_file: recordField(step, 'source_file') ?? '',
      },
    };
  });
  stages.sort((left, right) => (left.order < right.order ? -1 : left.order > right.order ? 1 : 0));
  return stages.map(({ stage }) => stage);
}

/**
 * Mirrors `fn is_dreaming_audit_task` in rust/src/agentic_coding/dreaming_audit.rs:
 * the prompt names the audit artifact itself.
 * @param {string} prompt
 */
export function isDreamingAuditTask(prompt) {
  return prompt.toLowerCase().includes('dreaming-gap-analysis');
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/dreaming_audit.rs. */
export function renderDocument() {
  return renderDocumentFrom(readText('data/meta/dreaming-recipe.lino'), readText('data/meta/dreaming-cues.lino'));
}

/**
 * Mirrors `fn render_document_from` in rust/src/agentic_coding/dreaming_audit.rs.
 * @param {string} recipe
 * @param {string} cues
 */
export function renderDocumentFrom(recipe, cues) {
  const stages = analyzeRecipe(parseRecipe(recipe));
  const cueCount = rustLines(cues).filter((line) => trimStart(line).startsWith('cue "')).length;
  const openGaps = stages.filter((stage) => stage.grounding_functions.length === 0).length;
  let out = 'dreaming_gap_analysis\n';
  out += '  record_type "agent_cli_gap_analysis"\n';
  out += '  issue "540"\n';
  out += `  method "${agenticMessage('dreaming_audit_method')}"\n`;
  out += `  grounded_recipe_steps "${stages.length}"\n`;
  out += `  multilingual_cues "${cueCount}"\n`;
  out += `  open_gaps "${openGaps}"\n`;
  out += fieldAt(2, 'conclusion', agenticMessage(openGaps === 0 ? 'dreaming_audit_conclusion_grounded' : 'dreaming_audit_conclusion_open'));
  stages.forEach((stage, index) => {
    out += `  resolution_${String(index + 1).padStart(2, '0')}\n`;
    out += fieldAt(4, 'stage', stage.id);
    out += fieldAt(4, 'generalization', stage.title);
    out += fieldAt(4, 'source_file', stage.source_file);
    out += fieldAt(4, 'grounding_functions', joinOrNone(stage.grounding_functions));
    out += fieldAt(4, 'pinned_by_suites', joinOrNone(stage.pinning_suites));
    out += fieldAt(4, 'status', stage.grounding_functions.length === 0 ? 'open_gap' : 'grounded');
  });
  return out;
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/dreaming_audit.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  const lines = rustLines(document);
  const stageCount = lines.filter((line) => trim(line).startsWith('resolution_')).length;
  const openGaps = lines.filter((line) => trim(line) === 'status "open_gap"').length;
  return agenticMessage('dreaming_audit_final_answer', {
    stage_count: stageCount, open_gaps: openGaps, path: DREAMING_AUDIT_PATH, document: trimEnd(document),
  });
}

/** Mirrors `fn join_or_none`. */
const joinOrNone = (values) => (values.length === 0 ? 'none' : values.join(', '));

/** Mirrors `fn field_at`: a backslash-escaped double-quoted value. */
function fieldAt(indent, name, value) {
  const escaped = value.split('\\').join('\\\\').split('"').join('\\"');
  return `${' '.repeat(indent)}${name} "${escaped}"\n`;
}
