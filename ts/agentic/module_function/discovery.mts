import { contractText } from '../workspace_discovery.mjs';
// Request-local source-callable observations; synthesis remains uncertified.
import { Capability } from '../capability.mjs';
import { Progress } from '../progress.mjs';
import { toolFor } from '../capability_router.mjs';
import { FinalDisposition, resolvedFinalAnswer } from '../final_result.mjs';
import { agenticMessage } from '../messages.mjs';
import { planOne } from '../plan.mjs';
import { readArguments } from '../workspace_change.mjs';
import { compactJson } from '../crate/rust_str.mjs';
import { sha256Hex } from '../crate/source_fetch.mjs';
import { userRequestText } from '../content.mjs';
import { clausesWithSpans } from '../crate/obligation_ledger.mjs';
import { mentionsRole } from '../write_lexicon.mjs';
import { sourceEvidence } from './source_contract.mjs';
import { observeSourceCallables, observedCallableGraphs, observedConditionalGraphs } from './callable_catalog.mjs';

/** Mirrors `fn plan_observed_callable_step`: reads followed by a typed contract gap. */
import { deriveCompleteSourceRequest, bindCompleteSourceNeeds } from "./complete-source-preflight.mjs";
import { sourceNeedPreflight } from "./source-need-preflight.mjs";
import { acceptedOperationSourceBytes, acceptedOperationCandidateStatus } from "./source-operation-port.mjs";
import { deriveSeededSourcePlan } from "./seeded-source-plan.mjs";
import { stepOutcome, StepOutcome, incompleteReceipt } from "../tool_result.mjs";
import { bindAcceptedImport } from "./accepted-import-authority.mjs";
import { writeArguments } from "../plan.mjs";
import { host as sourceHost } from "../host.mjs";
function originalStep(request, messages, toolNames) {
  const progress = Progress.scan(messages),
    observations = [],
    sources = [];
  const finish = (reason, disposition, detail = null) => {
    const discovery = {
      reason,
      request,
      observations,
      detail,
      missingContracts: request.inputs.map(path => ({
        path,
        required: ['inputs', 'result', 'effects', 'imports', 'optional-guard']
      })),
      goalLedger: observedCallableGoalLedger(request, messages),
      authored: false,
      verified: false
    };
    const answer = agenticMessage('callable-discovery-outcome', {
      reason,
      discovery: compactJson(discovery)
    });
    const plan = resolvedFinalAnswer(answer, disposition, 'observed-callable-discovery');
    plan.result.discovery = discovery;
    return plan;
  };
  const operands = [{
    path: request.destination,
    role: 'destination'
  }, ...request.inputs.map(path => ({
    path,
    role: 'source'
  })), ...request.acceptance.map(path => ({
    path,
    role: 'acceptance'
  }))];
  for (const operand of operands) {
    const read = progress.sourceReadFor(operand.path);
    if (read === null) {
      const tool = toolFor(toolNames, Capability.Read);
      return tool === null ? finish('MissingReadTool', FinalDisposition.Gap, operand) : planOne(tool, readArguments(operand.path));
    }
    const failure = read.error;
    if (failure !== null) {
      if (operand.role === 'destination' && /\bENOENT\b/u.test(failure)) {
        observations.push({
          ...operand,
          state: 'absent',
          contentId: null,
          bytes: null
        });
        continue;
      }
      return finish('ReadFailed', FinalDisposition.Failure, {
        ...operand,
        error: failure
      });
    }
    const content = read.source ?? '';
    if (operand.role === 'source' && read.complete) sources.push({
      path: operand.path,
      content
    });
    observations.push({
      ...operand,
      state: 'observed',
      contentId: sha256Hex(content),
      bytes: new TextEncoder().encode(content).length,
      complete: read.complete,
      providerStatus: read.status,
      ...(operand.role === 'source' && read.complete ? {
        catalog: observeSourceCallables(content, operand.path)
      } : {})
    });
  }
  return finish('MissingContract', FinalDisposition.Gap, {
    graphs: observedCallableGraphs(sources),
    conditionalGraphs: observedConditionalGraphs(sources),
    goal: 'unbound'
  });
}
/** Mirrors `fn plan_observed_callable_outcome` in rust/src/agentic_coding/module_function/discovery.rs. */
export function planObservedCallableOutcome(request, messages, toolNames) {
  const plan = planObservedCallableStep(request, messages, toolNames);
  return {
    plan,
    disposition: plan.result?.disposition ?? null,
    witness: plan.result?.discovery ?? null
  };
}

