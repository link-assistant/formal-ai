import { AsyncLocalStorage } from "node:async_hooks";
import { host as installedHost } from "../agentic/host.mjs";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { createAcceptedNodeSourceProvider } from "./accepted-node-source-provider.mjs";
import { deriveCompleteSourceRequest } from "../agentic/module_function/complete-source-preflight.mjs";
import { prepareSourceCandidate, authorizeSourceCandidateWrite, recordPreparedSourceWrite, finishSourceCandidate, abortSourceCandidate } from "../agentic/module_function/source-candidate-consumer.mjs";
import { Capability } from "../agentic/capability.mjs";
import { classifyTool } from "../agentic/capability_router.mjs";
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
/** Request-scoped trusted platform boundary; wire messages cannot install it. */
export function createNodeSourceSessionHost(readText) {
  const requestSeed = readText('data/seed/source-authoring-grammar.lino');
  const storage = new AsyncLocalStorage();
  const current = () => {
    const context = storage.getStore();
    if (!context || context.passthrough || installedHost().sourceOperation !== operation
        || installedHost().sourceSession !== api) throw Error('MissingSourceOperationContext');
    return context;
  };
  const operation = Object.freeze({
    hasContext: () => Boolean(storage.getStore()?.provider),
    requestSeed: () => requestSeed,
    compositionSeed: () => requestSeed,
    acceptedSources(receipt, request, command, workspace) {
      const context = current();
      if (request !== context.request || workspace !== context.workspace) throw Error('DifferentSourceOperationContext');
      return context.provider.validate(receipt, request, command);
    },
    operationDescriptor(receipt, request, command, workspace) {
      operation.acceptedSources(receipt, request, command, workspace);
      const context = current();
      return {
        command,
        acceptanceOperand: context.frame.request.acceptance[0],
        acceptancePath: context.frame.request.acceptance[0],
        acceptanceIdentity: receipt.acceptanceIdentity
      };
    },
    moduleURL(specifier, parent) {
      current();
      const url = specifier.startsWith('/') ? pathToFileURL(specifier) : new URL(specifier, pathToFileURL(parent));
      if (url.protocol !== 'file:' || url.search || url.hash) throw Error('UnsupportedSourceModuleURL');
      return url.href;
    },
    candidateStatus(...operands) {
      return current().provider.candidateStatus(...operands);
    },
    finalizedCandidateStatus(...operands) {
      const context = current();
      const status = context.provider.candidateStatus(...operands);
      if (context.completed?.verified !== true || context.completed.physicalDisposition.state !== 'committed') return null;
      if (context.io.processDisposition(operands[0]) !== 'succeeded') return null;
      return status;
    }
  });
  const api = {
    sourceOperation: operation,
    completedDisposition() {
      const context = current();
      if (context.completed?.verified !== true
          || context.completed.physicalDisposition.state !== 'committed') return null;
      return Object.freeze({ verified: true, physicalDisposition: 'committed' });
    },
    active: () => storage.getStore() !== undefined,
    async run(context, executeSession) {
      const frame = deriveCompleteSourceRequest(context.request, requestSeed);
      if (frame === null) return storage.run({
        passthrough: true
      }, executeSession);
      if (!Array.isArray(context.tools) || typeof executeSession !== 'function') throw Error('UnknownSourceExecutionPlatform');
      for (const capability of [Capability.Read, Capability.Write, Capability.Run]) {
        if (!context.tools.some(tool => classifyTool(tool) === capability)) throw Error('MissingSourceExecutionCapability');
      }
      const workspace = fs.realpathSync(context.workspace);
      const sources = frame.request.inputs.map(operand => {
        const file = path.resolve(workspace, operand);
        if (!file.startsWith(workspace + path.sep) || fs.realpathSync(file) !== file) throw Error('UnownedSourceOperand');
        return {
          path: operand,
          sha256: hash(fs.readFileSync(file))
        };
      });
      const provider = createAcceptedNodeSourceProvider({
        request: context.request,
        workspace,
        acceptanceOperand: frame.request.acceptance[0],
        acceptedCommands: [frame.request.command],
        sources,
        requestSeed
      });
      const owned = {
        frame,
        provider,
        workspace,
        request: context.request,
        tools: Object.freeze([...context.tools]),
        before: null,
        prepared: null,
        io: null,
        completed: null
      };
      return storage.run(owned, async () => {
        try {
          return await executeSession();
        } finally {
          if (owned.prepared !== null && owned.completed === null) owned.completed = abortSourceCandidate(owned.prepared);
          owned.io?.release();
        }
      });
    },
    executeResult(call, messages, fallback) {
      const context = storage.getStore();
      if (!context || context.passthrough) return fallback();
      const argumentsValue = JSON.parse(call.arguments);
      const capability = classifyTool(call.tool);
      if (capability === Capability.Write) {
        if (context.before === null || context.prepared !== null) throw Error('UnownedSourceWriteOperation');
        context.io = context.provider.createCandidateIO();
        const prepared = prepareSourceCandidate(operation, context.io, context.before, context.request, messages, context.workspace, context.tools);
        if (prepared.plan.calls.length !== 1 || prepared.plan.calls[0].tool !== call.tool || prepared.plan.calls[0].arguments !== call.arguments) throw Error('DifferentPreparedSourceWrite');
        authorizeSourceCandidateWrite(prepared);
        context.prepared = prepared;
        context.io.writeCandidate(argumentsValue.content);
        recordPreparedSourceWrite(prepared);
        return {
          content: ''
        };
      }
      if (capability !== Capability.Run || argumentsValue.command !== context.frame.request.command) return fallback();
      let receipt;
      try {
        receipt = context.provider.observe(argumentsValue.command);
      } catch (error) {
        if (context.prepared !== null) context.completed = abortSourceCandidate(context.prepared);
        return {
          content: JSON.stringify({
            is_error: true,
            error: error.message
          }),
          is_error: true,
          source_session_disposition: context.completed
        };
      }
      if (context.prepared === null) context.before = receipt;else context.completed = finishSourceCandidate(operation, context.prepared, receipt, receipt);
      return {
        content: 'Output: ' + receipt.stdout + '\n' + (receipt.stderr ? 'Error: ' + receipt.stderr + '\n' : '') + 'Exit Code: ' + receipt.process.status,
        accepted_operation_sources: receipt,
        source_session_disposition: context.completed
      };
    }
  };
  return Object.freeze(api);
}
