// Resume Hive Mind's existing working tree instead of reimplementing its
// issue: the JavaScript twin of rust/src/agentic_coding/restart_feedback.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { rustLines } from './content.mjs';
import { resolvesBody, shellQuote } from './git_commit.mjs';
import { agenticMessage } from './messages.mjs';
import { finalAnswer, jsonText, planOne } from './plan.mjs';
import { Progress } from './progress.mjs';
import { shellStep, observedPayload, reportedExitCode, harnessReportedFailure, incompleteReceipt } from './tool_result.mjs';
import { fill } from './work_item_steps.mjs';
import { isObject, parseJson, splitOnce, splitWhitespace, trim, trimMatches } from './crate/rust_str.mjs';
import { cached, childrenNamed, childValue, parseLino, readText } from './host.mjs';
import { programLanguages } from './crate/coding_catalog.mjs';
const HYGIENE_POLICY = 'data/meta/restart-hygiene.lino';
function hygienePolicy() {
  return cached('restart-hygiene-policy', () => {
    const root = parseLino(readText(HYGIENE_POLICY));
    return {
      source: [...childrenNamed(parseLino(readText('data/meta/workspace-source-io.lino')), 'source-line'), ...childrenNamed(root, 'source-line')].map(node => node.value).join('\n') + '\n',
      outputFlags: childrenNamed(root, 'output-flag').map(node => node.value),
      binaryPrefixes: childrenNamed(root, 'binary-prefix').map(node => node.value),
      evidenceRoot: childValue(root, 'evidence-root'),
      evidenceSuffixes: childrenNamed(root, 'empty-evidence-suffix').map(node => node.value),
      command: childValue(root, 'command'),
      untrack: childValue(root, 'untrack-command'),
      clean: childValue(root, 'clean-command'),
      indexGuard: childValue(root, 'index-guard-command'),
      statusCommand: childValue(root, 'status-command'),
      scopeRefusal: childValue(root, 'scope-refusal'),
      commitCommand: childValue(root, 'commit-command')
    };
  });
}
function hygieneOutputs(policy) {
  const outputs = [];
  const portable = value => /^[A-Za-z0-9_.\/-]+$/u.test(value) && !value.startsWith('/') && !value.split('/').includes('..');
  for (const language of programLanguages()) {
    const source = language.save_as;
    const words = (language.execution.check_command ?? '').split(/\s+/u);
    if (!portable(source) || !words.includes(source)) continue;
    for (let index = 0; index + 1 < words.length; index += 1) {
      const output = words[index + 1];
      if (policy.outputFlags.includes(words[index]) && portable(output) && output !== source) {
        // Shared executable operands retain each distinct documented source owner.
        if (!outputs.some(entry => entry.output === output && entry.source === source)) {
          outputs.push({
            source,
            output
          });
        }
      }
    }
  }
  return outputs;
}
/**
 * Source-owned compiler outputs and empty external logs are kept physically,
 * excluded from Git, and never confused with listed source files.
 * Mirrors `fn prepared_work_hygiene`.
 */