/** Mirrors observed_callable_goal_ledger: retain independent request clauses without binding unknown meaning. */
export function observedCallableGoalLedger(request, messages) {
  const user = [...messages].reverse().find(message => message.role === 'user');
  if (user === undefined) return null;
  const source = userRequestText(user.content);
  return {
    sourceIdentity: sha256Hex(source),
    bytes: new TextEncoder().encode(source).length,
    sourceBinding: 'unverified',
    semantics: 'unbound',
    declaration: {
      name: request.name,
      parameters: request.parameters,
      destination: request.destination
    },
    clauses: clausesWithSpans(source).map(([clause, [start, end]]) => ({
      ...sourceEvidence(source, start, end),
      returnAction: mentionsRole('coding_return_action', clause.toLowerCase()),
      status: 'unbound'
    })),
    needs: [{
      kind: 'read-destination',
      path: request.destination,
      status: 'unattempted'
    }, ...request.inputs.map(path => ({
      kind: 'read-source',
      path,
      status: 'unattempted'
    })), {
      kind: 'goal-binding',
      status: 'unbound'
    }, {
      kind: 'source-effects',
      status: 'unbound'
    }, {
      kind: 'import-initialization',
      status: 'unbound'
    }, ...(request.command === null ? [] : [{
      kind: 'declared-verification',
      command: request.command,
      status: 'unattempted'
    }])],
    authored: false,
    verified: false
  };
}
function planTrustedSourceCallableStep(request, messages, toolNames) {
  const SOURCE_NEED_SEED = sourceHost().sourceOperation.requestSeed();
  const SOURCE_COMPOSITION_SEED = sourceHost().sourceOperation.compositionSeed();
  const progress = Progress.scan(messages),
    observations = [],
    sources = [],
    acceptances = [];
  const finish = (reason, disposition, detail = null, state = {}) => {
    const discovery = {
      reason,
      request,
      observations,
      detail,
      missingContracts: request.inputs.map(path => ({
        path,
        required: ['inputs', 'result', 'effects', 'imports', 'optional-guard']
      })),
      goalLedger: observedCallableGoalLedger(request, messages),
      authored: state.authored === true,
      verified: state.verified === true
    };
    const answer = agenticMessage('callable-discovery-outcome', {
      reason,
      discovery: compactJson(detail?.composition ? {
        reason,
        request,
        authored: discovery.authored,
        verified: discovery.verified,
        observations: observations.map(({
          path,
          role,
          state,
          contentId,
          bytes,
          complete,
          providerStatus
        }) => ({
          path,
          role,
          state,
          contentId,
          bytes,
          complete,
          providerStatus
        })),
        plan: {
          sourceIdentity: detail.composition.sourceIdentity,
          declarations: detail.composition.graph.calls,
          bindings: detail.composition.graph.bindings,
          preconditions: detail.composition.graph.preconditions,
          effects: detail.composition.effectPlan,
          guard: detail.composition.graph.guard,
          goal: detail.composition.goal,
          moduleEffects: detail.composition.moduleEffects,
          callEffects: detail.composition.callEffects,
          effectAuthorization: detail.composition.effectAuthorization,
          rustEquivalence: detail.composition.rustEquivalence,
          needs: detail.composition.completeNeedProof ? {
            clauses: detail.composition.completeNeedProof.clauses,
            readPrerequisites: detail.composition.completeNeedProof.readPrerequisites,
            pending: detail.composition.completeNeedProof.pending,
            allowedCommand: detail.composition.completeNeedProof.allowedCommand,
            allowedDestination: detail.composition.completeNeedProof.allowedDestination,
            candidateWriteAuthority: detail.composition.completeNeedProof.candidateWriteAuthority,
            unconditionalDeliveryAuthority: detail.composition.completeNeedProof.unconditionalDeliveryAuthority,
            loadedByteRaceProof: detail.composition.completeNeedProof.loadedByteRaceProof
          } : null
        },
        verification: detail.verification
      } : discovery)
    });
    const plan = resolvedFinalAnswer(answer, disposition, 'observed-callable-discovery');
    plan.result.discovery = discovery;
    return plan;
  };
  const completeUser = [...messages].reverse().find(message => message.role === 'user');
  const completeText = completeUser === undefined ? '' : userRequestText(completeUser.content);
  const completeFrame = deriveCompleteSourceRequest(completeText, SOURCE_NEED_SEED);
  if (completeFrame === null) return finish('UnboundCompleteRequestNeeds', FinalDisposition.Gap);
  const operands = [{
    path: request.destination,
    role: 'destination'
  }, ...request.inputs.map(path => ({
    path,
    role: 'source'
  })), ...request.acceptance.map(path => ({
    path,
    role: 'acceptance'
  }))];
  for (const operand of operands) {
    const read = progress.sourceReadFor(operand.path);
    if (read === null) {
      const tool = toolFor(toolNames, Capability.Read);
      return tool === null ? finish('MissingReadTool', FinalDisposition.Gap, operand) : planOne(tool, readArguments(operand.path));
    }
    const failure = read.error;
    if (failure !== null) {
      if (operand.role === 'destination' && /\bENOENT\b/u.test(failure)) {
        observations.push({
          ...operand,
          state: 'absent',
          contentId: null,
          bytes: null
        });
        continue;
      }
      return finish('ReadFailed', FinalDisposition.Failure, {
        ...operand,
        error: failure
      });
    }
    const content = read.source ?? '';
    if (operand.role === 'source' && read.complete) sources.push({
      path: operand.path,
      content
    });
    if (operand.role === 'acceptance' && read.complete) acceptances.push({
      path: operand.path,
      content
    });
    observations.push({
      ...operand,
      state: 'observed',
      contentId: sha256Hex(content),
      bytes: new TextEncoder().encode(content).length,
      complete: read.complete,
      providerStatus: read.status,
      ...(operand.role === 'source' && read.complete ? {
        catalog: observeSourceCallables(content, operand.path)
      } : {})
    });
  }
  const importReuse = [];
  const originalUser = [...messages].reverse().find(message => message.role === 'user');
  const originalText = originalUser === undefined ? '' : userRequestText(originalUser.content);
  const workspace = messages.find(message => message.role === 'system')?.content?.match(/Working directory: ([^\n]+)\n/)?.[1];
  let operationSources = null;
  const observation = [...messages].reverse().find(message => message.role === 'tool' && message.accepted_operation_sources !== undefined);
  if (observation !== undefined) {
    try {
      operationSources = acceptedOperationSourceBytes(observation.accepted_operation_sources, originalText, request.command, workspace);
    } catch (error) {
      return finish('UnqualifiedProviderSourceWitness', FinalDisposition.Gap, {
        reason: error.message
      });
    }
  }
  for (const acceptance of acceptances) {
    if (request.command !== 'node --test ' + acceptance.path) continue;
    const accepted = observeSourceCallables(acceptance.content, acceptance.path);
    for (const imported of accepted.imports) {
      // Absolute canonical source URLs alone are qualified here. Relative and package resolution remain unknown.
      if (!imported.specifier.startsWith('/') || imported.specifier.includes('?') || imported.specifier.includes('#')) continue;
      const candidates = sources.filter(source => {
        const catalog = observeSourceCallables(source.content, source.path);
        return imported.bindings.some(binding => catalog.exports.some(value => value.exposed === binding.imported));
      });
      if (candidates.length !== 1) continue;
      if (operationSources === null) {
        const tool = toolFor(toolNames, Capability.Run);
        if (tool === null) return finish('MissingOperationSourceProvider', FinalDisposition.Gap, {
          path: imported.specifier
        });
        const plan = planOne(tool, JSON.stringify({
          command: request.command
        }));
        plan.sourceOperationPreflight = true;
        return plan;
      }
      const actual = operationSources.filter(value => value.path === imported.specifier);
      if (actual.length !== 1 || actual[0].sha256 !== sha256Hex(candidates[0].content)) return finish('SourceIdentityMismatch', FinalDisposition.Gap, {
        path: imported.specifier
      });
      const source = {
        path: imported.specifier,
        content: actual[0].content
      };
      for (const binding of imported.bindings) {
        try {
          importReuse.push(bindAcceptedImport(acceptance, source, {
            specifier: imported.specifier,
            contentId: sha256Hex(source.content),
            exported: binding.imported
          }, {
            command: request.command,
            acceptanceOperand: acceptance.path,
            acceptancePath: acceptance.path,
            acceptanceIdentity: sha256Hex(acceptance.content)
          }));
        } catch (error) {
          return finish('UnqualifiedAcceptedImport', FinalDisposition.Gap, {
            path: imported.specifier,
            reason: error.message
          });
        }
      }
    }
  }
  let composition = null,
    compositionGap = null;
  const user = [...messages].reverse().find(message => message.role === 'user');
  if (user !== undefined && acceptances.length === 1 && importReuse.length === 2) {
    try {
      const canonical = importReuse.map(receipt => {
        const path = new URL(receipt.moduleURL).pathname;
        const source = operationSources?.find(value => value.path === path);
        if (source === undefined) throw new Error(contractText('unobserved-provider-canonical-source'));
        return {
          path,
          content: source.content
        };
      });
      const text = userRequestText(user.content),
        acceptance = acceptances[0];
      composition = deriveSeededSourcePlan({
        ...request,
        text,
        identity: sha256Hex(text)
      }, canonical, acceptance, {
        command: request.command,
        acceptanceOperand: acceptance.path,
        acceptancePath: acceptance.path,
        acceptanceIdentity: sha256Hex(acceptance.content)
      }, SOURCE_COMPOSITION_SEED);
    } catch (error) {
      compositionGap = error.message;
    }
  }
  if (composition !== null) {
    const declaredDestination = observations.find(value => value.role === 'destination');
    if (declaredDestination?.state !== 'absent') return finish('UnprovedDestinationMerge', FinalDisposition.Gap, {
      composition
    });
    try {
      composition = {
        ...composition,
        completeNeedProof: bindCompleteSourceNeeds(completeFrame, composition, sourceNeedPreflight(completeText, messages), operationSources)
      };
    } catch (error) {
      return finish('UnsolvedSourceCompositionNeed', FinalDisposition.Gap, {
        reason: error.message
      });
    }
    const written = progress.successfulWriteContentFor(request.destination);
    if (written === null) {
      if (progress.attemptedWriteFor(request.destination)) return finish('WriteFailed', FinalDisposition.Failure, {
        composition
      });
      const tool = toolFor(toolNames, Capability.Write);
      if (tool === null) return finish('MissingWriteTool', FinalDisposition.Gap, {
        composition
      });
      const plan = planOne(tool, writeArguments(request.destination, composition.source));
      plan.sourceComposition = composition;
      return plan;
    }
    if (written !== composition.source) return finish('SourceArtifactMismatch', FinalDisposition.Gap, {
      composition
    });
    const afterWrite = progress.attemptsAfterLatestWrite(request.destination);
    if (afterWrite === null) return finish('MissingWriteReceipt', FinalDisposition.Gap, {
      composition
    });
    for (const path of [...request.inputs, acceptances[0].path]) {
      const fresh = afterWrite.some(attempt => attempt.capability === Capability.Read && attempt.source_read?.complete === true && attempt.arguments !== null && JSON.parse(attempt.arguments).path === path);
      if (!fresh) {
        const tool = toolFor(toolNames, Capability.Read);
        return tool === null ? finish('MissingReadTool', FinalDisposition.Gap, {
          path
        }) : planOne(tool, readArguments(path));
      }
    }
    const afterWriteRuns = afterWrite.filter(attempt => attempt.capability === Capability.Run);
    const output = afterWriteRuns.length === 0 ? null : progress.latestRunOutputFor(request.command);
    if (output === null) {
      const tool = toolFor(toolNames, Capability.Run);
      if (tool === null) return finish('MissingVerificationTool', FinalDisposition.Gap, {
        composition
      });
      const plan = planOne(tool, JSON.stringify({
        command: request.command
      }));
      plan.sourceComposition = composition;
      return plan;
    }
    let ownedStatus = null,
      verificationFailure = null;
    try {
      if (observation === undefined) throw new Error('MissingOwnedVerificationReceipt');
      ownedStatus = acceptedOperationCandidateStatus(observation.accepted_operation_sources, originalText, request.command, workspace, request.destination, sha256Hex(composition.source));
    } catch (error) {
      verificationFailure = error.message;
    }
    const verified = ownedStatus === 0 && stepOutcome(output) === StepOutcome.Succeeded && !incompleteReceipt(output);
    return finish(verified ? 'SourceCompositionVerified' : 'SourceCompositionVerificationFailed', verified ? FinalDisposition.Artifact : FinalDisposition.Failure, {
      composition,
      verification: {
        command: request.command,
        output,
        ownedStatus,
        verificationFailure
      }
    }, {
      authored: true,
      verified
    });
  }
  return finish('MissingContract', FinalDisposition.Gap, {
    acceptedImportReuse: importReuse,
    executionAuthority: 'not-yet-executed',
    compositionGap,
    graphs: observedCallableGraphs(sources),
    conditionalGraphs: observedConditionalGraphs(sources),
    goal: 'unbound'
  });
}
export function planObservedCallableStep(request, messages, tools) {
  if (!sourceHost().sourceOperation?.hasContext?.()) return originalStep(request, messages, tools);
  return planTrustedSourceCallableStep(request, messages, tools);
}
