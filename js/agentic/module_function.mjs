// Add a synthesized function to an existing source module, a test of it to
// its test module, and run the stated command (PR #1188 dogfooding, T1):
//
//   Add a function multiply(a, b) to math.mjs that returns a times b, add a
//   test for it to math.test.mjs, and run node --test to confirm it passes.
//
// Nothing here knows the task. The function is synthesized by the shared
// solver from the clause that states it, in the language the module's
// extension names (the seeded extension table of
// data/seed/page-formalization-rules.lino); the test's expected value is
// computed from the specification itself — the clause after the seeded
// return action, its parameters bound to the contract's sample arguments,
// evaluated by the calculator — never from the synthesized code. The edits
// and the command run through the execution-recipe reroute, so the module
// and the test are written whole after they were read, and the request's
// own command is the verification. Mirrors rust/src/agentic_coding/module_function.rs,
// which synthesizes natively by searching the seeded binary operations
// instead of calling the browser composer.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { sourceFromAgentReadResult, sourceFromReadResult } from './code_artifact.mjs';
import { planSymbolicCommandReroute } from './command_reroute.mjs';
import { cached, readText, realm, solve } from './host.mjs';
import { planOne } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { normalizeCommandWord } from './shell_command_policy.mjs';
import { failureMessage } from './tool_result.mjs';
import { readArguments, resultForPath } from './workspace_change.mjs';
import { meaningEvidencedIn, mentionsRole, wordsForRole } from './write_lexicon.mjs';
import { findChildValue, parseLinoRoot } from './write_lino.mjs';
import { cleanPathToken, looksLikeFilePath, safeRelativePath, tokens } from './write_request.mjs';
import { programLanguageBySlug } from './crate/coding_catalog.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { terminalCommandVocabulary } from './crate/seed_terminal_commands.mjs';

const fill = (template, slots) => slots.reduce((text, [slot, value]) => text.split(`{${slot}}`).join(value), template);
const bare = (word) => word.replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, '').toLowerCase();

/** Mirrors `fn contract`: the function-test contract of `language`, or null. */
export function contract(language) {
  const root = cached('module_function:contracts',
    () => parseLinoRoot(readText('data/meta/function-test-contracts.lino') ?? '').children[0] ?? null);
  return (root?.children || []).find((node) => node.name === 'language' && node.id === language) ?? null;
}

/** The language the seeded extension table names for `path`, or null. */
export function extensionLanguage(path) {
  const table = cached('module_function:extensions', () => {
    const root = parseLinoRoot(readText('data/seed/page-formalization-rules.lino') ?? '').children[0];
    return (root?.children || []).filter((node) => node.name === 'extension')
      .map((node) => [findChildValue(node, 'suffix'), findChildValue(node, 'language')]);
  });
  return table.find(([suffix]) => suffix !== '' && path.endsWith(suffix))?.[1] ?? null;
}

/** The request without its quoted segments. */
function outsideQuotes(request) {
  let outside = '';
  let cursor = 0;
  for (const segment of quotedSegmentSpans(request)) {
    if (segment.start < cursor) continue;
    outside += `${request.slice(cursor, segment.start)} `;
    cursor = segment.end;
  }
  return outside + request.slice(cursor);
}

/** The request's clauses: split at `,` `;` and sentence ends (in any script) outside parentheses. */
function clauses(request) {
  const out = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < request.length; index += 1) {
    const character = request[index];
    if (character === '(') depth += 1;
    else if (character === ')') depth = Math.max(0, depth - 1);
    else if (depth === 0 && (/[\uff0c\uff1b\u3002\uff01\uff1f\u0964]/u.test(character)
      || (/[,;.!?]/u.test(character) && (index + 1 === request.length || /\s/u.test(request[index + 1]))))) {
      out.push(request.slice(start, index));
      start = index + 1;
    }
  }
  out.push(request.slice(start));
  return out.map((clause) => clause.trim()).filter(Boolean);
}

