import { AsyncLocalStorage } from "node:async_hooks";
import * as trackedConsumer from './tracked-source-transaction.mjs';
import * as trackedEvidenceConsumer from './tracked-source-evidence-consumer.mjs';
import * as trackedBackend from './tracked-source-transaction-backend.mjs';
import * as validatedDrive from '../../experiments/js_dogfood/drive.mjs';
import { deriveReferenceReadRequest } from './source-reference-request.mjs';
import { declaredVerificationPermission } from '../../experiments/js_dogfood/drive.mjs';
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
const registeredEffectVerifiers = new WeakMap();
/** Public operation callbacks cannot install or resolve a trusted effect observation. */
export function registeredTrackedSourceEffectObservation(token, request, workspace) {
  const verifier = registeredEffectVerifiers.get(installedHost().sourceOperation);
  if (!verifier) throw Error('MissingRegisteredTrackedEffectHost');
  return verifier(token, request, workspace);
}
const registeredReadVerifiers = new WeakMap();
/** Only the installed real factory can resolve an opaque physical Read token. */
export function registeredPhysicalReadObservation(token, request, workspace, declaration) {
  const verifier = registeredReadVerifiers.get(installedHost().sourceOperation);
  if (!verifier) throw Error('MissingRegisteredPhysicalReadHost');
  return verifier(token, request, workspace, declaration);
}

const sourceSessionPolicies = new WeakMap();
/** Unknown APIs and public context copies cannot issue an inferred-command decision. */
export function queryOwnedInferredCommandPolicy(api, request, command, tools) {
  const query = sourceSessionPolicies.get(api);
  return query ? query(request, command, tools) : null;
}
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
const referencePlanQueries = new WeakMap();
/** The real active factory supplies only its complete request's initial Read prerequisite. */
export function registeredSourceReferenceReadPlan(request, workspace, tools) {
  const query = referencePlanQueries.get(installedHost().sourceOperation);
  return query ? query(request, workspace, tools) : null;
}
function referencePlanQuery(current) {
  return (request, workspace, tools) => {
    let context;
    try { context = current(); } catch { return null; }
    if (!context.referenceRead || context.request !== request
        || fs.realpathSync(workspace) !== context.workspace
        || JSON.stringify(tools) !== JSON.stringify(context.tools)) return null;
    context.referenceRead.validate();
    const operands = context.referenceRead.operands;
    const tool = context.tools.find(name => classifyTool(name) === Capability.Read);
    if (!tool || operands.length !== 1) return null;
    const plan = planOne(tool, readArguments(operands[0]));
    if (readPolicyBlocksPlan(context.request, plan)) return null;
    return Object.freeze({ kind: 'OnlySourceReferenceReadPrerequisite', originalRequest: request,
      plan, pendingNeeds: context.referenceRead.contract.unresolvedNeeds,
      mutationAuthority: false, completed: false });
  };
}

