import { applyPatchInput } from './crate/protocol_responses_apply_patch.mjs';
// Route command-bearing symbolic answers through an agentic CLI's real tools
// (rust/src/agentic_coding/command_reroute.rs).
//
// `symbolicAnswer` is the Rust `SymbolicAnswer` shape `{intent, answer,
// confidence, evidence_links, thinking_steps, links_notation,
// execution_recipe?}`; an `ExecutionRecipe` is `{language, source, path,
// supporting_files: [{path, source}], commands}`. The JavaScript host
// solver's answers carry no `execution_recipe`, so for them this route never
// applies; `crate/coding_program_contract.mjs` builds answers that do.

import { lower as canonicalUnaryLower } from './crate/rust_str.mjs';
import { Capability } from './capability.mjs';
import { classifyTool, isWorkspaceCreationTool, toolFor } from './capability_router.mjs';
import { latestUserRequest, plainText, rustLines } from './content.mjs';
import { pushRef, recipeCommitCommand, shellQuote, targetOf } from './git_commit.mjs';
import { attachExecutionRecipe, functionCallCommand, programRecoveryCommand } from './crate/coding_program_contract.mjs';
import { normalizeCommandWord } from './shell_command_policy.mjs';
import { pathExtension, tokens, typedWriteTarget } from './write_request.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { terminalCommandVocabulary } from './crate/seed_terminal_commands.mjs';
import { agenticMessage } from './messages.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { planRecovery, planRecoveryWithIdentity } from './prerequisite_recovery.mjs';
import { Progress, qualifiedToolAttempt, qualifiedCommandAttempt, writeMatches } from './progress.mjs';
import { recipeSourceCommand, recipeSourceResult, recipeSourceMatches, recipeSourceUpdateCommand, recipeSourceUpdateResult, planRecipeCiFallback
    , recipeDeliveryReference, recipeSourceRefusal } from './recipe_resume.mjs';
import { validateProgram } from './host.mjs';
import { requestedIn as requestedCiWorkflow, attach as attachCiWorkflow } from './ci_workflow.mjs';
import { MAX_REPAIR_RUNGS, failedStep, repairStep, stopNote } from './repair_loop.mjs';
import { feedbackNeedsChanges } from './restart_feedback.mjs';
import { commandArgument, failureMessage, reportedExitCode as toolReportedExitCode, shellStep } from './tool_result.mjs';
import { fill } from './work_item_steps.mjs';
import { detect } from './crate/language.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { trim } from './write_str.mjs';
import { parseJson } from './crate/rust_str.mjs';

/**
 * Mirrors `fn plan_symbolic_command_reroute`.
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 * @param {object} symbolicAnswer
 */
export function planSymbolicCommandReroute(messages, toolNames, symbolicAnswer, objective = latestUserRequest(messages) ?? '') {
  return recipeStep(messages, toolNames, symbolicAnswer, objective, false);
}

