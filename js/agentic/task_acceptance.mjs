// Independent task acceptance (issue #1166): every clause of a task gets a
// verdict -- discharged, contradicted, unresolved or inapplicable -- with a
// justification and the clause's source span. The specification is read
// from the task text alone and the check reads only the candidate's files and
// observed output, never the derivation that wrote the candidate, so a
// candidate and a test generated from the same misunderstanding are rejected
// together. A judgment clause ("clear", "best practices") stays unresolved
// for review; nothing is ever reported as a generic style pass.
// Native twin: rust/src/agentic_coding/task_acceptance.rs.
//
// Supported subset (anything else is an unresolved `unread` clause):
//   output        a print or output clause; its quoted, code-spanned or fenced
//                 lines with explicit counts, or the conditional statements of
//                 task_condition.mjs checked row by row
//   program_file  the requested language's file extension
//   comments      a comment line in the language's comment syntax
//   run           a leading comment that names the program file
//   workflow      path and directory literals (with negated exclusions), push
//                 and pull_request triggers, running the program, verifying
//                 the expected output, the runtime, an observed conclusion,
//                 a named file and a badge; branch filters are not read
import { readText } from './host.mjs';
import { conditionalStatement, conditionTable, fenceLines, firstRoleMatch, taskLiterals } from './task_condition.mjs';
import { workItemRequirements } from './work_item_requirements.mjs';
import { fill, template } from './work_item_steps.mjs';
import { meaningEvidencedIn, mentionsRole } from './write_lexicon.mjs';
import { findChildValue, parseLinoRoot } from './write_lino.mjs';
import { programLanguageBySlug } from './crate/coding_catalog.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';
import { explicitCount } from './crate/intent_formalization_obligations.mjs';
import { requestedIn } from './crate/implementation_language.mjs';

const HEADING = /^#{1,6}\s+(.*)$/u;
const CHECKBOX = /^\[[ xX]\]\s*/u;
const FILE_NAME = /^[\p{L}\p{N}_.-]*[\p{L}\p{N}_]\.[A-Za-z][A-Za-z0-9]{0,4}$/u;
const WORKFLOW_FILE = /\.ya?ml$/u;
const TRIGGER_LINE = /^(?:on|"on"|'on'|true)\s*:(.*)$/u;
const VERDICT_RANK = Object.freeze({ inapplicable: 0, discharged: 1, unresolved: 2, contradicted: 3 });
const JUDGMENT = 'judgment_required';
const UNREAD = new Set(['unread', 'output_unread', 'language_unread', 'comment_syntax_unread', JUDGMENT]);

