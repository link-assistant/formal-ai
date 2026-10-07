// `crate::orchestration::incremental` (rust/src/orchestration/incremental.rs):
// incremental, failure-driven dispatch to external agentic CLIs. The whole task
// is attempted first. Only a failure justifies a split, the pieces are executed
// by the same rule, and the parent is re-attempted once its pieces pass. An
// irreducible task that one CLI failed is escalated to the next CLI that has not
// tried it; when the list is exhausted the task is reported blocked with the
// evidence of every attempt, which is exactly the input the review-gated
// learning path needs.
//
// The controller is `solveRecursively` (recursive_execution.mjs) and the
// splitter is `splittingExecutor` (task_decomposition_tree.mjs); this module only
// supplies the tool boundary: an attempt is one recorded agent session in a copy
// of the workspace, and a passing attempt's effects are applied to the workspace
// before the next one starts.

import fs from 'node:fs';
import path from 'node:path';

import { learnClientContracts, learningLinksNotation } from './client_contract_learning.mjs';
import { stableId } from './engine_stable_id.mjs';
import { pushLinoNode } from './links_format.mjs';
import { observeOrchestrationSession } from './orchestration_analysis.mjs';
import { commitVerifiedEffect, validateEffectAttribution } from './orchestration_attribution.mjs';
import { candidateRunConfig, pad3, safeName } from './orchestration_dispatch.mjs';
import { DispatchError } from './orchestration_dispatch_error.mjs';
import { AgentRunError, runAgent, sessionDiffSize, sessionPassed, verifyWorkspace } from './orchestration_runner.mjs';
import { writeSession } from './orchestration_session_file.mjs';
import { applyChanges, copyWorkspace, ioErrorFrom } from './orchestration_workspace.mjs';
import {
  attemptFailed, attemptPassed, blockedLeaves, isPassed, recursiveLeaf, solveRecursively, splitDepthReached,
} from './recursive_execution.mjs';
import { clientIntegrations } from './seed_client_integrations.mjs';
import { splittingExecutor } from './task_decomposition_tree.mjs';

const COMPOSED_VERIFIER = 'composed-verifier';

/** `{:?}` of an `AgentStatus`. */
const STATUS_DEBUG = { succeeded: 'Succeeded', failed: 'Failed', timed_out: 'TimedOut' };

/**
 * Mirrors `IncrementalProposal::links_notation` in
 * rust/src/orchestration/incremental.rs: one proposal in the review-gated
 * learning vocabulary.
 */
export function proposalLinksNotation(proposal) {
  let out = pushLinoNode('', 0, 'incremental_proposal', proposal.id);
  out = pushLinoNode(out, 2, 'task', proposal.task);
  for (const cli of proposal.tried_clis) out = pushLinoNode(out, 2, 'tried_cli', cli);
  for (const evidence of proposal.failure_evidence) out = pushLinoNode(out, 2, 'failure_evidence', evidence);
  return pushLinoNode(out, 2, 'status', proposal.status);
}

/** Mirrors `fn root_id` in rust/src/orchestration/incremental.rs. */
const rootId = (task) => stableId('dispatch_task', task);

/**
 * Mirrors `fn write_learning` in rust/src/orchestration/incremental.rs: feed the
 * exact execution sessions into the proposal-only client contract learner.
 */
function writeLearning(outputDir, steps, sessions) {
  const observations = steps
    .map((step, index) => [step, sessions[index]])
    .filter(([step]) => step.cli !== COMPOSED_VERIFIER)
    .map(([step, session]) => observeOrchestrationSession(session, step.session_file));
  const learning = learnClientContracts(observations, clientIntegrations());
  try {
    fs.writeFileSync(path.join(outputDir, 'learning.lino'), learningLinksNotation(learning));
  } catch (error) {
    throw new DispatchError('io', ioErrorFrom(error));
  }
}

/** Mirrors `fn blocked_tasks`: goals of the irreducible tasks a run could not solve. */
const blockedTasks = (run) => blockedLeaves(run).map((leaf) => leaf.task.goal);

/**
 * Mirrors `fn proposals` in rust/src/orchestration/incremental.rs: one review
 * request per blocked task, in execution order.
 */
