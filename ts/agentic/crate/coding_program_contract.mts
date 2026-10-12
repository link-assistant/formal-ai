// Compose a process from explicit output requirements and source-backed
// operations (rust/src/coding/program_contract.rs).
//
// `programContractAnswer` returns a `SymbolicAnswer`-shaped object carrying
// an `execution_recipe`, or null.

import { cached, readText } from '../host.mjs';
import { pathExtension, typedWriteTarget } from '../write_request.mjs';
import { findChildValue, parseLinoRoot } from '../write_lino.mjs';
import { meaningEvidencedIn, mentionsRole } from '../write_lexicon.mjs';
import { lines } from '../write_str.mjs';
import { isFramework, programLanguageBySlug, programLanguages } from './coding_catalog.mjs';
import { normalizePrompt } from './engine.mjs';
import { requested } from './implementation_language.mjs';
import { boundOutputLiterals, obligationGapLines } from './intent_formalization_obligations.mjs';
import { quotedSegmentSpans } from './normal_markov.mjs';
import { fillWorkflowVersions, forGeneration } from './version_resolution.mjs';

function contracts() {
  return cached('write:stdout_program_contracts', () => parseLinoRoot(readText('data/meta/stdout-program-contracts.lino')).children[0] ?? null);
}

function contractFor(language) {
  return (contracts()?.children || []).find((node) => node.name === 'language' && node.id === language) ?? null;
}

/** @param {string} text @param {string} from @param {string} to */
const replaceText = (text, from, to) => text.split(from).join(to);

/** Mirrors `fn runtime_steps`: the language's CI setup steps, or null. */
export function runtimeSteps(language) {
  const contract = contractFor(language);
  return contract ? fillWorkflowVersions(findChildValue(contract, 'ci_setup'), forGeneration()) : null;
}

/** Mirrors `fn text_outside_quoted_segments` in rust/src/solver_handlers/text_manipulation.rs. */
export function textOutsideQuotedSegments(prompt) {
  let outside = '';
  let cursor = 0;
  for (const segment of quotedSegmentSpans(prompt)) {
    if (segment.start < cursor) continue;
    outside += `${prompt.slice(cursor, segment.start)} `;
    cursor = segment.end;
  }
  return outside + prompt.slice(cursor);
}

/**
 * Mirrors `fn explicit_stdout`: the output operands the obligation graph's
 * clauses bind (`boundOutputLiterals`), each once, in request order.
 */
export function explicitStdout(prompt) {
  const outputs = boundOutputLiterals(prompt);
  return outputs.length ? outputs.join('\n') : null;
}

/** Mirrors `fn program_language`. */
function programLanguage(prompt) {
  for (const line of lines(prompt)) {
    const normalized = normalizePrompt(textOutsideQuotedSegments(line));
    if (meaningEvidencedIn('coding_request_program', normalized)
      && (mentionsRole('program_request', normalized) || mentionsRole('coding_request_verb', normalized))) {
      const language = requested(normalized);
      if (language !== null && language !== undefined) return language;
    }
  }
  return namedSourceLanguage(prompt) ?? unnamedProgramLanguage(prompt);
}

/**
 * Mirrors `fn unnamed_program_language`: the seed's `unnamed_language` when
 * the request asks to write a program and to run it but names no language
 * (PR #1188 dogfooding). The run is the agent's own obligation, so the
 * program has to be written in something runnable; a request that only asks
 * for a program still asks which language (issue #906).
 */
function unnamedProgramLanguage(prompt) {
  const normalized = normalizePrompt(textOutsideQuotedSegments(prompt));
  const asks = meaningEvidencedIn('coding_request_program', normalized)
    && (mentionsRole('program_request', normalized) || mentionsRole('coding_request_verb', normalized))
    && mentionsRole('software_followup_execution', normalized);
  const language = asks ? findChildValue(contracts(), 'unnamed_language') : '';
  return language ? language : null;
}