export function preparedWorkHygiene(paths) {
  return workHygieneWithPolicy(paths, null);
}
function workHygieneWithPolicy(paths, sessionPolicy) {
  const policy = hygienePolicy();
  const outputs = hygieneOutputs(policy);
  const ignored = paths.filter(value => outputs.some(entry => entry.output === value) || value.startsWith(policy.evidenceRoot) && policy.evidenceSuffixes.some(suffix => value.endsWith(suffix)));
  const plan = {
    files: paths,
    outputs,
    ignored,
    'binary-prefixes': policy.binaryPrefixes,
    'evidence-root': policy.evidenceRoot,
    'empty-evidence-suffixes': policy.evidenceSuffixes,
    'session-policy': sessionPolicy,
    'session-contract': sessionContract()
  };
  const command = policy.command.replace(/\{(?:source|plan)\}/gu, placeholder => shellQuote(placeholder === '{source}' ? policy.source : jsonText(plan)));
  const excluded = [...new Set([...paths, '.gitignore'])].map(value => shellQuote(':(exclude,literal)' + value)).join(' ');
  const indexGuardCommand = policy.indexGuard.replace('{exclude}', () => excluded);
  const commands = [indexGuardCommand, command];
  if (ignored.length) commands.push(policy.untrack.replace('{files}', () => ignored.map(value => shellQuote(':(literal)' + value)).join(' ')));
  return {
    command: commands.join(' && '),
    commitFiles: [...paths.filter(value => !ignored.includes(value)), ...(ignored.length ? ['.gitignore'] : [])].filter((value, index, values) => values.indexOf(value) === index)
    ,
    generatedIndexRemovals: ignored,
    indexGuardCommand,
    cleanCommand: policy.clean
  };
}
function porcelainPaths(status) {
  if (status.length > 65536 || new TextEncoder().encode(status).length > 65536) return null;
  const paths = [];
  for (const line of rustLines(status).filter(candidate => trim(candidate) !== '')) {
    const bytes = new TextEncoder().encode(line);
    if (bytes.length < 4 || bytes[2] !== 32 || (bytes[3] & 0xc0) === 0x80) return null;
    if (!STATUS_BYTES.has(String.fromCharCode(bytes[0])) || !STATUS_BYTES.has(String.fromCharCode(bytes[1]))) return null;
    const path = line.slice(3);
    if (path === '' || path.startsWith('"') || path.includes(' -> ') || path.includes('\0') || path.startsWith('/') || path.split('/').includes('..')) {
      return null;
    }
    if (!paths.includes(path)) paths.push(path);
    if (paths.length > 256) return null;
  }
  return paths;
}
const STATUS_BYTES = new Set(Array.from(' MADRCU?!'));

/** Mirrors `fn changed_paths`: the porcelain listing's paths, or null. */
export function changedPaths(task) {
  const marked = splitOnce(task, agenticMessage('restart_feedback_uncommitted_marker'));
  if (marked === null) return null;
  const fenced = splitOnce(marked[1], '```');
  if (fenced === null) return null;
  const opened = splitOnce(fenced[1], '\n');
  if (opened === null) return null;
  const closed = splitOnce(opened[1], '```');
  if (closed === null) return null;
  const paths = porcelainPaths(closed[0]);
  return paths?.length ? paths : null;
}

/**
 * Mirrors `fn feedback_needs_changes` in rust/src/agentic_coding/restart_feedback.rs.
 * @param {string} output
 */
export function feedbackNeedsChanges(output) {
  const direct = parseJson(output);
  const directFeedback = isObject(direct) && Array.isArray(direct.comments) && Array.isArray(direct.reviews);
  const exit = reportedExitCode(output);
  if (harnessReportedFailure(output) || incompleteReceipt(output) || exit !== null && exit !== 0 || !directFeedback && exit !== 0) return true;
  const text = directFeedback ? output : observedPayload(output) ?? shellStep(output)?.text ?? output;
  const value = parseJson(text);
  if (value === undefined) return true;
  if (!isObject(value) || !Array.isArray(value.comments) || !Array.isArray(value.reviews)) return true;
  return value.comments.some(item => isObject(item) && typeof item.body === 'string' && trim(item.body) !== '') || value.reviews.some(item => isObject(item)
    && item.state === 'CHANGES_REQUESTED');
}
const REFERENCE_TRIM = new Set(['`', ')', ',', '.']);

/**
 * Mirrors `fn plan_restart` in rust/src/agentic_coding/restart_feedback.rs.
 * @param {string} task
 * @param {object[]} messages
 * @param {string[]} tools
 */
export function planRestart(task, messages, tools) {
  const listed = changedPaths(task);
  if (listed === null) return null;
  const word = splitWhitespace(task).find(candidate => candidate.startsWith('https://github.com/') && candidate.includes('/pull/'));
  if (word === undefined) return null;
  const reference = trimMatches(word, character => REFERENCE_TRIM.has(character));
  const progress = Progress.scan(messages);
  const run = toolFor(tools, Capability.Run);
  if (run === null) return null;
  const policy = hygienePolicy();
  const statusOutput = progress.latestSuccessfulRunOutputFor(policy.statusCommand);
  if (statusOutput === null || statusOutput === undefined) {
    const failed = progress.latestRunOutputFor(policy.statusCommand);
    if (failed !== null && failed !== undefined) return finalAnswer(fill('prepared_work_stopped_report', [['{command}', policy.statusCommand], ['{output}', failed]]));
    return planOne(run, jsonText({
      command: policy.statusCommand
    }));
  }
  const current = porcelainPaths(observedPayload(statusOutput) ?? statusOutput);
  if (current === null || current.some(path => !listed.some(scope => path === scope || scope.endsWith('/') && path.startsWith(scope)))) {
    return finalAnswer(policy.scopeRefusal);
  }
  // A fresh clean status is a no-op for recovery; issue obligations still belong
  // to the normal completion path, never to this working-tree repair route.
  if (current.length === 0) return null;
  const hygiene = preparedWorkHygiene(current);
  const commit = preparedWorkCommitCommand(hygiene, reference);
  const commands = [hygiene.command, commit, hygiene.cleanCommand];
  for (const command of commands) {
    if (progress.successfulRunCountFor(command) > 0) continue;
    const output = progress.latestRunOutputFor(command);
    if (output !== null && output !== undefined) return finalAnswer(fill('prepared_work_stopped_report', [['{command}', command], ['{output}', output]]));
    return planOne(run, jsonText({
      command
    }));
  }
  return null;
}