const byteLength = (text) => new TextEncoder().encode(text).length;
const basename = (path) => path.slice(path.lastIndexOf('/') + 1);
const extension = (path) => {
  const name = basename(path);
  const dot = name.lastIndexOf('.');
  return dot <= 0 ? '' : name.slice(dot).toLowerCase();
};
const stem = (path) => {
  const name = basename(path);
  const dot = name.lastIndexOf('.');
  return dot <= 0 ? name : name.slice(0, dot);
};
const lowered = (text) => normalizePrompt(text).toLowerCase();
const relative = (path) => path.replace(/^\.\//u, '');
/** Mirrors `fn output_lines`: stdout as lines, trailing blanks dropped and each line's trailing whitespace trimmed. */
export function outputLines(stdout) {
  const lines = String(stdout).replaceAll('\r\n', '\n').split('\n').map((line) => line.trimEnd());
  while (lines.length && lines[lines.length - 1] === '') lines.pop();
  return lines;
}

/** Mirrors `fn comment_prefix`: the line-comment syntax of `language` from the stdout program contracts. */
export function commentPrefix(language) {
  const root = parseLinoRoot(readText('data/meta/stdout-program-contracts.lino')).children[0];
  const contract = (root?.children ?? []).find((node) => node.name === 'language' && node.id === language);
  const prefix = contract ? findChildValue(contract, 'comment') : '';
  return prefix === '' ? null : prefix;
}

/** Mirrors `fn file_names`: file-name tokens of `clause`, quoted or bare. */
function fileNames(clause, literals) {
  const out = literals.map((literal) => literal.text).filter((text) => FILE_NAME.test(text));
  for (const word of clause.split(/\s+/u)) {
    const token = word.replace(/^[("'`«“]+|[)"'`»”,;:!?。，]+$/gu, '').replace(/\.$/u, '');
    if (FILE_NAME.test(token) && !token.includes('/') && !out.includes(token)) out.push(token);
  }
  return out;
}

/** Mirrors `fn clause_contexts`: each requirement with its section heading and list lead. */
function clauseContexts(text) {
  const headings = [];
  let offset = 0;
  for (const line of text.split('\n')) {
    const match = HEADING.exec(line.trim());
    if (match) headings.push([offset, match[1]]);
    offset += byteLength(line) + 1;
  }
  let lead = null,
    section = null;
  return workItemRequirements(text).map(([raw, [start, end]]) => {
    const heading = headings.filter(([at]) => at <= start).at(-1) ?? null;
    if (heading !== section) {
      section = heading;
      lead = null;
    }
    const clause = raw.trim().replace(CHECKBOX, '').replaceAll('**', '').trim();
    const isLead = /[:：]$/u.test(clause);
    const context = { clause, start, end, heading: heading?.[1] ?? '', lead: isLead ? null : lead, isLead };
    if (isLead) lead = clause;
    return context;
  });
}

const workflowContext = (text) => mentionsRole('ci_workflow_request', lowered(text)) || mentionsRole('workflow_context_noun', lowered(text));

/** Mirrors `fn workflow_checks`: the workflow checks a workflow-context clause asks for. */
function workflowChecks(clause, words, literals, language) {
  const checks = [];
  const negation = firstRoleMatch(clause.toLowerCase(), 'statement_negation_cue');
  const paths = literals.filter((literal) => literal.text.includes('/') && !/\s/u.test(literal.text));
  if (paths.length) {
    const included = paths.filter((literal) => negation === null || literal.start < negation[0]).map((literal) => relative(literal.text));
    const excluded = paths.filter((literal) => negation !== null && literal.start >= negation[0]).map((literal) => relative(literal.text));
    return [{ check: 'workflow_path', included, excluded }];
  }
  if (negation === null) {
    const events = JSON.parse(template('workflow-trigger-events')).filter(([role]) => mentionsRole(role, words)).map(([, event]) => event);
    if (events.length) checks.push({ check: 'workflow_triggers', events });
  }
  if (mentionsRole('workflow_execution_verb', words)) checks.push({ check: 'workflow_runs_program' });
  if (meaningEvidencedIn('print_stdout', words) || mentionsRole('output_obligation_verb', words)) {
    const lines = literals.filter((literal) => !literal.escaped).map((literal) => literal.text);
    checks.push({ check: 'workflow_verifies_output', lines: lines.length ? lines : null });
  }
  if (mentionsRole('workflow_status_obligation', words)) checks.push({ check: 'workflow_status' });
  if (mentionsRole('ci_badge_obligation', words)) checks.push({ check: 'workflow_badge' });
  const names = mentionsRole('file_naming_obligation', words) ? fileNames(clause, literals) : [];
  if (names.length) checks.push({ check: 'workflow_file_name', names });
  if (language !== null && requestedIn(clause) === language) checks.push({ check: 'workflow_runtime', language });
  if (!checks.length) checks.push({ check: 'workflow_exists' });
  return checks;
}

/** Mirrors `fn clause_checks`: what one clause asks of a candidate. */
function clauseChecks(context, language, leadIllustrative) {
  const { clause } = context;
  if (context.isLead) return [{ check: 'inapplicable', reason: 'lead_clause' }];
  if (leadIllustrative) return [{ check: 'inapplicable', reason: 'illustrative_item' }];
  const words = lowered(clause);
  const literals = taskLiterals(clause);
  const checks = [];
  if (workflowContext(clause) || (context.lead !== null && workflowContext(context.lead)) || workflowContext(context.heading)) {
    checks.push(...workflowChecks(clause, words, literals, language));
  } else {
    const output = meaningEvidencedIn('print_stdout', words) || mentionsRole('output_obligation_verb', words);
    if (output) checks.push(outputCheck(clause, literals));
    const runs = mentionsRole('run_instruction_obligation', words);
    if (runs) checks.push({ check: 'run_instructions' });
    if (!runs && mentionsRole('code_comment_obligation', words)) checks.push({ check: 'comments' });
    if (mentionsRole('program_file_obligation', words) || (!output && mentionsRole('coding_request_verb', words) && requestedIn(clause) !== null)) {
      checks.push({ check: 'program_file' });
    }
  }
  // A named example settles the judgment it illustrates (`a meaningful name like ci.yml`).
  if (mentionsRole('judgment_obligation', words) && !checks.some((check) => check.check === 'workflow_file_name')) checks.push({ check: 'judgment' });
  return checks.length ? checks : [{ check: 'unread' }];
}

/** Mirrors `fn output_check`: an output clause as fixed lines, a conditional statement or a gap. */
function outputCheck(clause, literals) {
  if (literals.some((literal) => literal.escaped)) return { check: 'output', gap: 'escaped_quote' };
  const statement = conditionalStatement(clause);
  if (statement !== null) return statement.gap ? { check: 'output', gap: statement.gap } : { check: 'output', statement };
  const lines = [];
  literals.forEach((literal, index) => {
    const count = explicitCount(clause.slice(literal.end, literals[index + 1]?.start ?? clause.length));
    for (let time = 0; time < count; time += 1) lines.push(literal.text);
  });
  lines.push(...(fenceLines(clause) ?? []));
  return { check: 'output', lines };
}

/** Mirrors `fn merged_lines`: each distinct line as often as the clause that repeats it most, in first-seen order. */
function mergedLines(groups) {
  const order = [],
    counts = new Map();
  for (const lines of groups) {
    const local = new Map();
    for (const line of lines) local.set(line, (local.get(line) ?? 0) + 1);
    for (const [line, count] of local) {
      if (!counts.has(line)) order.push(line);
      counts.set(line, Math.max(count, counts.get(line) ?? 0));
    }
  }
  return order.flatMap((line) => Array(counts.get(line)).fill(line));
}

/**
 * Mirrors `fn specify_task`: the typed obligations of `text` --
 * `{task_identity, language, clauses, output, meaning}` where `output` is
 * `{lines}`, `{table}` or `{gap}` and `meaning` is the canonical JSON that
 * equivalent paraphrases share.
 */
export function specifyTask(text) {
  const language = requestedIn(text);
  const contexts = clauseContexts(text);
  const clauses = contexts.map((context) => {
    const leadIllustrative = context.lead !== null && mentionsRole('illustrative_marker', lowered(context.lead));
    const words = lowered(context.clause);
    return {
      text: context.clause,
      span: [context.start, context.end],
      required: !mentionsRole('optional_obligation_marker', words),
      checks: clauseChecks(context, language, leadIllustrative),
    };
  });
  const outputs = clauses.flatMap((clause) => clause.checks.filter((check) => check.check === 'output'));
  const statements = [];
  for (const check of outputs.filter((output) => output.statement)) {
    if (check.statement.continues && statements.length) statements.at(-1).otherwise.push(...check.statement.otherwise);
    else if (!check.statement.continues) statements.push({ ...check.statement, otherwise: [...check.statement.otherwise] });
  }
  const fixed = outputs.filter((output) => output.lines && output.lines.length);
  let output;
  if (outputs.some((check) => check.gap)) output = { gap: outputs.find((check) => check.gap).gap };
  else if (statements.length && fixed.length) output = { gap: 'mixed_conditional_output' };
  else if (statements.length) {
    const table = conditionTable(statements);
    output = typeof table === 'string' ? { gap: table } : { table };
  } else output = { lines: mergedLines(fixed.map((check) => check.lines)) };
  const workflow = clauses.flatMap((clause) => clause.checks).filter((check) => check.check.startsWith('workflow_'));
  const pick = (name, field) => [...new Set(workflow.filter((check) => check.check === name).flatMap((check) => check[field]))].sort();
  const kinds = [...new Set(clauses.flatMap((clause) => clause.checks.map((check) => check.check)))]
    .filter((kind) => kind !== 'unread' && kind !== 'inapplicable').sort();
  const meaning = JSON.stringify({
    language,
    output,
    workflow: { included: pick('workflow_path', 'included'), excluded: pick('workflow_path', 'excluded'), events: pick('workflow_triggers', 'events'), names: pick('workflow_file_name', 'names') },
    kinds,
  });
  return { task_identity: stableId('task', text), language, clauses, output, meaning };
}

/** Mirrors `fn workflow_events`: the events of a workflow's `on:` key. */
export function workflowEvents(source) {
  const lines = String(source).split('\n');
  const index = lines.findIndex((line) => TRIGGER_LINE.test(line));
  if (index < 0) return [];
  const inline = TRIGGER_LINE.exec(lines[index])[1].replace(/#.*$/u, '').trim();
  const clean = (word) => word.trim().replace(/^["']|["']$/gu, '');
  if (inline.startsWith('[')) return inline.replace(/^\[|\]$/gu, '').split(',').map(clean).filter((word) => word !== '');
  if (inline !== '') return [clean(inline)];
  const events = [];
  let indent = null;
  for (const line of lines.slice(index + 1)) {
    if (line.trim() === '' || line.trim().startsWith('#')) continue;
    const depth = line.length - line.trimStart().length;
    if (depth === 0) break;
    if (indent === null) indent = depth;
    if (depth !== indent) continue;
    const word = clean(line.trim().replace(/^-\s*/u, '').replace(/:.*$/u, ''));
    if (word !== '') events.push(word);
  }
  return events;
}

/** Mirrors `fn comment_lines`: the comment lines of `source` and its leading comment block. */
function commentLines(source, prefix) {
  const lines = String(source).split('\n').map((line) => line.trim());
  const comments = lines.filter((line) => line.startsWith(prefix) && !line.startsWith('#!'));
  const leading = [];
  for (const line of lines) {
    if (line === '' || line.startsWith('#!')) continue;
    if (!line.startsWith(prefix)) break;
    leading.push(line);
  }
  return { comments, leading };
}

/** Mirrors `fn observed_lines`: the observation for `inputs` compared against `expected`. */
function observedMatch(observations, inputs, expected) {
  const observation = observations.find((candidate) => JSON.stringify(candidate.args ?? []) === JSON.stringify(inputs));
  if (!observation) return null;
  const lines = outputLines(observation.stdout ?? '');
  const compared = observation.tail ? lines.slice(Math.max(0, lines.length - expected.length)) : lines;
  return JSON.stringify(compared) === JSON.stringify(expected);
}

/** Mirrors `fn check_output`. */
function checkOutput(specification, candidate) {
  const output = specification.output;
  if (output.gap) return ['unresolved', output.gap];
  const observations = candidate.observations ?? [];
  if (output.table) {
    for (const row of output.table.rows) {
      const matched = observedMatch(observations, row.args, row.lines);
      if (matched === null) return ['unresolved', 'condition_rows_unobserved'];
      if (!matched) return ['contradicted', 'condition_row_differs'];
    }
    return ['discharged', 'condition_rows_match'];
  }
  if (!output.lines.length) return ['unresolved', 'output_unread'];
  const matched = observedMatch(observations, [], output.lines);
  if (matched === null) return ['unresolved', 'output_unobserved'];
  return matched ? ['discharged', 'output_matches'] : ['contradicted', 'output_differs'];
}

/** Mirrors `fn check_one`: one check against the candidate as `[verdict, justification]`. */
function checkOne(check, specification, candidate) {
  const files = candidate.files ?? [];
  const program = files.find((file) => file.path === candidate.program_path) ?? null;
  const workflows = files.filter((file) => WORKFLOW_FILE.test(file.path) && /^jobs\s*:/mu.test(file.source));
  const workflowText = () => {
    const named = new Set(workflows.flatMap((file) => files.filter((other) => file.source.includes(basename(other.path))).map((other) => other.path)));
    return files.filter((file) => named.has(file.path) || workflows.includes(file)).map((file) => file.source).join('\n');
  };
  const language = specification.language === null ? null : programLanguageBySlug(specification.language);
  switch (check.check) {
    case 'inapplicable':
      return ['inapplicable', check.reason];
    case 'unread':
      return ['unresolved', 'unread'];
    case 'judgment':
      return ['unresolved', JUDGMENT];
    case 'output':
      return checkOutput(specification, candidate);
    case 'program_file': {
      if (program === null) return ['unresolved', 'program_unobserved'];
      if (language === null) return ['unresolved', 'language_unread'];
      return extension(program.path) === extension(language.save_as) ? ['discharged', 'extension_matches'] : ['contradicted', 'extension_differs'];
    }
    case 'comments':
    case 'run_instructions': {
      if (program === null) return ['unresolved', 'program_unobserved'];
      const prefix = specification.language === null ? null : commentPrefix(specification.language);
      if (prefix === null) return ['unresolved', 'comment_syntax_unread'];
      const { comments, leading } = commentLines(program.source, prefix);
      const name = basename(program.path);
      if (check.check === 'run_instructions') return leading.some((line) => line.includes(name)) ? ['discharged', 'run_instructions_present'] : ['contradicted', 'run_instructions_missing'];
      return comments.some((line) => !line.includes(name)) ? ['discharged', 'comments_present'] : ['contradicted', 'comments_missing'];
    }
    default:
      return checkWorkflow(check, specification, candidate, workflows, workflowText, program);
  }
}

/** Mirrors `fn check_workflow`. */
function checkWorkflow(check, specification, candidate, workflows, workflowText, program) {
  if (check.check === 'workflow_badge') {
    const names = workflows.map((file) => basename(file.path));
    const badge = (candidate.files ?? []).some((file) => /\.(?:md|markdown|rst|txt)$/iu.test(file.path) && names.some((name) => file.source.includes(name)));
    return badge ? ['discharged', 'badge_present'] : ['unresolved', 'badge_unobserved'];
  }
  if (check.check === 'workflow_status') {
    if (candidate.workflow_conclusion === undefined || candidate.workflow_conclusion === null) return ['unresolved', 'workflow_status_unobserved'];
    return candidate.workflow_conclusion === 'success' ? ['discharged', 'workflow_passed'] : ['contradicted', 'workflow_failed'];
  }
  if (!workflows.length) return ['contradicted', 'workflow_missing'];
  const paths = workflows.map((file) => relative(file.path));
  const at = (path, place) => (place.endsWith('/') ? path.startsWith(place) : path === place);
  switch (check.check) {
    case 'workflow_path': {
      const fits = paths.some((path) => check.included.every((place) => at(path, place)) && !check.excluded.some((place) => at(path, place)));
      return fits ? ['discharged', 'workflow_path_matches'] : ['contradicted', 'workflow_path_differs'];
    }
    case 'workflow_triggers': {
      const events = new Set(workflows.flatMap((file) => workflowEvents(file.source)));
      return check.events.every((event) => events.has(event)) ? ['discharged', 'workflow_triggers_match'] : ['contradicted', 'workflow_trigger_missing'];
    }
    case 'workflow_runs_program': {
      if (program === null) return ['unresolved', 'program_unobserved'];
      const text = workflowText();
      return text.includes(basename(program.path)) || text.includes(stem(program.path)) ? ['discharged', 'workflow_runs_program'] : ['contradicted', 'workflow_does_not_run_program'];
    }
    case 'workflow_verifies_output': {
      const lines = check.lines ?? specification.output.lines ?? [];
      if (!lines.length) return ['unresolved', 'output_unread'];
      const text = workflowText();
      return lines.every((line) => text.includes(line)) ? ['discharged', 'workflow_verifies_output'] : ['contradicted', 'workflow_does_not_verify_output'];
    }
    case 'workflow_file_name':
      return paths.some((path) => check.names.includes(basename(path))) ? ['discharged', 'workflow_file_named'] : ['unresolved', JUDGMENT];
    case 'workflow_runtime': {
      const language = programLanguageBySlug(check.language);
      const text = workflowText().toLowerCase();
      return text.includes(check.language) || (language !== null && text.includes(language.name.toLowerCase())) ? ['discharged', 'workflow_runtime_present'] : ['contradicted', 'workflow_runtime_missing'];
    }
    default:
      return ['discharged', 'workflow_present'];
  }
}

/**
 * Mirrors `fn check_task`: the verdict of every clause of `specification` for
 * `candidate` (`{task_identity, program_path, files: [{path, source}],
 * observations: [{args, stdout, tail?}], workflow_conclusion?}`), with the
 * counts and whether the task is complete. A required clause that is
 * contradicted, or unresolved for any reason but judgment, keeps it incomplete.
 */
export function checkTask(specification, candidate) {
  const sameTask = candidate.task_identity === specification.task_identity;
  const verdicts = specification.clauses.map((clause) => {
    let verdict = 'inapplicable',
      justification = 'no_obligation';
    for (const check of clause.checks) {
      const [next, why] = sameTask || check.check === 'inapplicable' ? checkOne(check, specification, candidate) : ['contradicted', 'task_identity_differs'];
      if (VERDICT_RANK[next] > VERDICT_RANK[verdict]) [verdict, justification] = [next, why];
    }
    return { clause: clause.text, span: clause.span, required: clause.required, verdict, justification };
  });
  const count = (name) => verdicts.filter((verdict) => verdict.verdict === name).length;
  const blocking = verdicts.filter((verdict) => verdict.required && (verdict.verdict === 'contradicted' || (verdict.verdict === 'unresolved' && verdict.justification !== JUDGMENT)));
  return {
    verdicts,
    discharged: count('discharged'),
    contradicted: count('contradicted'),
    unresolved: count('unresolved'),
    inapplicable: count('inapplicable'),
    complete: blocking.length === 0,
  };
}

/**
 * Mirrors `fn acceptance_report`: the verdict lines a finished recipe's answer
 * carries, or '' when every clause of `objective` is discharged or
 * inapplicable. The candidate is the recipe's files and the last non-empty
 * verification output, compared at its tail.
 */
export function acceptanceReport(objective, recipe, outputs) {
  const specification = specifyTask(objective);
  const stdout = [...outputs].reverse().find((output) => output.trim() !== '') ?? '';
  const report = checkTask(specification, {
    task_identity: recipe.task_identity ?? specification.task_identity,
    program_path: recipe.path,
    files: [{ path: recipe.path, source: recipe.source }, ...recipe.supporting_files],
    observations: [{ args: [], stdout, tail: true }],
  });
  const open = report.verdicts.filter((verdict) => verdict.verdict !== 'discharged' && verdict.verdict !== 'inapplicable');
  // A clause outside the supported subset is a reader gap, not evidence against the candidate: the report is
  // appended only when some verdict is actionable (contradicted, or unresolved for want of an observation).
  if (!open.some((verdict) => !UNREAD.has(verdict.justification))) return '';
  const counts = ['discharged', 'contradicted', 'unresolved', 'inapplicable'].map((name) => [`{${name}}`, String(report[name])]);
  const lines = open.map((verdict) => fill('task_acceptance_line', [['{verdict}', verdict.verdict], ['{justification}', verdict.justification],
    ['{start}', String(verdict.span[0])], ['{end}', String(verdict.span[1])], ['{clause}', verdict.clause.replaceAll('\n', ' ')]]));
  const closing = fill(report.complete ? 'task_acceptance_review' : 'task_acceptance_incomplete', []);
  return [fill('task_acceptance_heading', counts), ...lines, closing].join('\n');
}