/** Mirrors fn plan_owned_recipe_recovery: only the still-pending exact owned wrapper failure. */
export function planOwnedRecipeRecovery(messages, toolNames, symbolicAnswer, objective) {
  return recipeStep(messages, toolNames, symbolicAnswer, objective, true);
}
function recipeStep(messages, toolNames, symbolicAnswer, objective, recoveryOnly) {
  const attached = attachExecutionRecipe(symbolicAnswer ?? {}, symbolicAnswer?.synthesized_program).execution_recipe;
  if (!attached) return null;
  const recipe = requestedRecipe(latestUserRequest(messages) ?? '', attached);
  if (requestedCiWorkflow(objective)) attachCiWorkflow(recipe, objective);
  const writeTool = toolFor(toolNames, Capability.Write) ?? toolNames.find(name => isWorkspaceCreationTool(name));
  if (!writeTool) return null;
  const runTool = toolFor(toolNames, Capability.Run);
  if (!runTool) return null;
  const request = latestUserRequest(messages);
  const requirementsTarget = request === null ? null : targetOf(request);
  const delivery = request === null ? {
    kind: 'absent'
  } : recipeDeliveryReference(request, requirementsTarget?.reference ?? null);
  if (delivery.kind === 'refused') return finalAnswer(delivery.reason);
  const commit = requirementsTarget === null ? null : {
    ...requirementsTarget,
    reference: delivery.kind === 'bound' ? delivery.reference : requirementsTarget.reference
  };
  const resume = Boolean(commit?.reference.includes('/pull/'));
  const repositoryCi = resume && requestedCiWorkflow(objective);
  const workflow = repositoryCi ? recipeWorkflow(recipe, objective) : null;
  const expectedCommands = [...recipe.commands];
  if (commit && workflow === null) expectedCommands.push(recipeCommitCommand(recipe, commit));
  if (commit && workflow === null) expectedCommands.push(...prCompletionCommands(recipe, commit));
  const progress = recipeProgressAfterLatestUser(messages, writeTool, recipe, expectedCommands, resume);
  if (progress.source_invalidated) return recipeSourceRefusal();
  if (recoveryOnly && !progress.failure?.from_run) return null;
  if (commit) {
    const followup = prCompletionCommands(recipe, commit);
    const observed = Progress.scan(messages);
    if (followup.length) {
      const output = observed.latestSuccessfulRunOutputFor(followup[0]);
      if (output !== null && output !== undefined && feedbackNeedsChanges(output)) {
        return finalAnswer(fill('pr_feedback_review_report', [['{output}', output]]));
      }
    }
  }
  const nextFile = () => {
    if (progress.files_written === 0) return [recipe.path, recipe.source];
    const file = recipe.supporting_files[progress.files_written - 1];
    return file ? [file.path, file.source] : null;
  };
  if (progress.failure) {
    const failure = progress.failure;
    const step = failure.from_run ? expectedCommands[progress.commands_done] ?? writeTool : writeTool;
    const failedPath = nextFile()?.[0] ?? recipe.path;
    let repairNote = '';
    if (failure.from_run) {
      const observation = Progress.scan(messages);
      const bound = qualifiedCommandAttempt(observation, step);
      const files = [{
        path: recipe.path,
        source: recipe.source
      }, ...recipe.supporting_files];
      const owned = files.every(file => recipeFileMatches(observation, file));
      const observed = failureMessage(failure.reported, true, false) ?? failure.reported;
      const identity = bound?.binding === 'exact' && owned ? programRecoveryCommand(objective, recipe, step, observed) : null;
      if (recoveryOnly && identity === null) return null;
      if (identity !== null && failure.exit_code === 127 && commit && workflow !== null) {
        return planRecipeCiFallback(messages.slice(progress.failure_index + 1), toolNames, recipe, commit.reference, workflow, prCompletionCommands(recipe
    , commit), false, requirementsTarget.reference);
      }
      const recovery = identity === null ? planRecovery(messages, toolNames, step, failure.exit_code, failure.reported) : planRecoveryWithIdentity(messages
    , toolNames, step, failure.exit_code, failure.reported, identity);
      if (recovery) return recovery;
      const failed = failedStep(recipe.language, failure.reported, {
        exit_code: failure.exit_code,
        failed_command: step,
        artifact_path: failedPath,
        validate_program: validateProgram
      });
      const rung = Math.max(0, progress.repair_rung - 1);
      const outcome = repairStep(messages, toolNames, failed, rung, MAX_REPAIR_RUNGS);
      if (outcome.kind !== 'stop') return outcome.plan;
      repairNote = stopNote(messages, failed, outcome.reason, rung, MAX_REPAIR_RUNGS);
    }
    const report = failureReport(failure, messages, failedPath, step);
    return finalAnswer(repairNote === '' ? report : `${report}\n\n${repairNote}`);
  }
  const file = nextFile();
  if (file) {
    if (resume && !progress.source_checked.has(file[0])) {
      return planOne(runTool, jsonText({
        command: recipeSourceCommand(file[0])
      }));
    }
    if (resume && progress.source_preimages.get(file[0])?.absent === false) {
      const command = recipeSourceUpdateCommand(file[0], file[1], progress.source_preimages.get(file[0]));
      return command === null ? recipeSourceRefusal() : planOne(runTool, jsonText({
        command
      }));
    }
    return planOne(writeTool, writeArguments(file[0], file[1]));
  }
  const command = expectedCommands[progress.commands_done];
  if (command !== undefined) return planOne(runTool, jsonText({
    command
  }));
  if (commit && workflow !== null) {
    return planRecipeCiFallback(messages.slice(progress.last_command_index + 1), toolNames, recipe, commit.reference, workflow, prCompletionCommands(recipe
    , commit), true, requirementsTarget.reference);
  }
  return finalAnswer(executionRecipeFinalAnswer(recipe, progress.command_outputs, commit));
}
function recipeFileMatches(observation, file) {
  const index = observation.latestSourceMutationIndex(file.path);
  const attempt = index === null ? null : observation.attempts[index];
  if (attempt?.succeeded && attempt.arguments !== null) {
    if (attempt.capability === Capability.Write && writeMatches(attempt.arguments, file.path, file.source)) {
      const frame = qualifiedToolAttempt(observation, Capability.Write, JSON.parse(attempt.arguments));
      if (frame?.binding === 'exact' && frame.succeeded) return true;
    }
    if (attempt.capability === Capability.Edit && isWorkspaceCreationTool(attempt.tool ?? '') && file.source !== '' && file.source.endsWith('\n')
    && applyPatchInput(writeArguments(file.path, file.source)) === attempt.arguments) {
      const frame = qualifiedToolAttempt(observation, Capability.Edit, attempt.arguments);
      if (frame?.binding === 'exact' && frame.succeeded) return true;
    }
  }
  return recipeSourceMatches(observation, file.path, file.source);
}
function recipeWorkflow(recipe, objective) {
  const expected = {
    ...recipe,
    supporting_files: []
  };
  attachCiWorkflow(expected, objective);
  const workflow = expected.supporting_files[0];
  if (!workflow || !recipe.supporting_files.some(file => file.path === workflow.path && file.source === workflow.source)) return null;
  return /^name: ([^\r\n]+)$/mu.exec(workflow.source)?.[1] ?? null;
}
function prCompletionCommands(recipe, target) {
  if (!target.reference.includes('/pull/')) return [];
  const targetUrl = shellQuote(target.reference);
  const commands = recipe.commands.map(command => `- \`${command}\``).join('\n');
  const body = fill('pr_body_template', [['{path}', recipe.path], ['{commands}', commands], ['{reference}', target.reference]]);
  return [fill('pr_comments_command', [['{target}', targetUrl]]), fill('pr_edit_command', [['{target}', targetUrl], ['{body}', shellQuote(body)]]), fill('pr_ready_command', [['{target}', targetUrl]])];
}

