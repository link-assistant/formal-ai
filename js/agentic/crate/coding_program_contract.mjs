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
  return namedSourceLanguage(prompt);
}

/** Mirrors `fn source_extension`: a bare-word file's extension, or null. */
function sourceExtension(token) {
  const path = token.replace(/^[`"'(]+|[`"',;:.!?)]+$/gu, '');
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

/**
 * Mirrors `fn answer` in rust/src/coding/program_contract.rs.
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
  const path = typedWriteTarget(prompt, extension) ?? catalog.save_as;
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
