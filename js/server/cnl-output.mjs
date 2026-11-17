// Server policy adapter. The shared projection never changes tool arguments.
// Native adapter: rust/src/cnl_output/configuration.rs and rust/src/protocol.rs.
import {parseLino, readRepoFile} from './lino.mjs';
import {formalizeOutput, outputConfiguration, outputDirective, outputSource, outputTurn, renderOutput} from '../agentic/crate/cnl_output.mjs';
import {parseOutputPolicy} from '../agentic/crate/cnl_output.mjs';

let policy;
export function serverOutputPolicy() {
  return policy ??= parseOutputPolicy(readRepoFile('data/seed/cnl-output.lino'), parseLino);
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

/** Language comes from the worker's detector, including its configured override. */
export async function projectServerAnswer(ctx, symbolic, configuration) {
  const policy = serverOutputPolicy();
  const retained = symbolic.output_projection;
  const seed = await ctx.worker.seedReaders();
  const language = retained?.document?.language ?? seed.detectLanguage(symbolic.answer);
  const text = outputSource(symbolic.answer, language, policy);
  const document = retained?.content === symbolic.answer ? retained.document : formalizeOutput(text, language, policy);
  const rendered = renderOutput(document, configuration, policy);
  return {...symbolic, answer: rendered.content, output_projection: rendered};
}

export function hasOutputDirective(prompt) {
  return outputDirective(prompt, serverOutputPolicy()) !== null;
}