function proposalsOf(run, steps) {
  return blockedLeaves(run).map((leaf) => {
    const triedClis = [];
    for (const attempt of steps.filter((step) => step.task_id === leaf.task.id)) {
      if (!triedClis.includes(attempt.cli)) triedClis.push(attempt.cli);
    }
    const failureEvidence = leaf.attempts.filter((attempt) => !attempt.passed).map((attempt) => attempt.evidence);
    return {
      id: stableId('incremental_proposal', `${leaf.task.id}|${failureEvidence.join('|')}`),
      task: leaf.task.goal,
      tried_clis: triedClis,
      failure_evidence: failureEvidence,
      status: 'human_review_required',
    };
  });
}

/**
 * Mirrors `fn write_proposals` in rust/src/orchestration/incremental.rs: mirror
 * the proposals next to the report, empty but for its header when nothing is blocked.
 */
function writeProposals(outputDir, proposals) {
  let document = pushLinoNode('', 0, 'incremental_proposals', null);
  for (const proposal of proposals) document += proposalLinksNotation(proposal);
  try {
    fs.writeFileSync(path.join(outputDir, 'proposals.lino'), document);
  } catch (error) {
    throw new DispatchError('io', ioErrorFrom(error));
  }
}

/**
 * Mirrors `fn evidence_of` in rust/src/orchestration/incremental.rs: what an
 * attempt observed, as whitespace-free `key:value` tokens.
 */
function evidenceOf(cli, session) {
  const verification = session.verification.map((result) => `${result.program}=${result.passed}`).join(',');
  const exit = session.exit_code === null || session.exit_code === undefined ? 'none' : String(session.exit_code);
  return [`cli:${cli}`, `status:${STATUS_DEBUG[session.status]}`, `exit:${exit}`, `verification:${verification}`].join(' ');
}

/**
 * Mirrors `struct AgentExecutor` in rust/src/orchestration/incremental.rs: the
 * tool boundary where one attempt is one recorded agent session. The first
 * controller error is kept in `error` because the executor interface has no way
 * to report one; a run that hit it is reported as an error, never as a failed
 * task, since a controller fault is not evidence about the task.
 */
function agentExecutor(config, workspace, outputDir) {
  const executor = {
    sessions: [],
    steps: [],
    changes: [],
    escalations: new Map(),
    error: null,

    /** `AgentExecutor::cli_for`: the first CLI, then one further along per escalation. */
    cliFor(task) {
      const index = executor.escalations.get(task.id) ?? 0;
      return config.clis[Math.min(index, config.clis.length - 1)];
    },

    /** `AgentExecutor::record`: keep the first controller error. */
    record(error) {
      if (executor.error === null) executor.error = error;
    },

    /** `AgentExecutor::record_preserved_session`. */
    recordPreservedSession(task, cli, session, sessionFile) {
      const passed = sessionPassed(session);
      const evidence = evidenceOf(cli, session);
      executor.steps.push({ task_id: task.id, task: task.goal, cli, passed, session_file: sessionFile });
      executor.sessions.push(session);
      return passed ? attemptPassed(evidence) : attemptFailed(evidence);
    },

    /** `AgentExecutor::preserve_session`. */
    preserveSession(task, cli, session) {
      const index = executor.sessions.length;
      const sessionFile = `sessions/${pad3(index)}-${safeName(cli)}.json`;
      try {
        writeSession(path.join(outputDir, sessionFile), session);
      } catch (error) {
        executor.record(new DispatchError('replay', error));
        return attemptFailed('controller_aborted');
      }
      return executor.recordPreservedSession(task, cli, session, sessionFile);
    },

    /** `TaskExecutor::attempt` for `AgentExecutor`. */
    async attempt(task) {
      if (executor.error !== null) return attemptFailed('controller_aborted');
      const index = executor.sessions.length;
      const cli = executor.cliFor(task);
      const candidate = path.join(outputDir, 'candidates', `${pad3(index)}-${safeName(cli)}`);
      try {
        copyWorkspace(workspace, candidate, outputDir);
      } catch (error) {
        executor.record(new DispatchError('io', ioErrorFrom(error)));
        return attemptFailed('controller_aborted');
      }
      const home = path.join(outputDir, 'native-sessions', `${pad3(index)}-${safeName(cli)}`);
      const run = candidateRunConfig(config, cli, task.goal, candidate, home);
      let session;
      try {
        session = await runAgent(run);
      } catch (error) {
        if (!(error instanceof AgentRunError)) throw error;
        executor.record(new DispatchError('run', error));
        return attemptFailed('controller_aborted');
      }
      const passed = sessionPassed(session);
      const sessionFile = `sessions/${pad3(index)}-${safeName(cli)}.json`;
      const sessionPath = path.join(outputDir, sessionFile);
      try {
        writeSession(sessionPath, session);
      } catch (error) {
        executor.record(new DispatchError('replay', error));
        return attemptFailed('controller_aborted');
      }
      if (passed && config.pull_request !== null) {
        try {
          validateEffectAttribution(workspace, sessionPath, session);
        } catch (error) {
          executor.record(error);
          return attemptFailed('controller_aborted');
        }
      }
      // A passing attempt's effects become the starting point of the next one, so
      // a later sub-task -- and the parent's own retry -- sees the work its
      // predecessors did instead of a workspace that forgot it.
      if (passed) {
        try {
          applyChanges(workspace, candidate, session.changes);
        } catch (error) {
          executor.record(new DispatchError('io', ioErrorFrom(error)));
          return attemptFailed('controller_aborted');
        }
        if (config.pull_request !== null) {
          try {
            commitVerifiedEffect(workspace, sessionPath, session, config.pull_request);
          } catch (error) {
            executor.record(error);
            return attemptFailed('controller_aborted');
          }
        }
        for (const change of session.changes) {
          executor.changes = executor.changes.filter((prior) => prior.path !== change.path);
          executor.changes.push(change);
        }
      }
      return executor.recordPreservedSession(task, cli, session, sessionFile);
    },

    /** `TaskExecutor::extend_for` for `AgentExecutor`: escalate to the next CLI not yet tried. */
    async extend_for(task) {
      if (executor.error !== null) return false;
      const next = (executor.escalations.get(task.id) ?? 0) + 1;
      if (next >= config.clis.length) return false;
      executor.escalations.set(task.id, next);
      return true;
    },

    /** `TaskExecutor::retry_after_children` for `AgentExecutor`. */
    async retry_after_children(task) {
      if (executor.error !== null) return attemptFailed('controller_aborted');
      if (config.verification.length === 0) return executor.attempt(task);
      const cli = executor.cliFor(task);
      const home = path.join(outputDir, 'native-sessions', `${pad3(executor.sessions.length)}-${safeName(cli)}`);
      const run = candidateRunConfig(config, cli, task.goal, workspace, home);
      let session;
      try {
        session = await verifyWorkspace(run);
      } catch (error) {
        if (!(error instanceof AgentRunError)) throw error;
        executor.record(new DispatchError('run', error));
        return attemptFailed('controller_aborted');
      }
      if (sessionPassed(session)) return executor.preserveSession(task, COMPOSED_VERIFIER, session);
      return executor.attempt(task);
    },
  };
  return executor;
}