const referenceConsumerServices = new WeakMap();
/** Lookup alone cannot install a service or turn a public callback into authority. */
export async function consumeRegisteredSourceReference(request, workspace, tools) {
  const consume = referenceConsumerServices.get(installedHost().sourceOperation);
  return consume ? consume(request, workspace, tools) : null;
}
const trackedConsumerSources = Object.freeze([
  ['./tracked-source-transaction.mjs', '97c77d9a128ea4a29e48a9f123f0f921396f0a0edba318ff7a98c9368847f031'],
  ['./tracked-source-evidence-proposal.mjs', 'e3263f575db509875590bf4d56cf4bc1705124c28ffd55cc972f3264f899a265'],
  ['./tracked-source-evidence-consumer.mjs', '9212d206a280606867bb528e2045d5f084a776b040441eebf078399c9f560ab7'],
  ['../../experiments/js_dogfood/exclusive-create.mjs', '5f97c0d40374464637e3a813c3fa9690b729a79a0aa0ec9d98cf8cc9d9b03a15'],
  ['../../experiments/js_dogfood/exclusive-create.py', '2292a8db8e1d2fc182242c20d6ade68c94acea3dc8f85113a2227f40a8e5d048'],
  ['./rust-source-transform.mjs', '77dfad4fbfd4292a43ff5a4874630d2b100e62f6687ea278b746c8bba8b2ba81'],
  ['../agentic/crate/rust_constant_rename.mjs', 'f7aee9d6c2148bd21800659f289b809bec79a7de703c86faed21e983047f84d6']
]);
function trackedConsumerSourceBinding() {
  const inputs = trackedConsumerSources.map(([relative, expected]) => {
    const file = new URL(relative, import.meta.url);
    const physical = fs.realpathSync(file);
    if (physical !== file.pathname) throw Error('AliasedTrackedConsumerSource');
    const stat = fs.lstatSync(physical, {bigint: true});
    if (!stat.isFile() || hash(fs.readFileSync(physical)) !== expected) {
      throw Error('ChangedTrackedConsumerSource');
    }
    return {physical, expected, identity: [stat.dev, stat.ino, stat.ctimeNs, stat.size]};
  });
  return () => {
    for (const input of inputs) {
      const stat = fs.lstatSync(input.physical, {bigint: true});
      if (!stat.isFile() || fs.realpathSync(input.physical) !== input.physical
          || [stat.dev, stat.ino, stat.ctimeNs, stat.size].some((value, index) => value !== input.identity[index])
          || hash(fs.readFileSync(input.physical)) !== input.expected) {
        throw Error('ChangedTrackedConsumerSource');
      }
    }
  };
}
function referenceConsumerService(operation, current) {
  return async (request, workspace, tools) => {
    const context = current();
    if (!context.referenceRead || context.request !== request
        || fs.realpathSync(workspace) !== context.workspace
        || JSON.stringify(context.tools) !== JSON.stringify(tools)) return null;
    context.referenceRead.validate();
    if (validatedDrive.declaredTrackedWritePermission(request, workspace, tools) !== true) {
      throw Error('MissingValidatedTrackedWriteContext');
    }
    const read = context.latestReferenceRead;
    if (!read || context.referenceConsumptionStarted) throw Error('MissingFreshReferenceRead');
    const source = operation.physicalReadObservation(read.token, request, workspace, read.declaration);
    if (source.referenceRequest !== context.referenceRead.contract
        || source.referenceRequest.wholeRequestConsumed !== true) throw Error('DifferentTrackedRequest');
    const validateSources = trackedConsumerSourceBinding();
    const validate = () => {
      if (current() !== context || context.latestReferenceRead !== read) throw Error('ReplacedTrackedReferenceContext');
      context.referenceRead.validate();
      validateSources();
      if (validatedDrive.declaredTrackedWritePermission(request, workspace, tools) !== true) {
        throw Error('MissingValidatedTrackedWriteContext');
      }
    };
    validate();
    context.referenceConsumptionStarted = true;
    const prepared = await trackedConsumer.prepareTrackedRustSourceTransaction(
      read.token, request, workspace, read.declaration);
    validate();
    operation.physicalReadObservation(read.token, request, workspace, read.declaration);
    const disposition = await trackedConsumer.executePreparedTrackedRustSource(prepared);
    validate();
    const effectToken = await trackedConsumer.issuePreparedTrackedSourceEffectReceipt(prepared);
    validate();
    operation.trackedSourceEffectObservation(effectToken, request, workspace);
    const evidenceArtifacts = await trackedEvidenceConsumer.executeTrackedSourceEvidence(
      effectToken, request, workspace);
    validate();
    operation.trackedSourceEffectObservation(effectToken, request, workspace);
    return Object.freeze({...disposition, evidenceArtifacts});
  };
}