/** `name(a, b)`: the first call-shaped signature whose parameters are identifiers. */
export function signature(text) {
  for (const match of text.matchAll(/([A-Za-z_$][\w$]*)\s*\(([^()]*)\)/gu)) {
    const parameters = match[2].split(',').map((part) => part.trim());
    if (parameters.every((parameter) => /^[A-Za-z_$][\w$]*$/u.test(parameter))) {
      return { name: match[1], parameters, at: match.index };
    }
  }
  return null;
}

/** The marks that end a command's last word: sentence and clause marks in any script. */
const COMMAND_ENDS = /[,;.!?\uff0c\uff1b\u3002\uff01\uff1f\u0964]$/u;
const COMMAND_PEEL = /[,;.!?\uff0c\uff1b\u3002\uff01\uff1f\u0964]+$/u;

/**
 * The command the request's last seeded run verb names, headed by a seeded
 * shell token: the words after the verb ("run node --test", "запусти node
 * --test", "运行 node --test"), or, in a verb-final clause, the words from the
 * shell token up to the verb ("node --test चलाओ"). A run verb is a seeded
 * `run_verbs` word in any script, or a word ending in a seeded
 * `cjk_run_verbs` verb ("然后运行").
 */
function statedCommand(request) {
  const vocabulary = terminalCommandVocabulary();
  const heads = [...vocabulary.shell_tokens, ...vocabulary.bare_shell_tokens];
  const stops = [...wordsForRole('statement_function_word'), ...wordsForRole('skill_procedure_clause_separator')];
  const words = request.split(/\s+/u).filter(Boolean);
  const runs = (word) => vocabulary.run_verbs.includes(normalizeCommandWord(word)) || vocabulary.run_verbs.includes(bare(word))
    || vocabulary.cjk_run_verbs.some((verb) => bare(word).endsWith(verb));
  let verb = -1;
  words.forEach((word, index) => {
    if (runs(word)) verb = index;
  });
  if (verb < 0) return null;
  const commandFrom = (span) => {
    const command = [];
    for (const word of span) {
      if (stops.includes(bare(word))) break;
      const ends = COMMAND_ENDS.test(word);
      command.push(word.replace(COMMAND_PEEL, ''));
      if (ends) break;
    }
    return command.length && heads.includes(command[0]) ? command.join(' ') : null;
  };
  const after = commandFrom(words.slice(verb + 1));
  if (after !== null) return after;
  const head = words.slice(0, verb).findLastIndex((word) => heads.includes(word));
  return head < 0 ? null : commandFrom(words.slice(head, verb));
}

/** Mirrors `fn paths_in`: the paths `text` names, each once, in order. */
export function pathsIn(text) {
  return [...new Set(tokens(text).map((token) => cleanPathToken(token.text))
    .filter((path) => looksLikeFilePath(path) && safeRelativePath(path)))];
}

/**
 * The module-function request `task` states, or null: a seeded code construct
 * (`coding_request_object`) the request asks to write or add, a signature,
 * the module it goes in (the first path after the signature whose extension
 * names a language with a test contract), and — when the request names a test
 * (`coding_test_artifact_kind`) — the one other path, the test module.
 */
