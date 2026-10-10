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
import { userRequestText } from "../agentic/content.mjs";
import { readPolicyBlocksPlan } from "../agentic/file_read/ownership.mjs";
import { planOne } from "../agentic/plan.mjs";
import { readArguments } from "../agentic/workspace_change.mjs";
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function declaredPhysicalReadOperand(call, argumentsValue) {
    if (argumentsValue === null || typeof argumentsValue !== 'object' || Array.isArray(argumentsValue)
        || !Object.hasOwn(argumentsValue, 'file_path')) throw Error('UnknownPhysicalReadArguments');
    const operand = argumentsValue.file_path;
    if (typeof operand !== 'string') throw Error('UnknownPhysicalReadArguments');
    const keys = Object.keys(argumentsValue).sort().join(',');
    const sole = keys === 'file_path' && call.arguments === JSON.stringify({file_path: operand});
    const canonical = keys === 'filePath,file_path,path' && argumentsValue.filePath === operand
      && argumentsValue.path === operand && call.arguments === readArguments(operand);
    if (!sole && !canonical) throw Error('UnknownPhysicalReadArguments');
    return operand;
}
/** Request-scoped trusted platform boundary; wire messages cannot install it. */
export function createNodeSourceSessionHost(readText) {
  const requestSeed = readText('data/seed/source-authoring-grammar.lino');
  const storage = new AsyncLocalStorage();
  const current = () => {
    const context = storage.getStore();
    if (!context || context.passthrough || !context.active || installedHost().sourceOperation !== operation
        || installedHost().sourceSession !== api) throw Error('MissingSourceOperationContext');
    return context;
  };

  const targetReads = new WeakMap();
  const hostFile = fs.realpathSync(new URL(import.meta.url));
  const hostIdentity = hash(fs.readFileSync(hostFile));
  function physicalTarget(context, operand) {
    if (!context.frame.request.inputs.includes(operand)
        && !context.frame.request.acceptance.includes(operand)) throw Error('UnownedPhysicalReadOperand');
    if (path.isAbsolute(operand) || operand.split(/[\\/]/u).includes('..')) throw Error('UnownedPhysicalReadPath');
    const file = path.resolve(context.workspace, operand);
    if (!file.startsWith(context.workspace + path.sep)) throw Error('OutsidePhysicalReadWorkspace');
    if (hash(fs.readFileSync(hostFile)) !== hostIdentity) throw Error('PhysicalReadIssuerSourceDrift');
    let cursor = path.parse(file).root;
    const identities = [];
    for (const part of file.slice(cursor.length).split(path.sep)) {
      cursor = path.join(cursor, part);
      const fileStatus = fs.lstatSync(cursor, {bigint: true});
      if (fileStatus.isSymbolicLink()) throw Error('PhysicalReadSymlink');
      if (cursor === file ? !fileStatus.isFile() : !fileStatus.isDirectory()) throw Error('PhysicalReadNotRegular');
      const identity = [cursor, fileStatus.dev, fileStatus.ino, fileStatus.birthtimeNs];
      if (cursor === file) identity.push(fileStatus.ctimeNs);
      identities.push(identity.join(':'));
    }
    const before = fs.statSync(file, {bigint: true});
    if (before.size > 1048576n) throw Error('PhysicalReadSizeBound');
    const bytes = fs.readFileSync(file);
    const after = fs.statSync(file, {bigint: true});
    if (before.ino !== after.ino || before.ctimeNs !== after.ctimeNs
        || before.size !== BigInt(bytes.length)) throw Error('PhysicalReadChangedDuringCapture');
    const content = new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(bytes);
    return {path: file, operand, content, bytesBase64: bytes.toString('base64'),
      bytes: bytes.length, sha256: hash(bytes), identities};
  }
  function declaredRead(call, messages) {
    const declaration = [...messages].reverse().find(message => message.role === 'assistant' && message.tool_calls);
    const matches = declaration?.tool_calls.filter(value => value.function?.name === call.tool
      && value.function.arguments === call.arguments) ?? [];
    if (matches.length !== 1 || typeof matches[0].id !== 'string' || !matches[0].id) throw Error('UnboundPhysicalReadCall');
    return Object.freeze({id: matches[0].id, name: call.tool, arguments: call.arguments});
  }
  function permittedRead(context, call, messages, argumentsValue) {
    if (!context.tools.includes(call.tool)) throw Error('UnregisteredPhysicalReadTool');
    const operand = declaredPhysicalReadOperand(call, argumentsValue);
    if (readPolicyBlocksPlan(context.request, planOne(call.tool, readArguments(operand)))) throw Error('PhysicalReadPolicyRefused');
    const user = [...messages].reverse().find(message => message.role === 'user');
    if (!user || userRequestText(user.content) !== context.request) throw Error('DifferentPhysicalReadRequest');
    const declaration = declaredRead(call, messages);
    return {operand, declaration};
  }
  function observePhysicalRead(context, call, messages, argumentsValue) {
    const {operand, declaration} = permittedRead(context, call, messages, argumentsValue);
    const attempt = Object.freeze({});
    context.readAttempts.set(operand, attempt);
    const snapshot = physicalTarget(context, operand);
    const token = Object.freeze({});
    targetReads.set(token, {context, attempt, declaration, snapshot});
    return {content: snapshot.content, owned_source_read: token,
      source_read: {path: operand, success: true, complete: true, format: 'raw'}};
  }

  const operation = Object.freeze({
    physicalReadObservation(token, request, workspace, declaration) {
      const context = current();
      const proof = targetReads.get(token);
      if (!proof || proof.context !== context || request !== context.request
          || workspace !== context.workspace) throw Error('UnissuedPhysicalReadObservation');
      if (!declaration || Object.keys(declaration).sort().join(',') !== 'arguments,id,name'
          || Object.keys(proof.declaration).some(key => declaration[key] !== proof.declaration[key])) throw Error('DifferentPhysicalReadCall');
      if (context.readAttempts.get(proof.snapshot.operand) !== proof.attempt) throw Error('StalePhysicalReadAttempt');
      const currentBytes = physicalTarget(context, proof.snapshot.operand);
      if (currentBytes.sha256 !== proof.snapshot.sha256
          || JSON.stringify(currentBytes.identities) !== JSON.stringify(proof.snapshot.identities)) throw Error('PhysicalReadSourceDrift');
      return Object.freeze({kind: 'OnlyPhysicalRead', requestIdentity: hash(request), workspace,
        declaration: proof.declaration, path: currentBytes.path, operand: currentBytes.operand,
        content: currentBytes.content, bytesBase64: currentBytes.bytesBase64,
        bytes: currentBytes.bytes, sha256: currentBytes.sha256, sourceComplete: true,
        pathNamespaceAuthority: 'Unknown', readRaceProof: false,
        syntax: 'Unknown', moduleInitializationEffects: 'Unknown', callEffects: 'Unknown',
        acceptedRun: 'Unknown', publicationAuthority: 'Unknown'});
    },

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
        completed: null,
        active: true,
        readAttempts: new Map()
      };
      return storage.run(owned, async () => {
        try {
          return await executeSession();
        } finally {
          try {
            if (owned.prepared !== null && owned.completed === null) owned.completed = abortSourceCandidate(owned.prepared);
            owned.io?.release();
          } finally { owned.active = false; }
        }
      });
    },
    executeResult(call, messages, fallback) {
      const context = storage.getStore();
      if (!context || context.passthrough) return fallback();
      const argumentsValue = JSON.parse(call.arguments);
      const capability = classifyTool(call.tool);
      if (capability === Capability.Read) {
        try {
          const ownedContext = current();
          if (argumentsValue !== null && typeof argumentsValue === 'object'
              && argumentsValue.file_path === ownedContext.frame.request.destination) {
            permittedRead(ownedContext, call, messages, argumentsValue);
            const result = fallback();
            if (result && Object.hasOwn(result, 'owned_source_read')) {
              throw Error('UnexpectedDestinationReadAuthority');
            }
            return result;
          }
          return observePhysicalRead(ownedContext, call, messages, argumentsValue);
        }
        catch (error) {
          if (argumentsValue !== null && typeof argumentsValue === 'object'
              && typeof argumentsValue.file_path === 'string') context.readAttempts.set(argumentsValue.file_path, Object.freeze({}));
          return {content: error.message, is_error: true};
        }
      }
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
