import { finalAnswer, planOne, jsonText, writeArguments } from './plan.mjs';
import { agenticMessage } from './messages.mjs';
import { fill } from './work_item_steps.mjs';
import { splitWhitespace } from './write_str.mjs';
import { host, hasHost, readText } from './host.mjs';
import { parseRoot, findChildValue } from './crate/seed_parser.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { Progress, writeMatches, qualifiedToolAttempt } from './progress.mjs';
import { shellQuote } from './git_commit.mjs';
import { stepOutcome, StepOutcome, reportedExitCode, commandArgument } from './tool_result.mjs';
import { FinalPayloadRole } from './final_payload.mjs';
const COMMAND_NOT_FOUND_EXIT = 127;
const notFoundMarker = () => agenticMessage('prerequisite_recovery_not_found_marker');
const bootstrapPath = '.formal-ai/bootstrap/prerequisite-installer.py';
export function classifyFailure(command, exitCode, stderr) {
  const program = splitWhitespace(command)[0] ?? '';
  if (!program || exitCode === 0) return null;
  const missing = exitCode === COMMAND_NOT_FOUND_EXIT || stderr.includes(notFoundMarker());
  const denied = exitCode === 126 || stderr.includes('Permission denied');
  if (!missing && !denied) return null;
  return {
    program,
    observed: missing ? 'missing' : 'unusable'
  };
}
function publisherNodeFor(program) {
  return parseRoot(readText('data/seed/setup-publishers.lino')).children.find(node => findChildValue(node, 'program') === program);
}
function recoveryContext(command, program) {
  const grant = hasHost() && typeof host().prerequisiteGrant === 'function' ? host().prerequisiteGrant() : null;
  if (!grant || grant.scope !== 'workspace' || !Array.isArray(grant.programs) || !grant.programs.includes(program)) return null;
  if (grant.bootstrap?.program !== 'python3' || jsonText(grant.bootstrap.arguments) !== jsonText([bootstrapPath]) || grant.allowNetwork !== true) return null;
  const publisherNode = publisherNodeFor(program);
  if (!publisherNode) return null;
  const publisher = Object.fromEntries(['program', 'source_id', 'host', 'documentation'].map(key => [key, findChildValue(publisherNode, key)]));
  const probe = parseRoot(readText('data/seed/toolchains.lino')).children.find(node => findChildValue(node, 'program') === program);
  const dependencies = (findChildValue(probe, 'requires') || '').split(/\s+/u).map(value => value.replace(/^[()"]+|[()"]+$/gu, '')).filter(Boolean);
  const request = jsonText({
    program,
    publisher,
    dependencies,
    grant
  });
  const active = '.formal-ai/toolchains/active-' + program + '.bin';
  const recovery = 'python3 ' + shellQuote(bootstrapPath) + ' ' + shellQuote(request) + ' && read -r __formal_ai_prerequisite_bin < ' + shellQuote(active) + ' && export PATH="$__formal_ai_prerequisite_bin:$PATH" &&\n# __formal_ai_prerequisite_retry\n' + command;
  return {
    grant,
    recovery,
    source: readText('data/meta/prerequisite-installer.py')
  };
}
/** Mirrors fn grant_required_report (issue #1159): name the program and the pinned publisher procedure a grant would run. */
function grantRequiredReport(program, command, output) {
  const node = publisherNodeFor(program);
  const documentation = node ? findChildValue(node, 'documentation') || '' : '';
  const procedure = documentation === '' ? fill('prerequisite-grant-no-procedure', [['{program}', program]]) : fill('prerequisite-grant-procedure', [['{program}', program], ['{documentation}', documentation]]);
  return fill('prerequisite_grant_required_report', [['{program}', program], ['{procedure}', procedure], ['{command}', command], ['{output}', output]]);
}
/** Mirrors fn recovered_prerequisites_report (issue #1159): one sentence per prerequisite recovered in the current turn. */
export function recoveredPrerequisitesReport(messages) {
  const start = Math.max(0, messages.findLastIndex(message => String(message.role).toLowerCase() === 'user'));
  const reports = [];
  for (const attempt of Progress.scan(messages.slice(start)).attempts) {
    if (attempt.capability !== Capability.Run || !attempt.succeeded || attempt.arguments === null || attempt.arguments === undefined) continue;
    const command = commandArgument(attempt.arguments);
    const report = command === null ? null : recoveredReport(command);
    if (report !== null && !reports.includes(report)) reports.push(report);
  }
  return reports.length === 0 ? null : reports.join('\n');
}
function recoveredReport(command) {
  const marker = '\n# __formal_ai_prerequisite_retry\n';
  const at = command.indexOf(marker);
  if (at < 0) return null;
  const prefix = command.slice(0, at), original = command.slice(at + marker.length);
  if (!prefix.startsWith('python3 ' + shellQuote(bootstrapPath) + ' ')) return null;
  const active = prefix.split('.formal-ai/toolchains/active-')[1];
  if (active === undefined || !active.includes('.bin')) return null;
  const program = active.slice(0, active.indexOf('.bin'));
  const node = publisherNodeFor(program);
  if (!node) return null;
  return fill('prerequisite-recovered-report', [['{program}', program], ['{documentation}', findChildValue(node, 'documentation') || ''], ['{command}', original]]);
}
/** Mirrors fn with_recovery_report: lead a final answer with the recovery report; a source module payload stays byte-exact. */
export function withRecoveryReport(messages, plan) {
  if (plan?.kind !== 'final') return plan;
  if (plan.result?.text === plan.answer && plan.result?.payloadRole === FinalPayloadRole.SourceModule) return plan;
  const report = recoveredPrerequisitesReport(messages);
  if (report === null || String(plan.answer).includes(report)) return plan;
  const answer = report + '\n\n' + plan.answer;
  return plan.result?.text === plan.answer ? { ...plan, answer, result: { ...plan.result, text: answer } } : { ...plan, answer };
}
/** Exact authored transaction identity, separate from arbitrary marker-bearing text. */
export function recoveryOutput(messages, command) {
  const program = splitWhitespace(command)[0] ?? '';
  const context = recoveryContext(command, program);
  return context === null ? null : Progress.scan(messages).latestRunOutputFor(context.recovery);
}
export function planRecovery(messages, tools, command, exitCode, output) {
  return planRecoveryWithIdentity(messages, tools, command, exitCode, output, command);
}
/** Mirrors fn plan_recovery_with_identity; identity is supplied only by the request-owned recipe adapter. */
export function planRecoveryWithIdentity(messages, tools, command, exitCode, output, sourceCommand) {
  const need = classifyFailure(sourceCommand, exitCode, output);
  if (!need || need.observed !== 'missing') return null;
  const context = recoveryContext(command, need.program);
  if (context === null) return finalAnswer(grantRequiredReport(need.program, command, output));
  const progress = Progress.scan(messages),
    observed = progress.latestRunOutputFor(context.recovery);
  if (observed !== null) return stepOutcome(observed) === StepOutcome.Failed ? finalAnswer(observed) : null;
  const write = toolFor(tools, Capability.Write),
    run = toolFor(tools, Capability.Run);
  if (write === null || run === null) return finalAnswer(fill('prerequisite_grant_exceeded_report', []));
  const latestWrite = progress.latestSuccessfulWriteIndex(bootstrapPath);
  const written = latestWrite !== null && writeMatches(progress.attempts[latestWrite].arguments, bootstrapPath, context.source);
  return written ? planOne(run, jsonText({
    command: context.recovery
  })) : planOne(write, writeArguments(bootstrapPath, context.source));
}