/**
 * Mirrors `ExecutionRecipe::final_answer`: the harness-voice completion.
 * @param {object} recipe
 * @param {Array<string>} allOutputs
 * @param {object|null} committed a `CommitTarget` or null
 */
export function executionRecipeFinalAnswer(recipe, allOutputs, committed) {
  let outputs = allOutputs;
  let commitOutput;
  if (committed && allOutputs.length > recipe.commands.length) {
    outputs = allOutputs.slice(0, recipe.commands.length);
    commitOutput = allOutputs[recipe.commands.length];
  }
  let answer = agenticMessage('command_reroute_created_and_verified', {
    path: recipe.path,
    language: recipe.language,
    source: recipe.source
  });
  for (const file of recipe.supporting_files) answer += `\n\`${file.path}\`\n\n\`\`\`text\n${file.source}\n\`\`\`\n`;
  for (const command of recipe.commands) answer += `- \`${command}\`\n`;
  const nonEmpty = [...outputs].reverse().find(output => trim(output) !== '');
  const actual = nonEmpty === undefined ? agenticMessage('command_reroute_completed_without_output') : trim(nonEmpty);
  answer += agenticMessage('command_reroute_actual_output', {
    actual
  });
  if (committed && commitOutput !== undefined) {
    answer += agenticMessage('command_reroute_committed_and_pushed', {
      branch: pushRef(committed),
      reference: committed.reference,
      output: trim(commitOutput)
    });
  }
  return answer;
}

/** Mirrors `StepFailure::report`. */
function failureReport(failure, messages, path, step) {
  const language = detect(latestUserRequest(messages) ?? '');
  const intent = failure.exit_code !== null ? 'agentic_step_failed_with_exit_code' : 'agentic_step_failed';
  const code = failure.exit_code === null ? '' : String(failure.exit_code);
  return (localizedResponse(intent, language) ?? '').split('{step}').join(step).split('{path}').join(path).split('{code}').join(code).split('{report}').join(trim(failure.reported));
}

