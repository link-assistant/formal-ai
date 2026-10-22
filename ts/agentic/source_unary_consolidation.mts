import {quoteShellWord as quote} from './portable/string_contracts.mjs';
import { agenticMessage } from './messages.mjs';
// Finite installed AST operation; source observations never supply shell implementation.
import { Progress, qualifiedToolAttempt } from './progress.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { sourceUnaryConsolidationCommand } from './host.mjs';
import { readArguments } from './workspace_change.mjs';
import { planOne, jsonText, resolvedFinalAnswer, FinalDisposition } from './plan.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { observedPayload, incompleteReceipt, stepOutcome, StepOutcome } from './tool_result.mjs';

/** Every clause is owned by this one declared callback-value operation. */
export function sourceUnaryConsolidationRequest(task) {
  if (typeof task !== 'string' ||
         task.length > 4096) return null;
  const match = /^Consolidate\s+(?:the\s+)?stateless\s+unary\s+callbacks\s+in\s+`?(js\/agentic\/[A-Za-z0-9_./-]+\.mjs)`?\s+(?:using|with)\s+(?:the\s+)?(?:shared\s+)?canonical\s+helpers\.\s+Preserve\s+callback\s+value\s+behavior\s+and\s+comments\.?$/iu.exec(task.trim());
  if (!match ||
         match[1].split('/').some(x => x === '.' ||
         x === '..' ||
         !x)) return null;
  return Object.freeze({
    owner: match[1],
    operation: 'stateless-unary-callback-values',
    reflection: 'not-preserved',
    independentBehaviorValidation: 'required'
  });
}
const gap = reason => resolvedFinalAnswer(agenticMessage(reason, {}), FinalDisposition.Gap, 'source-unary-consolidation-gap');
export function planSourceUnaryConsolidationStep(task, messages, tools) {
  const request = sourceUnaryConsolidationRequest(task);
  if (!request) return null;
  const commandPrefix = sourceUnaryConsolidationCommand();
  if (!commandPrefix) return gap('source-unary-gap-1');
  const progress = Progress.scan(messages),
    read = progress.sourceReadFor(request.owner),
    readTool = toolFor(tools, Capability.Read);
  if (read === null) return readTool ? planOne(readTool, readArguments(request.owner)) : gap('source-unary-gap-2');
  if (read.status !== 'reported-success' ||
         read.complete !== true ||
         read.absent ||
         typeof read.source !== 'string') return gap('source-unary-gap-3');
  // The first exact current-request source observation fixes this transaction's command.
  const sourceAttempt = progress.attempts.find(attempt => {
    if (attempt.capability !== Capability.Read ||
         !attempt.source_read) return false;
    try {
      const a = JSON.parse(attempt.arguments);
      return a.path === request.owner &&
         a.filePath === request.owner &&
         a.file_path === request.owner;
    } catch {
      return false;
    }
  });
  if (!sourceAttempt ||
         sourceAttempt.source_read.status !== 'reported-success' ||
         sourceAttempt.source_read.complete !== true ||
         typeof sourceAttempt.source_read.source !== 'string') return gap('source-unary-gap-4');
  const command = agenticMessage('installed-source-operation-command', {
    node: quote(commandPrefix[0]),
    script: quote(commandPrefix[1]),
    task: quote(task),
    digest: sha256Hex(sourceAttempt.source_read.source)
  });
  const latestRun = [...progress.attempts].reverse().find(attempt => {
    try {
      return attempt.capability === Capability.Run &&
         JSON.parse(attempt.arguments).command === command;
    } catch {
      return false;
    }
  });
  if (latestRun) {
    let operationArguments;
    try {
      operationArguments = JSON.parse(latestRun.arguments);
    } catch {
      return gap('source-unary-gap-5');
    }
    const observed = qualifiedToolAttempt(progress, Capability.Run, operationArguments);
    if (!observed ||
         !observed.succeeded ||
         incompleteReceipt(observed.detail) ||
         stepOutcome(observed.detail) !== StepOutcome.Succeeded) return gap('source-unary-gap-6');
    let receipt;
    try {
      const payload = observedPayload(observed.detail);
      receipt = JSON.parse(payload);
      if (typeof payload !== 'string' ||
         payload.trim() !== JSON.stringify(receipt)) return gap('source-unary-gap-7');
    } catch {
      return gap('source-unary-gap-7');
    }
    if (receipt === null ||
         Array.isArray(receipt) ||
         receipt.schema !== 'stateless-unary-request/v1' ||
         receipt.request !== task ||
         receipt.owner !== request.owner ||
         receipt.beforeSha256 !== sha256Hex(sourceAttempt.source_read.source) ||
         receipt.autonomousAuthorshipCredit !== 0 ||
         receipt.completeSemanticEquivalence !== false ||
         receipt.independentBehaviorValidation !== 'pending' ||
         !Number.isSafeInteger(receipt.callbacks) ||
         receipt.callbacks < 0 ||
         receipt.callbacks > 100000 ||
         ![0, 1].includes(receipt.files) ||
         !/^([a-f0-9]{64})$/u.test(receipt.afterSha256 ?? '')) return gap('source-unary-gap-8');
    const afterRun = progress.attempts.slice(progress.attempts.indexOf(latestRun) + 1).some(attempt => {
      if (attempt.capability !== Capability.Read ||
         attempt.source_read?.source !== read.source ||
         attempt.source_read.status !== 'reported-success' ||
         attempt.source_read.complete !== true) return false;
      try {
        const a = JSON.parse(attempt.arguments);
        return a.path === request.owner &&
         a.filePath === request.owner &&
         a.file_path === request.owner;
      } catch {
        return false;
      }
    });
    if (!afterRun) return readTool ? planOne(readTool, readArguments(request.owner)) : gap('source-unary-gap-9');
    if (sha256Hex(read.source) !== receipt.afterSha256) return gap('source-unary-gap-10');
    return resolvedFinalAnswer(jsonText({
      sourceUnaryConsolidation: {
        owner: request.owner,
        callbacks: receipt.callbacks,
        sourceReadbackVerified: true,
        independentBehaviorValidation: 'pending',
        completeSemanticEquivalence: false,
        authorshipCredit: 0
      }
    }), FinalDisposition.Finding, 'source-unary-consolidation-observed');
  }
  if (!sourceAttempt) return gap('source-unary-gap-11');
  const tool = toolFor(tools, Capability.Run);
  return tool ? planOne(tool, jsonText({
    command
  })) : gap('source-unary-gap-12');
}