/** Mirrors fn observed_recovery_command: exact current-turn observations only, not authority. */
export function observedRecoveryCommand(messages) {
  return observedPrerequisiteCommand(messages, false);
}

/** Observation-only denial guidance, never a recovery or installation grant. */
export function observedDeniedPrerequisiteCommand(messages) {
  const command = observedPrerequisiteCommand(messages, true);
  if (command === null) return null;
  const program = splitWhitespace(command)[0] ?? '';
  return recoveryContext(command, program) === null ? command : null;
}

function observedPrerequisiteCommand(messages, allowDenied) {
  const progress = Progress.scan(messages),
    attempts = progress.attempts;
  let latestRun = -1;
  for (let index = attempts.length - 1; index >= 0; index -= 1) {
    if (attempts[index].capability === Capability.Run) {
      latestRun = index;
      break;
    }
  }
  if (latestRun < 0) return null;
  const latest = attempts[latestRun];
  if (latest.arguments === null) return null;
  const latestCommand = commandArgument(latest.arguments);
  if (latestCommand === null) return null;
  for (let index = latestRun; index >= 0; index -= 1) {
    const attempt = attempts[index];
    if (attempt.capability !== Capability.Run || attempt.arguments === null) continue;
    const command = commandArgument(attempt.arguments);
    if (command === null) continue;
    const receipt = progress.latestRunOutputFor(command);
    if (receipt === null || stepOutcome(receipt) !== StepOutcome.Failed) continue;
    const bound = qualifiedToolAttempt(progress, Capability.Run, JSON.parse(attempt.arguments));
    if (bound?.binding !== 'exact') continue;
    const need = classifyFailure(command, reportedExitCode(receipt), receipt);
    if (need?.observed !== 'missing') continue;
    const context = recoveryContext(command, need.program);
    if (context === null) {
      // The exact latest failure can produce grant guidance, never a tool call.
      if (allowDenied && latestCommand === command && latestRun + 1 === attempts.length && publisherNodeFor(need.program)) return command;
      continue;
    }
    if (latestCommand !== command && latestCommand !== context.recovery) continue;
    const latestBound = qualifiedToolAttempt(progress, Capability.Run, JSON.parse(latest.arguments));
    if (latestBound?.binding !== 'exact') return null;
    for (const following of attempts.slice(latestRun + 1)) {
      if (following.capability !== Capability.Write || !following.succeeded || following.arguments === null || !writeMatches(following.arguments, bootstrapPath, context.source)) return null;
      const written = qualifiedToolAttempt(progress, Capability.Write, JSON.parse(following.arguments));
      if (written?.binding !== 'exact' || !written.succeeded) return null;
    }
    return command;
  }
  return null;
}
