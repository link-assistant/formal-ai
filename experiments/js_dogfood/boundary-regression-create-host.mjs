// Experimental dogfood driver provider. No production/native feature claim.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { createScopedSourceEditHost } from './scoped-source-edit-host.mjs';
import { exclusiveCreate } from './exclusive-create.mjs';
import { boundaryCandidateObservations, deriveBoundaryWorkItemCandidate } from '../../js/agentic/work_item_boundary_candidate.mjs';
const aliases = ['filePath', 'file_path', 'path'];
const hash = text => createHash('sha256').update(text, 'utf8').digest('hex');
const failure = (code, target = null) => ({
  content: JSON.stringify({
    is_error: true,
    error: code
  }),
  is_error: true,
  ...(target ? {
    source_read: {
      path: target,
      success: false,
      complete: false,
      format: 'raw',
      error_code: code
    }
  } : {})
});
function argumentsOf(call) {
  try {
    return JSON.parse(call.arguments);
  } catch {
    return null;
  }
}
function boundArguments(call, keys, owner) {
  const argumentsValue = argumentsOf(call);
  if (!argumentsValue || typeof argumentsValue !== 'object' || Array.isArray(argumentsValue) || Object.keys(argumentsValue).some(key => !keys.includes(key))) return null;
  const paths = aliases.filter(key => Object.hasOwn(argumentsValue, key)).map(key => argumentsValue[key]);
  return paths.length > 0 && paths.every(value => value === owner) ? argumentsValue : null;
}

/** Scoped single-owner scratch custody; not a sandbox against hostile JavaScript
 * or concurrent external namespace relocation. Root/paths are rechecked and
 * physical creation uses the existing descriptor-relative exclusive adapter. */