/** Mirrors `fn prepared_work_commit_command`; preserves the actual client branch. */
export function preparedWorkCommitCommand(hygiene, reference) {
  const policy = hygienePolicy();
  const stageFiles = hygiene.commitFiles.map(value => shellQuote(':(literal)' + value)).join(' ');
  return policy.commitCommand.replace(/\{(?:guard|files|subject|body|branch)\}/gu, placeholder => ({
    '{guard}': hygiene.indexGuardCommand,
    '{files}': stageFiles,
    '{subject}': shellQuote(agenticMessage('restart_feedback_commit_subject')),
    '{body}': shellQuote(resolvesBody(reference)),
    '{branch}': shellQuote('HEAD')
  })[placeholder]);
}
function sessionContract() {
  const root = parseLino(readText('data/meta/prepared-artifact-inventory.lino'));
  return {
    rootPattern: childValue(root, 'evidence-pattern'),
    pattern: childValue(root, 'session-pattern'),
    maximumBytes: Number(childValue(root, 'session-limit')),
    filename: 'session-id.txt'
  };
}

/** Mirrors `fn prepared_recipe_hygiene_with_inventory`; only caller-qualified inventory grants scope. */
export function preparedRecipeHygiene(sourcePaths, changed, inventory = null) {
  const policy = hygienePolicy();
  const outputs = hygieneOutputs(policy).filter(entry => sourcePaths.includes(entry.source));
  let sessionPolicy = null;
  let owned = [];
  if (inventory !== null) {
    const contract = sessionContract();
    const root = inventory.evidenceRoot;
    if (typeof root !== 'string' || new RegExp(contract.rootPattern, 'u').exec(root)?.[0] !== root || jsonText(inventory.sourcePaths) !== jsonText(sourcePaths)
    || !Array.isArray(inventory.ownedPaths) || inventory.ownedPaths.length > 256 || new Set(inventory.ownedPaths).size !== inventory.ownedPaths.length) return null;
    sessionPolicy = {
      evidenceRoot: root,
      filename: contract.filename,
      path: root + contract.filename,
      pattern: contract.pattern,
      maximumBytes: contract.maximumBytes
    };
    if (jsonText(inventory.sessionPolicy) !== jsonText(sessionPolicy)) return null;
    const installed = parseLino(readText('data/meta/prepared-artifact-inventory.lino'));
    const members = [...childrenNamed(installed, 'stream')].map(node => root + node.value);
    members.push(sessionPolicy.path);
    if (!inventory.ownedPaths.every(path => members.includes(path) || outputs.some(entry => entry.output === path))) return null;
    owned = inventory.ownedPaths;
  }
  const known = changed.every(path => sourcePaths.includes(path) || outputs.some(entry => entry.output === path) || (inventory !== null ? path === sessionPolicy.path
    || [...childrenNamed(parseLino(readText('data/meta/prepared-artifact-inventory.lino')), 'stream')].some(node => path === inventory.evidenceRoot + node.value) : path.startsWith(policy.evidenceRoot)
    && policy.evidenceSuffixes.some(suffix => path.endsWith(suffix))));
  return known ? workHygieneWithPolicy([...new Set([...changed, ...owned])], sessionPolicy) : null;
}

/** Mirrors `fn prepared_work_paths`; caller binds the exact status receipt. */
export function preparedWorkPaths(output) {
  const payload = observedPayload(output);
  return payload === null ? null : porcelainPaths(payload);
}
