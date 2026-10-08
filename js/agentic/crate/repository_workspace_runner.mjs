// Shared JavaScript execution of the repository protocol's declared stages.
// File and process operations belong to the injected workspace.io port.
import { stableId } from './engine_stable_identifier.mjs';
import { observedEvidence, ObservationKind, EvidenceSource } from './execution_evidence.mjs';
import { EDITOR_STRUCTURAL, loadProtocol, newProtocolTrace, recordStage, renderProtocolTrace,
  renderProtocolTemplate, stepAppliesTo } from './repository_workspace.mjs';
import { deriveRepositoryChange, locateRepositoryTargets, locateRepositoryAmbiguity, repositoryCommandArguments, runRepositoryCommand }
  from './repository_workspace_stages.mjs';

function observed(outcome, step, command, argumentsList, exit, output, kind, source = EvidenceSource.LocalProcess) {
  const evidence = observedEvidence(command, argumentsList, exit, new TextEncoder().encode(output), kind, source);
  const row = outcome.need_ledger.rows.find((candidate) => candidate.route === step.id);
  evidence.for_need = row.need_id;
  evidence.produced_by = `repository_protocol_${step.id}`;
  // Only a linked successful observation can upgrade this planned need.
  if ((exit === null && evidence.observed_byte_length > 0) || exit === 0) row.status = 'satisfied';
  outcome.observations.push(evidence);
  return evidence;
}

function protocolLedger(steps, task) {
  const frame = stableId('repository_protocol', `${task.clone.base_commit}:${task.requirement}`);
  return {
    frame_id: frame,
    rows: steps.filter((step) => stepAppliesTo(step, EDITOR_STRUCTURAL)).map((step) => ({
      need_id: stableId('repository_protocol_step', `${frame}:${step.id}`),
      source_span: task.requirement,
      status: 'planned',
      leaf_reason: 'direct_method',
      unit_id: null,
      route: step.id,
    })),
  };
}

/**
 * Mirrors `WorkspaceProtocol::execute` with injected workspace reads, writes and processes.
 * io.run(root, program, arguments, policy) returns observed exit/output data.
 * io.sourceFiles(root) returns [relative path, source] pairs; io.census(root) is optional.
 * No process or file operation is performed outside this port.
 */
export async function executeWorkspaceProtocol(workspace, task, {
  steps = loadProtocol(), caller = 'solve', trace_path = 'repository-protocol.lino',
} = {}) {
  const trace = newProtocolTrace(steps, caller, EDITOR_STRUCTURAL);
  const outcome = {
    located: [], edited: [], observations: [], diff: '', stopped_at: null, open: [],
    need_ledger: protocolLedger(steps, task), trace,
  };
  const source = new Map();
  for (const step of steps) {
    if (!stepAppliesTo(step, EDITOR_STRUCTURAL)) continue;
    try {
      let status = 'observed';
      switch (step.id) {
        case 'clone': {
          const result = await runRepositoryCommand(workspace, 'git', ['rev-parse', 'HEAD']);
          const head = result.exit_code === 0 ? (result.stdout ?? '').trim() : 'unavailable';
          if (!/^[a-fA-F0-9]{40}$/u.test(task.clone.base_commit) || head !== task.clone.base_commit) {
            throw new Error(renderProtocolTemplate('workspace_base_commit_mismatch', [
              ['expected', task.clone.base_commit], ['observed', head],
            ]) ?? 'workspace_base_commit_mismatch');
          }
          observed(outcome, step, 'git rev-parse HEAD', ['HEAD'], 0, head, ObservationKind.CommandExit);
          break;
        }
        case 'locate': {
          const files = await workspace.io.sourceFiles(workspace.root);
          const census = workspace.io.census ? await workspace.io.census(workspace.root) : { modules: [] };
          outcome.located = locateRepositoryTargets(files, task.requirement, census, task.language ?? '');
          if (!outcome.located.length) {
            const candidates = locateRepositoryAmbiguity(files, task.requirement, census, task.language ?? '');
            throw new Error(candidates.length ? 'ambiguous repository declarations: '
              + candidates.map((candidate) => candidate.relative_path + ':' + candidate.symbol).join(', ') : task.requirement);
          }
          observed(outcome, step, 'repository locate', [], null,
            outcome.located.map((location) => `${location.relative_path}:${location.symbol ?? ''}:${location.how}`).join('\n'),
            ObservationKind.SymbolicCheck, EvidenceSource.Engine);
          break;
        }
        case 'read': {
          for (const location of outcome.located) {
            if (!source.has(location.relative_path)) source.set(location.relative_path,
              await workspace.io.read(workspace.root, location.relative_path));
          }
          const evidence = observed(outcome, step, 'repository read', [...source.keys()], null,
            [...source.values()].join(''), ObservationKind.FileBytes);
          evidence.source_ids = [...source].map(([path, contents]) => stableId('repository_source', `${path}:${contents}`));
          break;
        }
        case 'edit': {
          const changes = [...new Map(outcome.located.map((location) => [location.relative_path, location])).values()]
            .map((location) => deriveRepositoryChange(location, source.get(location.relative_path), task.requirement)).filter(Boolean);
          if (!changes.length) throw new Error('no registry-grounded structural edit was derivable');
          for (const change of changes) {
            await workspace.io.write(workspace.root, change.relative_path, change.contents);
            const contents = await workspace.io.read(workspace.root, change.relative_path);
            if (contents !== change.contents) throw new Error(`write was not observed: ${change.relative_path}`);
            outcome.edited.push(change.relative_path);
            observed(outcome, step, `write ${change.relative_path}`, [change.relative_path], null, contents, ObservationKind.FileBytes);
          }
          break;
        }
        case 'verify': {
          if (!task.tests) { status = 'unobserved'; break; }
          const argumentsList = repositoryCommandArguments(task.tests.line);
          const result = await runRepositoryCommand(workspace, argumentsList[0], argumentsList.slice(1), { probe: true });
          const evidence = observed(outcome, step, task.tests.line, argumentsList, result.exit_code,
            `${result.stdout ?? ''}${result.stderr ?? ''}`, ObservationKind.CommandExit);
          evidence.produced_by = 'repository_workspace_named_tests';
          evidence.detail = { kind: 'tests', passed: result.exit_code === 0 ? task.tests.names : [],
            failed: result.exit_code === 0 ? [] : task.tests.names, timed_out: false };
          if (result.exit_code !== 0) throw new Error('named tests did not pass');
          break;
        }
        case 'diff': {
          const intent = await runRepositoryCommand(workspace, 'git', ['add', '--intent-to-add', '--all']);
          if (intent.exit_code !== 0) throw new Error(intent.stderr ?? 'git add failed');
          const result = await runRepositoryCommand(workspace, 'git', ['diff']);
          if (result.exit_code !== 0) throw new Error(result.stderr ?? 'git diff failed');
          outcome.diff = result.stdout ?? '';
          observed(outcome, step, 'git diff', [], 0, outcome.diff, ObservationKind.CommandExit);
          break;
        }
        case 'commit': status = 'refused'; break;
        default: status = 'unobserved';
      }
      recordStage(trace, step.id, status);
    } catch (error) {
      outcome.stopped_at = step;
      outcome.open.push(error.message);
      trace.open.push(error.message);
      recordStage(trace, step.id, 'stopped');
      break;
    }
  }
  outcome.trace_document = renderProtocolTrace(trace);
  if (trace_path !== null) await workspace.io.write(workspace.root, trace_path, outcome.trace_document);
  return outcome;
}