/** Mirrors `RecipeProgress::after_latest_user`. */
export function recipeProgressAfterLatestUser(messages, writeTool, recipe, expectedCommands, resume = targetOf(latestUserRequest(messages) ?? '') !== null) {
  const start = evidenceWindowStart(messages);
  const progress = {
    files_written: 0,
    commands_done: 0,
    command_outputs: [],
    failure: null,
    repair_rung: 0,
    source_checked: new Set(),
    source_preimages: new Map(),
    source_invalidated: false,
    failure_index: -1,
    last_command_index: -1
  };
  const files = [[recipe.path, recipe.source], ...recipe.supporting_files.map(file => [file.path, file.source])];
  const observedIds = new Set();
  for (let index = start; index < messages.length; index += 1) {
    const message = messages[index];
    if (message.role !== 'tool') continue;
    const callId = message.tool_call_id;
    if (callId === null || callId === undefined) continue;
    let call = null;
    for (let prior = index - 1; prior >= start && call === null; prior -= 1) {
      call = (messages[prior].tool_calls || []).find(candidate => candidate.id === callId) ?? null;
    }
    if (call === null || observedIds.has(callId)) continue;
    observedIds.add(callId);
    const resultTool = call.function.name;
    if (message.name !== null && message.name !== undefined && message.name.toLowerCase() !== resultTool.toLowerCase()) {
      continue;
    }
    const capability = classifyTool(resultTool);
    const pending = files[progress.files_written];
    const matchesWrite = pending !== undefined && (resultTool.toLowerCase() === writeTool.toLowerCase() && writeMatches(call.function.arguments
    , pending[0], pending[1]) || capability === Capability.Edit && isWorkspaceCreationTool(resultTool) && pending[1] !== '' && pending[1].endsWith('\n')
    && call.function.arguments === applyPatchInput(writeArguments(pending[0], pending[1])));
    let matchesRun = false;
    if (capability === Capability.Run && progress.files_written === files.length) {
      const command = runCommandOf(messages.slice(start, index), callId);
      const expected = expectedCommands[progress.commands_done];
      if (command !== null && expected !== undefined) {
        const marker = '\n# __formal_ai_prerequisite_retry\n';
        const at = command.indexOf(marker);
        matchesRun = command === expected || at >= 0 && command.slice(at + marker.length) === expected;
      }
    }
    const matchesProbe = resume && pending !== undefined && capability === Capability.Run && commandArgument(call.function.arguments) === recipeSourceCommand(pending[0]);
    const updateCommand = resume && pending !== undefined ? recipeSourceUpdateCommand(pending[0], pending[1], progress.source_preimages.get(pending[0])) : null;
    const matchesUpdate = capability === Capability.Run && updateCommand !== null && commandArgument(call.function.arguments) === updateCommand;
    if (!matchesWrite && !matchesRun && !matchesProbe && !matchesUpdate) continue;
    const output = plainText(message.content);
    if (matchesUpdate) {
      const frame = qualifiedCommandAttempt(Progress.scan(messages.slice(start, index + 1)), updateCommand);
      if (frame?.binding === 'exact' && frame.succeeded && reportedExitCode(output) === 0 && recipeSourceUpdateResult(output, pending[0])) {
        progress.source_checked.delete(pending[0]);
        progress.source_preimages.delete(pending[0]);
      } else {
        progress.failure = {
          reported: output,
          exit_code: reportedExitCode(output),
          from_run: false
        };
        progress.failure_index = index;
      }
      continue;
    }
    if (matchesProbe) {
      const observed = Progress.scan(messages.slice(start, index + 1));
      const frame = qualifiedCommandAttempt(observed, recipeSourceCommand(pending[0]));
      const result = frame?.binding === 'exact' && frame.succeeded && reportedExitCode(output) === 0 ? recipeSourceResult(output, pending[0]) : null;
      if (result === null) {
        progress.failure = {
          reported: output,
          exit_code: reportedExitCode(output),
          from_run: false
        };
      } else {
        progress.failure = null;
        progress.source_checked.add(pending[0]);
        progress.source_preimages.set(pending[0], result);
        if (!result.absent && result.source === pending[1]) progress.files_written += 1;
      }
      continue;
    }
    if (message.is_error) {
      if (matchesRun) progress.repair_rung = Math.min(255, progress.repair_rung + 1);
      progress.failure = {
        reported: output,
        exit_code: reportedExitCode(output),
        from_run: matchesRun
      };
      progress.failure_index = index;
      continue;
    }
    const failure = stepFailureFromResult(output, capability === Capability.Run);
    if (failure) {
      if (matchesRun) progress.repair_rung = Math.min(255, progress.repair_rung + 1);
      progress.failure = failure;
      progress.failure_index = index;
      continue;
    }
    progress.failure = null;
    if (matchesWrite) {
      progress.files_written += 1;
    } else if (capability === Capability.Run) {
      progress.commands_done += 1;
      progress.last_command_index = index;
      const step = shellStep(output);
      progress.command_outputs.push(step ? step.text : output);
    }
  }
  if (resume) {
    const observed = Progress.scan(messages);
    for (let index = 0; index < progress.files_written; index += 1) {
      const [path, content] = files[index];
      if (!recipeFileMatches(observed, {
        path,
        source: content
      })) {
        progress.files_written = index;
        progress.source_checked.delete(path);
        progress.source_invalidated = true;
        break;
      }
    }
  }
  return progress;
}
function asciiLower(text) {
  return text.replace(/[A-Z]/g, canonicalUnaryLower);
}