/**
 * Mirrors `fn dispatch_incrementally` in rust/src/orchestration/incremental.rs:
 * run one task incrementally and report every attempt, split, and effect.
 * @returns {Promise<object>} the `DispatchReport` carrying an `incremental` trace
 */
export async function dispatchIncrementally(config, workspace, outputDir) {
  const root = recursiveLeaf(rootId(config.task), config.task);
  const agent = agentExecutor(config, workspace, outputDir);
  const executor = splittingExecutor(agent);
  const run = await solveRecursively(root, executor);
  const splits = executor.splits.map((split) => ({
    task: split.goal,
    failure_evidence: split.failure_evidence,
    split_depth: split.split_depth,
    children: split.children,
  }));
  if (agent.error !== null) throw agent.error;
  writeLearning(outputDir, agent.steps, agent.sessions);

  const entries = agent.steps.map((step, index) => ({
    cli: step.cli,
    task: agent.sessions[index].task,
    passed: sessionPassed(agent.sessions[index]),
    diff_size: sessionDiffSize(agent.sessions[index]),
    wall_time_ms: agent.sessions[index].wall_time_ms,
    session_file: step.session_file,
  }));
  const proposals = proposalsOf(run, agent.steps);
  writeProposals(config.output_dir, proposals);
  const trace = {
    schema: 'formal-ai-incremental-trace-v1',
    solved: isPassed(run),
    split_depth_reached: splitDepthReached(run),
    steps: agent.steps,
    splits,
    blocked_tasks: blockedTasks(run),
    proposals,
  };
  return {
    schema: 'formal-ai-dispatch-report-v1',
    mode: 'incremental',
    tasks: trace.steps.map((step) => step.task),
    sessions: agent.sessions,
    ledger: {
      schema: 'formal-ai-comparison-ledger-v1',
      selection_rule: 'pass,diff_size,wall_time,cli,session_file',
      entries,
      winner: null,
    },
    composed_changes: agent.changes,
    incremental: trace,
  };
}