export function moduleFunctionRequest(task) {
  const outside = outsideQuotes(task);
  const normalized = normalizePrompt(outside).toLowerCase();
  if (!mentionsRole('coding_request_object', normalized)
    || !(mentionsRole('coding_request_verb', normalized) || mentionsRole('coding_member_add_action', normalized))) return null;
  const stated = signature(outside);
  if (!stated) return null;
  const parts = clauses(outside);
  const at = parts.findIndex((part) => signature(part)?.name === stated.name);
  // The module is the path the signature's own clause names, before or after
  // it ("math.mjs में एक फ़ंक्शन f(a, b) जोड़ो"); otherwise the first path
  // after the signature.
  const own = at < 0 ? [] : pathsIn(parts[at]);
  const paths = own.length === 1
    ? [own[0], ...pathsIn(outside).filter((path) => path !== own[0])]
    : pathsIn(outside.slice(stated.at));
  const module = paths[0];
  const language = module === undefined ? null : extensionLanguage(module);
  if (language === null || contract(language) === null || !programLanguageBySlug(language)) return null;
  const others = paths.slice(1);
  const wantsTest = mentionsRole('coding_test_artifact_kind', normalized);
  if (others.length > 1 || (wantsTest && others.length !== 1) || (!wantsTest && others.length)) return null;
  // The specification is the signature's clause, joined by the relative
  // clause right after it when the return is stated there ("…, которая
  // возвращает их сумму").
  const returnsIn = (part) => part.split(/\s+/u).some((word) => mentionsRole('coding_return_action', bare(word)));
  const relative = at >= 0 && !returnsIn(parts[at]) && at + 1 < parts.length && returnsIn(parts[at + 1]);
  const clause = at < 0 ? outside : relative ? `${parts[at]} ${parts[at + 1]}` : parts[at];
  return { ...stated, module, test: others[0] ?? null, language, clause, command: statedCommand(outside) };
}

/**
 * The binary operations the seed relates to a named arithmetic relation: each
 * `coding_fragment` of data/seed/coding-composition-fragments.lino whose idiom
 * is `{left} <operator> {right}`, with the reductions it `supports`
 * (`integer_add` supports `reduce_sum`, `integer_multiply` `reduce_product`).
 */
function relationOperations() {
  return cached('module_function:relations', () => {
    const root = parseLinoRoot(readText('data/seed/coding-composition-fragments.lino') ?? '').children[0];
    return (root?.children || []).map((node) => ({
      idiom: findChildValue(node, 'idiom'),
      supports: (node.children || []).filter((child) => child.name === 'supports').map((child) => String(child.id ?? child.value ?? '')),
      // The realization per language (the Python idiom is the canonical
      // surface) and whether the typed signature takes integers.
      realizations: Object.fromEntries((node.children || []).filter((child) => child.name === 'realization')
        .flatMap((child) => child.children || []).map((child) => [child.name, String(child.value ?? '')])),
      integer: findChildValue(node, 'fragment_signature').replace(/[()]/gu, '').split(/\s+/u)
        .every((type) => type === 'integer'),
    })).filter(({ idiom }) => /^\{left\} \S+ \{right\}$/u.test(idiom));
  });
}

/**
 * `left <operator> right` for the one arithmetic relation `text` names in any
 * seeded language ("their sum", "их сумму", "उनका योग", "它们的乘积"), or
 * null when it names none or several.
 */
function relationExpression(text, samples) {
  if (samples.length !== 2) return null;
  const normalized = normalizePrompt(text).toLowerCase();
  const named = relationOperations()
    .filter(({ supports }) => supports.some((relation) => meaningEvidencedIn(relation, normalized)));
  return named.length === 1 ? fill(named[0].idiom, [['left', samples[0]], ['right', samples[1]]]) : null;
}

/** The calculator's value of `expression`, or null. */
function evaluated(expression) {
  try {
    const value = realm().evaluateArithmetic(expression);
    return value === null || value === undefined ? null : String(value);
  } catch {
    return null;
  }
}

/**
 * The specification's value at `samples`, computed by the calculator, or
 * null: the clause after the seeded return action with the parameters bound
 * to the samples ("a times b" -> "2 times 3"), or, when that names no
 * operands, the arithmetic relation the clause names around its return
 * action applied to the samples in parameter order ("returns their sum" ->
 * "2 + 3"; the relation can precede the verb, as in "उनका योग लौटाता है").
 */
function specifiedValue(request, samples) {
  const words = request.clause.split(/\s+/u);
  const returns = words.findIndex((word) => mentionsRole('coding_return_action', bare(word)));
  if (returns < 0) return null;
  const afterSignature = request.clause.slice(request.clause.indexOf(')') + 1);
  return statedValue(words.slice(returns + 1), afterSignature, request.parameters, samples);
}