/** Mirrors `fn reported_exit_code` (command_reroute.rs). */
function reportedExitCode(output) {
  const code = toolReportedExitCode(output);
  return Number.isInteger(code) && code >= -2147483648 && code <= 2147483647 ? code : null;
}

/** Mirrors `StepFailure::from_result`. */
function stepFailureFromResult(reported, fromRun) {
  const exitCode = reportedExitCode(reported);
  if (exitCode !== null) return exitCode !== 0 ? {
    reported,
    exit_code: exitCode,
    from_run: fromRun
  } : null;
  // Nested JSON event fields are data, not the surrounding process's own prose.
  const prose = rustLines(reported).filter(line => parseJson(line) === undefined).join('\n');
  if (failureMessage(reported, false, false) !== null || failureMessage(prose, false, fromRun) !== null) {
    const code = toolReportedExitCode(reported);
    const fits = typeof code === 'number' && code >= -2147483648 && code <= 2147483647;
    return {
      exit_code: fits ? code : null,
      reported,
      from_run: fromRun
    };
  }
  return reportsFailureInProse(prose) ? {
    reported,
    exit_code: null,
    from_run: fromRun
  } : null;
}
const ABSENT_FIELD_VALUES = ['', '-', 'none', '(none)', 'empty', '(empty)', 'no output', '(no output)'];
function reportsFailureInProse(output) {
  const normalized = asciiLower(output);
  if (agenticMessage('command_reroute_failure_markers').split('|').some(marker => normalized.includes(marker))) {
    return true;
  }
  return rustLines(normalized).some(line => ['error:', 'failed:'].some(marker => {
    const at = line.indexOf(marker);
    return at >= 0 && !ABSENT_FIELD_VALUES.includes(trim(line.slice(at + marker.length)));
  }));
}
function runCommandOf(messages, callId) {
  for (const message of messages) {
    for (const call of message.tool_calls || []) {
      if (call.id === callId) return commandArgument(call.function.arguments) ?? null;
    }
  }
  return null;
}

/**
 * Mirrors `fn requested_recipe`: the recipe saved as the file the request
 * names (`… in add.py`), and — when the request runs the function with stated
 * arguments (`run it with 2 and 3`) — ending with the call that prints its
 * result (PR #1188 dogfooding). A recipe with supporting files already binds
 * its own path.
 */
export function requestedRecipe(request, recipe) {
  let next = recipe;
  const extension = pathExtension(recipe.path);
  const target = extension === null ? null : typedWriteTarget(request, extension);
  if (target && target !== recipe.path && !recipe.supporting_files.length) {
    next = {
      ...recipe,
      path: target,
      commands: recipe.commands.map(command => command.split(recipe.path).join(target))
    };
  }
  const call = functionCallCommand(next.language, next.path, next.source, statedCallArguments(request));
  if (call !== null && !next.commands.includes(call)) next = {
    ...next,
    commands: [...next.commands, call]
  };
  return next;
}

/**
 * Mirrors `fn stated_call_arguments`: the numbers and quoted literals after
 * the request's last seeded run verb, in order.
 */
export function statedCallArguments(request) {
  const verbs = terminalCommandVocabulary().run_verbs;
  const toks = tokens(request);
  let start = -1;
  toks.forEach((token, index) => {
    if (verbs.includes(normalizeCommandWord(token.text))) start = index;
  });
  if (start < 0) return [];
  const from = toks[start].end;
  const quoted = quotedSegmentSpans(request).filter(segment => segment.start >= from);
  const inside = token => quoted.some(segment => token.start >= segment.start && token.end <= segment.end);
  const found = [];
  for (const token of toks.slice(start + 1)) {
    if (inside(token)) continue;
    const value = token.text.replace(/[,.;:!?]+$/u, '');
    if (/^-?\d+(\.\d+)?$/u.test(value)) found.push([token.start, value]);
  }
  for (const segment of quoted) found.push([segment.start, JSON.stringify(segment.text)]);
  return found.sort((left, right) => left[0] - right[0]).map(([, value]) => value);
}