/** Mirrors `fn source_path`: a bare-word file path, sentence marks peeled. */
function sourcePath(token) {
  return token.replace(/^[`"'(]+|[`"',;:.!?)]+$/gu, '');
}

/**
 * Mirrors `fn named_source_file`: the one relative source file with
 * `extension` the request names, cued or not ("Write a Python program
 * hello.py that prints …"), or null when it names none or two (PR #1188 T18).
 */
function namedSourceFile(prompt, extension) {
  const paths = textOutsideQuotedSegments(prompt).split(/\s+/u)
    .filter((token) => sourceExtension(token) === extension)
    .map(sourcePath)
    .filter((path) => !path.startsWith('/') && !path.split('/').includes('..'))
    .filter((path, index, all) => index === 0 || all[index - 1] !== path);
  return paths.length === 1 ? paths[0] : null;
}

/** Mirrors `fn source_extension`: a bare-word file's extension, or null. */
function sourceExtension(token) {
  const path = sourcePath(token);
  const extension = pathExtension(path);
  return extension !== null && /^[A-Za-z0-9]+$/u.test(extension) ? extension : null;
}

/**
 * Mirrors `fn named_source_language`: a line that asks for printed output
 * and names exactly one source file whose extension is the saved-file
 * extension of exactly one catalogued language (`hello.py` -> python) is a
 * program request in that language, the way a programmer reads the file
 * name (PR #1188 dogfooding). Only a line that asks to write or create one:
 * `Change greet.py so it prints "Hi"` edits a file, it does not replace it.
 */
function namedSourceLanguage(prompt) {
  for (const line of lines(prompt)) {
    const outside = textOutsideQuotedSegments(line);
    const normalized = normalizePrompt(outside);
    if (!meaningEvidencedIn('print_stdout', normalized) || !mentionsRole('coding_request_verb', normalized)) continue;
    const slugs = new Set();
    for (const token of outside.split(/\s+/u)) {
      const extension = sourceExtension(token);
      if (extension === null) continue;
      for (const language of programLanguages()) {
        if (!isFramework(language) && pathExtension(language.save_as) === extension) slugs.add(language.slug);
      }
    }
    if (slugs.size === 1) return [...slugs][0];
  }
  return null;
}

/** Mirrors `fn quotes_every_output`: every output the request binds is quoted in it. */
function quotesEveryOutput(prompt) {
  const quoted = quotedSegmentSpans(prompt).map((segment) => segment.text);
  return boundOutputLiterals(prompt).every((output) => quoted.includes(output));
}

/** Mirrors `fn asks_to_run`: the request asks for the program to be run, outside its quotes. */
function asksToRun(prompt) {
  return mentionsRole('software_followup_execution', normalizePrompt(textOutsideQuotedSegments(prompt)));
}

/**
 * Mirrors `fn answer` in rust/src/coding/program_contract.rs.
 * An output read in the open, without quotes (PR #1188 T18), binds the
 * contract only beside a named source file or a run; without one the request
 * is the catalog task the documentation route answers, in every locale
 * (issue #932).
 * The `SymbolicAnswer` is partial: `finalize_simple`'s event-log evidence
 * links, thinking steps and Links Notation trace are not rebuilt (native-only:
 * rust/src/solver_handlers/mod.rs finalize_simple; the JS host has no
 * EventLog), only `intent`, `answer`, `confidence` and `execution_recipe`.
 * The `obligation_gap` events the native run records
 * (`intent_formalization::record_obligation_gaps`, R1166-4) are carried as
 * `obligation_gaps`, one `obligation_gap_lines` line each, in order.
 * @param {string} prompt
 */
export function programContractAnswer(prompt) {
  const language = programLanguage(prompt);
  if (language === null) return null;
  const output = explicitStdout(prompt);
  if (output === null) return null;
  const catalog = programLanguageBySlug(language);
  if (!catalog) return null;
  const extension = pathExtension(catalog.save_as);
  if (extension === null) return null;
  const namedPath = typedWriteTarget(prompt, extension) ?? namedSourceFile(prompt, extension);
  if (namedPath === null && !quotesEveryOutput(prompt) && !asksToRun(prompt)) return null;
  const path = namedPath ?? catalog.save_as;
  const root = contracts();
  const contract = contractFor(language);
  if (!root || !contract) return null;
  const value = stringLiteral(output, findChildValue(contract, 'escape_characters'), findChildValue(contract, 'unicode_escape'));
  const body = replaceText(findChildValue(contract, 'operation'), '{value}', value);
  const entry = replaceText(findChildValue(contract, 'entry'), '{body}', body);
  const commands = [catalog.execution.check_command, catalog.execution.run_command]
    .filter((command) => command !== null && command !== undefined)
    .map((command) => replaceText(command, catalog.save_as, path));
  const comment = findChildValue(contract, 'comment');
  const sourceUrl = findChildValue(contract, 'source');
  const instructions = commands
    .map((command) => replaceText(replaceText(findChildValue(root, 'instruction'), '{comment}', comment), '{command}', command))
    .join('');
  const source = replaceText(replaceText(replaceText(findChildValue(root, 'documented_source'), '{instructions}', instructions), '{comment}', comment), '{source}', entry);
  const response = replaceText(replaceText(replaceText(findChildValue(root, 'response'), '{language}', language), '{source_url}', sourceUrl), '{source}', source);
  if (!commands.length) return null;
  const verifier = outputVerifier(output, commands[commands.length - 1]);
  if (!verifier) return null;
  commands[commands.length - 1] = `sh ${verifier.path}`;
  return {
    intent: 'write_program',
    answer: response,
    confidence: 1.0,
    evidence_links: [],
    thinking_steps: [],
    links_notation: '',
    execution_recipe: { language, source, path, supporting_files: [verifier], commands },
    obligation_gaps: obligationGapLines(prompt),
  };
}

/** Mirrors `fn output_verifier`. */
function outputVerifier(expected, command) {
  const contract = parseLinoRoot(readText('data/meta/process-verification.lino')).children[0];
  if (!contract) return null;
  const quoted = `'${replaceText(expected, "'", "'\\''")}'`;
  return {
    path: findChildValue(contract, 'path'),
    source: replaceText(replaceText(findChildValue(contract, 'template'), '{command}', command), '{expected}', quoted),
  };
}

/** Rust `char::is_control`: general category Cc. */
const isControl = (character) => /\p{Cc}/u.test(character);

/** Mirrors `fn string_literal`. */
export function stringLiteral(value, extraEscapes, unicodeEscape) {
  let out = '"';
  for (const character of value) {
    if (character === '"') out += '\\"';
    else if (character === '\\') out += '\\\\';
    else if (character === '\n') out += '\\n';
    else if (character === '\r') out += '\\r';
    else if (character === '\t') out += '\\t';
    else if (isControl(character)) {
      const code = character.codePointAt(0);
      let escaped = replaceText(unicodeEscape, '{hex4}', code.toString(16).padStart(4, '0'));
      escaped = replaceText(escaped, '{hex}', code.toString(16));
      out += replaceText(replaceText(escaped, '{{', '{'), '}}', '}');
    } else {
      if (extraEscapes.includes(character)) out += '\\';
      out += character;
    }
  }
  return `${out}"`;
}

/**
 * Mirrors `fn attach_execution_recipe` in rust/src/coding/synthesis_runtime.rs:
 * a synthesized function travels as a typed recipe — its source saved as the
 * catalog's file and checked with the catalog's check command — so an agent
 * protocol can write it and run it with its own tools (PR #1188 dogfooding).
 * A function is checked, never run as a program.
 */
export function attachExecutionRecipe(answer, program) {
  if (!program || answer.execution_recipe) return answer;
  const language = programLanguageBySlug(program.language);
  if (!language) return answer;
  const commands = [language.execution.check_command].filter((command) => command !== null && command !== undefined);
  return {
    ...answer,
    execution_recipe: { language: program.language, source: program.source, path: language.save_as, supporting_files: [], commands },
  };
}

/**
 * Mirrors `fn function_call_command`: the command that calls the function a
 * recipe's `source` defines with `args`, printing its result — from the
 * language's `definition` and `call` contract templates — or null when the
 * language has none, no definition matches, or the argument count differs.
 */
export function functionCallCommand(language, path, source, args) {
  const contract = contractFor(language);
  if (!contract || !args.length) return null;
  const call = findChildValue(contract, 'call');
  const signature = definedFunction(source, findChildValue(contract, 'definition'));
  if (!call || !signature || signature.parameters.length !== args.length) return null;
  const module = path.replace(/\.[^./]+$/u, '').split('/').join('.');
  return replaceText(replaceText(replaceText(call, '{module}', module), '{name}', signature.name), '{arguments}', args.join(', '));
}

/** Mirrors `fn defined_function`: `{name, parameters}` of the first definition `template` matches. */
function definedFunction(source, template) {
  const [prefix, afterName] = String(template || '').split('{name}');
  if (!prefix || afterName === undefined) return null;
  const [middle, suffix] = afterName.split('{parameters}');
  if (!middle || suffix === undefined) return null;
  for (const raw of lines(source)) {
    const line = raw.trimStart();
    if (!line.startsWith(prefix)) continue;
    const rest = line.slice(prefix.length);
    const at = rest.indexOf(middle);
    const tail = at > 0 ? rest.slice(at + middle.length) : '';
    const end = tail.lastIndexOf(suffix);
    const name = at > 0 ? rest.slice(0, at) : '';
    if (end < 0 || !/^[A-Za-z_][A-Za-z0-9_]*$/u.test(name)) continue;
    const parameters = tail.slice(0, end).split(',').map((part) => part.split(/[:=]/u)[0].trim()).filter(Boolean);
    return { name, parameters };
  }
  return null;
}