/**
 * Mirrors `fn stated_value` in rust/src/agentic_coding/module_function.rs: the
 * value a stated return computes at `samples` -- the longest expression the
 * words open with, its parameters bound to the samples ("a - b to m.mjs" ->
 * "2 - 3"; the words after it belong to the request, PR #1188 T92), or else
 * the arithmetic relation `relationText` names applied to the samples.
 */
export function statedValue(words, relationText, parameters, samples) {
  const bound = words.map((word) => {
    const index = parameters.indexOf(bare(word));
    return index < 0 ? word : samples[index];
  });
  for (let end = bound.length; end > 0; end -= 1) {
    const extracted = realm().extractArithmeticExpression(bound.slice(0, end).join(' '));
    if (extracted && extracted.expression) return evaluated(extracted.expression);
  }
  const relation = relationExpression(relationText, samples);
  return relation === null ? null : evaluated(relation);
}

/** The seeded function a language lowers a body into (`{language}_ir_function`), or null. */
function functionTemplate(language) {
  const root = cached('module_function:runtime-templates',
    () => parseLinoRoot(readText('data/seed/coding-discovery-runtime.lino') ?? ''));
  const template = (root.children || []).flatMap((node) => node.children || [])
    .find((node) => node.name === 'template' && String(node.id ?? node.value ?? '') === `${language}_ir_function`);
  return template ? findChildValue(template, 'text') || null : null;
}

/**
 * Mirrors `fn synthesized_source` in rust/src/agentic_coding/module_function.rs:
 * the one seeded integer operation whose idiom meets the specification at
 * every sample pair of the contract, applied to the parameters in order and
 * lowered through the language's function template; null when none or
 * several meet it.
 */
function searchedSource(request, terms) {
  const arity = request.parameters.length;
  const samples = findChildValue(terms, 'samples').split(/\s+/u).filter(Boolean);
  if (arity !== 2 || samples.length < arity) return null;
  const pairs = [];
  for (let index = 0; index + arity <= samples.length; index += arity) pairs.push(samples.slice(index, index + arity));
  const specified = pairs.map((pair) => specifiedValue(request, pair));
  if (specified.some((value) => value === null)) return null;
  const meeting = relationOperations().filter((operation) => operation.integer && pairs.every((pair, index) =>
    evaluated(fill(operation.idiom, [['left', pair[0]], ['right', pair[1]]])) === specified[index]));
  if (meeting.length !== 1) return null;
  const [operation] = meeting;
  const surface = request.language === 'python' ? operation.idiom : operation.realizations[request.language];
  const template = functionTemplate(request.language);
  if (!surface || template === null) return null;
  const expression = fill(surface, [['left', request.parameters[0]], ['right', request.parameters[1]]]);
  return fill(template, [['name', request.name], ['parameters', request.parameters.join(', ')], ['expression', expression]]);
}

/** `source` with `name` imported from `specifier` through the contract's import line. */
function withImport(source, template, name, specifier) {
  const [prefix, suffix] = fill(template, [['specifier', specifier]]).split('{names}');
  const lines = source.split('\n');
  const at = lines.findIndex((line) => line.startsWith(prefix) && line.endsWith(suffix));
  if (at >= 0) {
    const names = lines[at].slice(prefix.length, lines[at].length - suffix.length).split(',').map((part) => part.trim());
    if (!names.includes(name)) lines[at] = `${prefix}${[...names, name].join(', ')}${suffix}`;
    return lines.join('\n');
  }
  const head = prefix.split(/\s/u)[0];
  const last = lines.reduce((found, line, index) => (line.startsWith(head) ? index : found), -1);
  lines.splice(last + 1, 0, `${prefix}${name}${suffix}`);
  return lines.join('\n');
}

/** The import specifier of `module` from a file at `from`, both workspace-relative. */
function specifierFrom(from, module) {
  const base = from.split('/').slice(0, -1);
  const target = module.split('/');
  let shared = 0;
  while (shared < base.length && shared < target.length - 1 && base[shared] === target[shared]) shared += 1;
  const up = base.slice(shared).map(() => '..');
  const path = [...up, ...target.slice(shared)].join('/');
  return up.length ? path : `./${path}`;
}