const trackedContextIssuers = new WeakMap();
const trackedContextObservers = new WeakMap();
export function issueTrackedSourceContext(token, request, workspace, declaration) {
  const issuer = trackedContextIssuers.get(installedHost().sourceOperation);
  if (!issuer) throw Error('MissingTrackedSourceContextIssuer');
  return issuer(token, request, workspace, declaration);
}
export function trackedSourceContextObservation(token, command) {
  const observer = trackedContextObservers.get(installedHost().sourceOperation);
  if (!observer) throw Error('MissingTrackedSourceContextObserver');
  return observer(token, command);
}
function createTrackedScopeBindings(operation, current) {
  const issued = new WeakMap();
  const writePermitted = context => {
    const query = validatedDrive.declaredTrackedWritePermission;
    if (typeof query !== 'function' || query(context.request, context.workspace, context.tools) !== true) {
      throw Error('MissingValidatedTrackedWriteContext');
    }
    context.referenceRead?.validate();
  };
  return {
    issue(token, request, workspace, declaration) {
      const source = operation.physicalReadObservation(token, request, workspace, declaration);
      const context = current();
      if (!context.referenceRead || source.referenceRequest !== context.referenceRead.contract) throw Error('MissingTrackedRequest');
      writePermitted(context);
      const capability = Object.freeze({});
      issued.set(capability, { context, source, attempt: context.readAttempts.get(source.operand) });
      return capability;
    },
    observe(token, command) {
      const proof = issued.get(token), context = current();
      if (!proof || proof.context !== context) throw Error('UnissuedTrackedSourceContext');
      if (context.readAttempts.get(proof.source.operand) !== proof.attempt) throw Error('StaleTrackedReadAttempt');
      writePermitted(context);
      return Object.freeze({ request: context.request, workspace: context.workspace,
        path: proof.source.path, contract: proof.source.referenceRequest,
        commandPolicy: typeof command === 'string'
          ? declaredVerificationPermission(context.request, context.workspace, context.tools, command) : null });
    },
  };
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
    context.referenceRead?.validate();
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
    const sameRequest = context.referenceRead
      ? typeof user?.content === 'string' && user.content === context.request
      : user && userRequestText(user.content) === context.request;
    if (!sameRequest) throw Error('DifferentPhysicalReadRequest');
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
    if (context.referenceRead) context.latestReferenceRead = Object.freeze({token, declaration});
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
        referenceRequest: context.referenceRead?.contract ?? null,
        declaration: proof.declaration, path: currentBytes.path, operand: currentBytes.operand,
        content: currentBytes.content, bytesBase64: currentBytes.bytesBase64,
        bytes: currentBytes.bytes, sha256: currentBytes.sha256, sourceComplete: true,
        pathNamespaceAuthority: 'Unknown', readRaceProof: false,
        syntax: 'Unknown', moduleInitializationEffects: 'Unknown', callEffects: 'Unknown',
        acceptedRun: 'Unknown', publicationAuthority: 'Unknown'});
    },

    prepareTrackedSourceTransaction(...values) { current(); return trackedBackend.prepareTrackedSourceTransaction(...values); },
    persistTrackedSourceInverse(...values) { current(); return trackedBackend.persistTrackedSourceInverse(...values); },
    writeTrackedSourceCandidate(...values) { current(); return trackedBackend.writeTrackedSourceCandidate(...values); },
    readTrackedSourceCandidate(...values) { current(); return trackedBackend.readTrackedSourceCandidate(...values); },
    trackedSourceTransactionDisposition(...values) { current(); return trackedBackend.trackedSourceTransactionDisposition(...values); },
    abortTrackedSourceTransaction(...values) { current(); return trackedBackend.abortTrackedSourceTransaction(...values); },
    recoverTrackedSourceTransaction(...values) { current(); return trackedBackend.recoverTrackedSourceTransaction(...values); },
    issueTrackedSourceEffectReceipt(...values) {
      current();
      const validate = trackedConsumerSourceBinding();
      validate();
      const token = trackedBackend.issueTrackedSourceEffectReceipt(...values);
      current(); validate();
      return token;
    },
    trackedSourceEffectObservation(...values) {
      current();
      const validate = trackedConsumerSourceBinding();
      validate();
      const observed = trackedBackend.trackedSourceEffectObservation(...values);
      current(); validate();
      return observed;
    },

    async createTrackedSourceEvidenceArtifact(...values) {
      const context = current(), validate = trackedConsumerSourceBinding();
      const check = () => {
        if (current() !== context || values[1] !== context.request || values[2] !== context.workspace
            || validatedDrive.declaredTrackedWritePermission(context.request, context.workspace, context.tools) !== true) {
          throw Error('MissingValidatedTrackedArtifactWriteContext');
        }
        validate();
      };
      check();
      const receipt = await trackedBackend.createTrackedSourceEvidenceArtifact(...values);
      check();
      return receipt;
    },
    trackedSourceEvidenceArtifactObservation(...values) {
      current();
      const validate = trackedConsumerSourceBinding(); validate();
      const observed = trackedBackend.trackedSourceEvidenceArtifactObservation(...values);
      current(); validate(); return observed;
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
      if (frame === null) {
        const referenceRead = deriveReferenceReadRequest(context.request, requestSeed, import.meta.url);
        if (referenceRead !== null) {
          if (!Array.isArray(context.tools) || typeof executeSession !== 'function') {
            throw Error('UnknownReferenceReadPlatform');
          }
          if (!context.tools.some(tool => classifyTool(tool) === Capability.Read)) {
            throw Error('MissingReferenceReadCapability');
          }
          const owned = { referenceRead, provider: null,
            frame: {request: {inputs: referenceRead.operands, acceptance: []}},
            workspace: fs.realpathSync(context.workspace), request: context.request,
            tools: Object.freeze([...context.tools]), active: true, readAttempts: new Map() };
          return storage.run(owned, async () => {
            try { return await executeSession(); } finally { owned.active = false; }
          });
        }
        let workspace = null;
        try { workspace = fs.realpathSync(context.workspace); } catch { /* Unknown workspace is passthrough only. */ }
        const owned = { passthrough: true, request: context.request, workspace,
          tools: Array.isArray(context.tools) ? Object.freeze([...context.tools]) : null, active: true };
        return storage.run(owned, async () => {
          try { return await executeSession(); } finally { owned.active = false; }
        });
      }
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
      if (context.referenceRead && capability !== Capability.Read) {
        throw Error('MissingReferenceEffectAuthority');
      }
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
  sourceSessionPolicies.set(api, (request, command, tools) => {
    const context = storage.getStore();
    if (!context?.active || context.request !== request
        || installedHost().sourceSession !== api || installedHost().sourceOperation !== operation
        || JSON.stringify(context.tools) !== JSON.stringify(tools)) return null;
    return declaredVerificationPermission(request, context.workspace, tools, command);
  });
  registeredReadVerifiers.set(operation, (token, request, workspace, declaration) =>
    operation.physicalReadObservation(token, request, workspace, declaration));
  registeredEffectVerifiers.set(operation, (token, request, workspace) =>
    operation.trackedSourceEffectObservation(token, request, workspace));
  referencePlanQueries.set(operation, referencePlanQuery(current));
  referenceConsumerServices.set(operation, referenceConsumerService(operation, current));
  const trackedBindings = createTrackedScopeBindings(operation, current);
  trackedContextIssuers.set(operation, trackedBindings.issue);
  trackedContextObservers.set(operation, trackedBindings.observe);
  return Object.freeze(api);
}
