// Workspace source artifacts and bounded rewrites of them
// (rust/src/agentic_coding/code_artifact.rs).

import { Capability } from './capability.mjs';
import { classifyTool, toolFor } from './capability_router.mjs';
import { plainText } from './content.mjs';
import { agenticMessage } from './messages.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { programLanguageByAlias, programSaveAs, programTaskByAlias, programTemplate } from './crate/coding_catalog.mjs';
import { escapeReference } from './crate/links_format.mjs';
import { parseSubstitutionQuery, renderSubstitutionQuery, substitutionEffect } from './crate/links_substitution_query.mjs';
import {
  executeRewrite, quotedSegments, rewriteProgram, rewriteRule, unwrapTransportQuotes,
} from './crate/normal_markov.mjs';
import { isAsciiDigit, lines, trim, trimEnd } from './write_str.mjs';

const MAX_REWRITE_STEPS = 100000;
const RENDERED_TRACE_EDGE_STEPS = 32;

const eqIgnoreAsciiCase = (left, right) => left.replace(/[A-Z]/g, (c) => c.toLowerCase()) === right.replace(/[A-Z]/g, (c) => c.toLowerCase());

/** Rust `Debug` of `RewriteHalt`. */
function haltDebug(halt) {
  if (halt.kind === 'terminal_rule') return `TerminalRule(${halt.index})`;
  return halt.kind === 'step_limit' ? 'StepLimit' : 'NoApplicableRule';
}

function linkField(indent, name, value) {
  return `${' '.repeat(indent)}${name} ${escapeReference(value)}\n`;
}

/** Mirrors `WorkspaceRewrite::links_notation`. */
function rewriteLinksNotation(rewrite, outcome) {
  let links = 'normal_markov_program\n';
  links += linkField(2, 'target', rewrite.target);
  links += linkField(2, 'substitution_query', renderSubstitutionQuery(rewrite.program));
  links += linkField(2, 'max_steps', String(rewrite.program.max_steps));
  rewrite.program.rules.forEach((rule, index) => {
    links += `  rewrite_rule "${index}"\n`;
    links += linkField(4, 'effect', substitutionEffect(rule));
    links += linkField(4, 'pattern', rule.pattern);
    links += linkField(4, 'replacement', rule.replacement);
    links += linkField(4, 'terminal', String(rule.terminal));
  });
  if (outcome) {
    links += '  execution\n';
    links += linkField(4, 'halt', haltDebug(outcome.halt));
    links += linkField(4, 'steps', String(outcome.trace.length));
    const omitted = Math.max(0, outcome.trace.length - RENDERED_TRACE_EDGE_STEPS * 2);
    outcome.trace.forEach((step, index) => {
      if (omitted > 0 && index === RENDERED_TRACE_EDGE_STEPS) links += linkField(4, 'omitted_steps', String(omitted));
      if (omitted > 0 && index >= RENDERED_TRACE_EDGE_STEPS && index < outcome.trace.length - RENDERED_TRACE_EDGE_STEPS) return;
      links += `    applied rule=${step.rule_index} byte_offset=${step.byte_offset}\n`;
    });
  }
  return links;
}

/**
 * Mirrors `fn plan_code_artifact_step`.
 * @param {string} rawTask
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planCodeArtifactStep(rawTask, messages, toolNames) {
  const task = unwrapTransportQuotes(rawTask);
  const writeTool = toolFor(toolNames, Capability.Write);
  if (!writeTool) return null;
  let latestUser = -1;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (eqIgnoreAsciiCase(messages[index].role, 'user')) {
      latestUser = index;
      break;
    }
  }
  if (latestUser < 0) return null;
  const current = messages.slice(latestUser + 1);
  const artifact = latestWorkspaceArtifact(messages.slice(0, latestUser));
  if (artifact) {
    const rewrite = requestedRewrite(task, artifact);
    if (!rewrite) return null;
    const result = latestResult(current, Capability.Write);
    if (result !== null) {
      const read = latestResult(current, Capability.Read);
      const outcome = read === null ? null : executeRewrite(rewrite.program, sourceFromReadResult(read));
      return finalAnswer(agenticMessage('code_artifact_updated', {
        path: artifact.path, links: rewriteLinksNotation(rewrite, outcome), result: trim(result),
      }));
    }
    const readSource = latestResult(current, Capability.Read);
    if (readSource !== null) {
      const currentSource = sourceFromReadResult(readSource);
      const outcome = executeRewrite(rewrite.program, currentSource);
      if (outcome.halt.kind === 'step_limit') {
        return finalAnswer(agenticMessage('code_artifact_step_limit', {
          path: artifact.path, max_steps: MAX_REWRITE_STEPS, links: rewriteLinksNotation(rewrite, outcome),
        }));
      }
      if (outcome.output === currentSource) return finalAnswer(agenticMessage('code_artifact_no_match', { path: artifact.path }));
      return planOne(writeTool, writeArguments(artifact.path, outcome.output));
    }
    const readTool = toolFor(toolNames, Capability.Read);
    return readTool ? planOne(readTool, readArguments(artifact.path)) : null;
  }
  if (toolFor(toolNames, Capability.Run)) return null;
  const generated = generatedArtifact(task);
  if (!generated) return null;
  if (latestResult(current, Capability.Write) !== null) {
    return finalAnswer(agenticMessage('code_artifact_created', { path: generated.path }));
  }
  return planOne(writeTool, writeArguments(generated.path, generated.content));
}

/** Mirrors `fn catalog_claims`. */
export function catalogClaims(task) {
  return generatedArtifact(task) !== null;
}