const withFinalNewline = (text) => (text === '' || text.endsWith('\n') ? text : `${text}\n`);

/** The recipe that adds the function and its test and runs the check, or null. */
async function moduleFunctionRecipe(request, moduleSource, testSource) {
  const terms = contract(request.language);
  const catalog = programLanguageBySlug(request.language);
  // The module is where the function goes and the member-add verb how it
  // gets there, not what it computes: the clause is solved without them, so a
  // path never reads as a page to open and "जोड़ो" (add it) never as a sum.
  const adds = wordsForRole('coding_member_add_action');
  const specification = request.clause.split(request.module).join(' ').split(/\s+/u)
    .filter((word) => word !== '' && !adds.includes(bare(word))).join(' ');
  const answer = await solve(`${specification} ${catalog.name}`, []);
  const program = answer?.synthesized_program;
  // A body stated as an expression ("returns a - b") names no structure the
  // composer can discover; the seeded operation that meets the specification
  // at every sample pair is the function then, as natively (PR #1188 T92).
  const composed = program && program.language === request.language ? program.source : searchedSource(request, terms);
  if (composed === null) return null;
  const definition = fill(findChildValue(terms, 'definition'), [['name', request.name]]);
  const defined = moduleSource.split('\n').some((line) => line.startsWith(definition));
  const source = defined ? moduleSource
    : `${withFinalNewline(moduleSource)}${moduleSource.trim() === '' ? '' : '\n'}${composed}`;
  const commands = [catalog.execution.check_command]
    .filter((command) => command !== null && command !== undefined)
    .map((command) => command.split(catalog.save_as).join(request.module));
  const supporting = [];
  if (request.test !== null) {
    const samples = findChildValue(terms, 'samples').split(/\s+/u).slice(0, request.parameters.length);
    const expected = specifiedValue(request, samples);
    if (expected === null || samples.length !== request.parameters.length) return null;
    const specifier = specifierFrom(request.test, request.module);
    const base = testSource.trim() === '' ? findChildValue(terms, 'header') : withFinalNewline(testSource);
    const imported = withImport(base, findChildValue(terms, 'import'), request.name, specifier);
    const testCase = fill(findChildValue(terms, 'case'),
      [['name', request.name], ['arguments', samples.join(', ')], ['expected', expected]]);
    supporting.push({ path: request.test, source: imported.includes(testCase) ? imported : `${imported}${testCase}` });
  }
  const run = request.command ?? (request.test === null ? null : fill(findChildValue(terms, 'run'), [['test', request.test]]));
  if (run !== null) commands.push(run);
  return { language: request.language, source, path: request.module, supporting_files: supporting, commands };
}

/** The file `path` as the transcript's read returned it; '' for a missing file, null when unread. */
export function readSource(currentTurn, path) {
  const read = resultForPath(currentTurn, Capability.Read, path, null);
  if (read === null) return null;
  const missing = sourceFromAgentReadResult(read) === null && failureMessage(read, false, true) !== null;
  return missing ? '' : sourceFromReadResult(read);
}

/**
 * Plan the next step of a module-function request: read the module and the
 * test module, then hand the computed recipe to the execution-recipe reroute.
 */
export async function planModuleFunctionStep(task, messages, toolNames) {
  const request = moduleFunctionRequest(task);
  if (!request) return null;
  const currentTurn = messages.slice(evidenceWindowStart(messages));
  const readTool = toolFor(toolNames, Capability.Read);
  const sources = [];
  for (const path of [request.module, request.test].filter((path) => path !== null)) {
    const source = readSource(currentTurn, path);
    if (source === null) return readTool ? planOne(readTool, readArguments(path)) : null;
    sources.push(source);
  }
  const recipe = await moduleFunctionRecipe(request, sources[0], sources[1] ?? '');
  return recipe ? planSymbolicCommandReroute(messages, toolNames, { execution_recipe: recipe }) : null;
}
