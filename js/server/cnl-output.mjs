// Server policy adapter. The shared projection never changes tool arguments.
// Native adapter: rust/src/cnl_output/configuration.rs and rust/src/protocol.rs.
import {parseLino, readRepoFile} from './lino.mjs';
import {formalizeOutput, outputConfiguration, outputDirective, outputSource, outputTurn, renderOutput, validateOutput} from '../agentic/crate/cnl_output.mjs';
import {CNL_POLICY_PATH, parseOutputPolicy} from '../agentic/crate/cnl_output.mjs';
import {CNL_LANGUAGE_PATH} from '../agentic/crate/cnl_grammar.mjs';

/** Each language's grammar file, as rust/src/cnl_output.rs `cnl_policy` reads it. */
function languageGrammar(language) {
  try {
    return readRepoFile(CNL_LANGUAGE_PATH.replace('{language}', language)) || null;
  } catch {
    return null;
  }
}

let policy;
export function serverOutputPolicy() {
  return policy ??= parseOutputPolicy(readRepoFile(CNL_POLICY_PATH), parseLino, languageGrammar);
}

export function serverOutputConfiguration(value = {}) {
  return outputConfiguration(value, serverOutputPolicy());
}

export function requestOutputTurn(prompt, history, request, defaults = {}) {
  let configuration = serverOutputConfiguration(defaults);
  if (request.output_standard != null) {
    const named = serverOutputConfiguration({output_standard: request.output_standard});
    configuration = named.lexical_density === 'standard'
      ? {...configuration, output_standard: named.output_standard}
      : {...configuration, lexical_density: named.lexical_density};
  }
  if (request.lexical_density != null) {
    configuration = {...configuration, lexical_density: request.lexical_density};
  }
  return outputTurn(prompt, history, configuration, serverOutputPolicy());
}

/** Optional extensions are absent on unchanged requests, preserving wire DTOs. */
export function parseOutputOptions(object, optionalString) {
  const options = {};
  for (const key of ['output_standard', 'lexical_density']) {
    const value = optionalString(object, key);
    if (value !== null) options[key] = value;
  }
  serverOutputConfiguration(options);
  return options;
}

/**
 * Rust carries `output_projection` as a `serde_json::Value` built without
 * `preserve_order` (rust/src/protocol/output.rs), so its object keys
 * serialise sorted; emit the same canonical order on the wire.
 */
function canonicalKeys(value) {
  if (Array.isArray(value)) return value.map(canonicalKeys);
  if (value === null || typeof value !== 'object') return value;
  return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonicalKeys(value[key])]));
}

/** The protocol may preserve a body already composed under request constraints. */
export async function projectServerAnswer(ctx, symbolic, configuration, {language: forcedLanguage, preserveAnswerBody = false, answerText} = {}) {
  const policy = serverOutputPolicy();
  const retained = symbolic.output_projection;
  const seed = await ctx.worker.seedReaders();
  const source = answerText ?? symbolic.answer;
  const sameAnswer = retained && (retained.content === source || retained.text === source);
  const language = forcedLanguage ?? (sameAnswer ? retained.document.language : seed.detectLanguage(source));
  const text = preserveAnswerBody ? source : outputSource(source, language, policy);
  const document = sameAnswer && retained.document.language === language ? retained.document : formalizeOutput(text, language, policy);
  const rendered = renderOutput(document, configuration, policy);
  if (preserveAnswerBody) {
    rendered.content = symbolic.answer;
    if (rendered.text !== text) {
      rendered.text = text;
      rendered.report = validateOutput(text, language, configuration, policy);
    }
  }
  return {...symbolic, answer: rendered.content, output_projection: canonicalKeys(rendered)};
}

export function hasOutputDirective(prompt) {
  return outputDirective(prompt, serverOutputPolicy()) !== null;
}