function generatedArtifact(task) {
  const normalized = task.toLowerCase();
  const language = programLanguageByAlias(normalized);
  if (!language) return null;
  const programTask = programTaskByAlias(normalized);
  if (!programTask) return null;
  const template = programTemplate(programTask.slug, language.slug);
  if (!template) return null;
  return { path: programSaveAs(programTask.slug, language), content: `${trimEnd(template.code)}\n` };
}

function requestedRewrite(task, artifact) {
  const explicit = explicitSubstitutionQuery(task);
  if (explicit) return { target: artifact.path, program: explicit };
  if (task.includes('?') || task.includes('？')) return null;
  const quoted = quotedSegments(task);
  let pairs;
  if (quoted.length === 1) {
    const last = lastStringLiteral(artifact.content);
    if (last === null) return null;
    pairs = [[last, quoted[0]]];
  } else if (quoted.length && quoted.length % 2 === 0) {
    pairs = [];
    for (let index = 0; index < quoted.length; index += 2) pairs.push([quoted[index], quoted[index + 1]]);
  } else return null;
  if (pairs.some(([old, next]) => old === next)) return null;
  const single = pairs.length === 1;
  const rules = pairs.map(([old, next]) => rewriteRule(old, next, single || old === ''));
  return { target: artifact.path, program: rewriteProgram(rules, MAX_REWRITE_STEPS) };
}

function explicitSubstitutionQuery(task) {
  const trimmed = trim(task);
  if (!trimmed.startsWith('(')) return null;
  const program = parseSubstitutionQuery(trimmed, MAX_REWRITE_STEPS);
  return program && program.rules.length ? program : null;
}

function parseArguments(text) {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function argumentString(value, keys) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) return null;
  for (const key of keys) {
    if (typeof value[key] === 'string') return value[key];
  }
  return null;
}

function latestWorkspaceArtifact(messages) {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const calls = messages[index].tool_calls || [];
    for (let at = calls.length - 1; at >= 0; at -= 1) {
      const call = calls[at];
      if (classifyTool(call.function.name) !== Capability.Write) continue;
      const args = parseArguments(call.function.arguments);
      const path = argumentString(args, ['path', 'filePath', 'file_path']);
      const content = argumentString(args, ['content']);
      if (path !== null && content !== null) return { path, content };
    }
  }
  return null;
}

/** Mirrors `fn latest_result`: the latest `capability` tool result text, or null. */
export function latestResult(messages, capability) {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (!eqIgnoreAsciiCase(message.role, 'tool')) continue;
    const callId = message.tool_call_id;
    if (callId === null || callId === undefined) continue;
    let call = null;
    for (const prior of messages.slice(0, index)) {
      call = (prior.tool_calls || []).find((candidate) => candidate.id === callId) ?? null;
      if (call) break;
    }
    if (!call) continue;
    if (classifyTool(call.function.name) === capability) return plainText(message.content);
  }
  return null;
}

/** Mirrors `fn source_from_read_result`: the file bytes a read tool printed. */
export function sourceFromReadResult(result) {
  const agent = sourceFromAgentReadResult(result);
  if (agent !== null) return agent;
  const open = result.indexOf('<content>\n');
  if (open < 0) return result;
  const afterOpen = result.slice(open + '<content>\n'.length);
  const close = afterOpen.lastIndexOf('\n</content>');
  if (close < 0) return result;
  let decoded = '';
  for (const line of lines(afterOpen.slice(0, close))) {
    if (line.startsWith('(End of file - total ')) break;
    const split = line.indexOf(': ');
    const numbered = split >= 0 && Array.from(line.slice(0, split)).every(isAsciiDigit);
    decoded += `${numbered ? line.slice(split + 2) : line}\n`;
  }
  while (decoded.endsWith('\n\n')) decoded = decoded.slice(0, -1);
  return decoded;
}

function sourceFromAgentReadResult(result) {
  if (!result.startsWith('<file>\n')) return null;
  const afterOpen = result.slice('<file>\n'.length);
  const marker = '\n\n(End of file - total ';
  const at = afterOpen.lastIndexOf(marker);
  if (at < 0) return null;
  if (!afterOpen.slice(at + marker.length).endsWith('</file>')) return null;
  const out = [];
  for (const line of lines(afterOpen.slice(0, at))) {
    const split = line.indexOf('| ');
    if (split < 0 || !Array.from(line.slice(0, split)).every(isAsciiDigit)) return null;
    out.push(line.slice(split + 2));
  }
  return out.join('\n');
}

function readArguments(path) {
  return jsonText({ path, filePath: path, file_path: path });
}

function lastStringLiteral(source) {
  const segments = quotedSegments(source);
  return segments.length ? segments[segments.length - 1] : null;
}