export function createBoundaryRegressionCreateHost({
  workspace,
  contract: providedContract,
  operation
}) {
  const contract = Object.freeze(structuredClone(providedContract));
  if (contract?.mode !== 'regression' || typeof operation !== 'function') throw Error('BoundaryRegressionCapabilityRequired');
  for (const owner of [contract.owner, contract.target]) if (typeof owner !== 'string' || path.isAbsolute(owner) || owner.includes('\\') || owner.split('/').some(part => !part || part === '.' || part === '..' || part.includes('\0'))) throw Error('BoundaryRegressionOwnedPathRequired');
  if (contract.owner === contract.target) throw Error('BoundaryRegressionDistinctPathsRequired');
  const root = path.resolve(workspace),
    initial = fs.lstatSync(root, {
      bigint: true
    });
  if (!initial.isDirectory() || initial.isSymbolicLink() || fs.realpathSync(root) !== root) throw Error('BoundaryRegressionWorkspaceRequired');
  const identity = {
    dev: String(initial.dev),
    ino: String(initial.ino)
  };
  const sourceReader = createScopedSourceEditHost({
    workspace: root,
    ownedPath: contract.owner,
    maximumBytes: 131072
  });
  const authorized = new WeakMap();
  let creationAttempted = false,
    targetReader = null;
  const current = () => {
    try {
      const now = fs.lstatSync(root, {
        bigint: true
      });
      return now.isDirectory() && !now.isSymbolicLink() && String(now.dev) === identity.dev && String(now.ino) === identity.ino && fs.realpathSync(root) === root;
    } catch {
      return false;
    }
  };
  const canonicalRead = owner => ({
    tool: 'read',
    arguments: JSON.stringify({
      filePath: owner
    })
  });
  function targetState() {
    let currentPath = root;
    const parts = contract.target.split('/');
    for (let index = 0; index < parts.length; index++) {
      currentPath = path.join(currentPath, parts[index]);
      let status;
      try {
        status = fs.lstatSync(currentPath, {
          bigint: true
        });
      } catch (error) {
        if (error.code === 'ENOENT') return {
          absent: true,
          error
        };
        throw error;
      }
      if (status.isSymbolicLink() || (index < parts.length - 1 ? !status.isDirectory() : !status.isFile() || status.nlink !== 1n)) throw Error('BoundaryRegressionTargetPathRefused');
    }
    return {
      absent: false
    };
  }
  function readTarget() {
    const state = targetState();
    if (state.absent) return {
      content: JSON.stringify({
        is_error: true,
        error: state.error.message
      }),
      is_error: true,
      source_read: {
        path: contract.target,
        success: false,
        complete: false,
        format: 'raw',
        error_code: 'ENOENT'
      }
    };
    targetReader ??= createScopedSourceEditHost({
      workspace: root,
      ownedPath: contract.target,
      maximumBytes: 262144
    });
    return targetReader.execute(canonicalRead(contract.target));
  }
  function authorizeTurn(calls, maintained, messages) {
    if (!current()) return 'ChangedSemanticRegressionWorkspace';
    if (maintained?.kind !== 'tool_calls' || calls.length !== 1 || maintained.calls.length !== calls.length || !calls.every((call, index) => call.tool === maintained.calls[index].tool && call.arguments === maintained.calls[index].arguments)) return 'UnboundSemanticRegressionPlan';
    const call = calls[0];
    if (call.tool === 'read') {
      const owner = [contract.owner, contract.target].find(owner => boundArguments(call, aliases, owner));
      if (!owner) return 'UnboundSemanticRegressionRead';
      authorized.set(call, {
        kind: 'read',
        owner
      });
      return null;
    }
    if (call.tool !== 'write' || creationAttempted) return 'UndeclaredSemanticRegressionEffect';
    const argumentsValue = boundArguments(call, [...aliases, 'content'], contract.target);
    if (!argumentsValue || typeof argumentsValue.content !== 'string') return 'UnboundSemanticRegressionWrite';
    const proposal = deriveBoundaryWorkItemCandidate(contract, boundaryCandidateObservations(contract, messages), operation);
    const candidate = proposal.candidate;
    if (!candidate || candidate.mode !== 'regression' || candidate.path !== contract.target || candidate.sourceOwner !== contract.owner ||
     candidate.requiresAbsent !== true || candidate.noOp !== false || candidate.after !== argumentsValue.content || candidate.afterIdentity !== hash(argumentsValue.content) ||
     Buffer.byteLength(argumentsValue.content, 'utf8') > 262144) return 'UnboundSemanticRegressionCandidate';
    if (!targetState().absent) return 'SemanticRegressionTargetAlreadyExists';
    authorized.set(call, {
      kind: 'write',
      candidate
    });
    return null;
  }
  function execute(call) {
    const permission = authorized.get(call);
    authorized.delete(call);
    if (!permission) return failure('UnboundSemanticRegressionOperation');
    try {
      if (!current()) return failure('ChangedSemanticRegressionWorkspace');
      if (permission.kind === 'read') return permission.owner === contract.owner ? sourceReader.execute(canonicalRead(contract.owner)) : readTarget();
      if (creationAttempted) return failure('SemanticRegressionCreationAlreadyAttempted');
      creationAttempted = true;
      const source = sourceReader.execute(canonicalRead(contract.owner));
      if (source.is_error === true || hash(source.content) !== permission.candidate.sourceIdentity) return failure('SemanticRegressionSourcePreimageChanged');
      if (!targetState().absent) return failure('SemanticRegressionTargetAlreadyExists');
      const receipt = exclusiveCreate(root, contract.target, permission.candidate.after, identity, {
        timeoutMs: 5000
      });
      return {
        ...receipt,
        semantic_creation: {
          kind: 'comparison-boundary-regression',
          owner: contract.owner,
          sourceIdentity: permission.candidate.sourceIdentity,
          candidateIdentity: permission.candidate.afterIdentity,
          maintainedPlanMatch: true,
          independentAcceptance: false
        }
      };
    } catch (error) {
      return failure(error.code ?? error.message ?? 'SemanticRegressionProviderFailure', permission.kind === 'read' ? permission.owner : null);
    }
  }
  const authorize = (calls, maintained, messages) => {
    try {
      return authorizeTurn(calls, maintained, messages);
    } catch (error) {
      return error.code ?? error.message ?? 'SemanticRegressionGuardFailure';
    }
  };
  return Object.freeze({
    authorize,
    execute,
    current
  });
}
