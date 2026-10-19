import fs from 'node:fs';
import path from 'node:path';
import {randomUUID, createHash} from 'node:crypto';
import {serverMessage} from './messages.mjs';
import {WorkerHost} from './worker-host.mjs';
import {installDefaultNodeSourceHost} from './default-node-source-bootstrap.mjs';
import {host} from '../agentic/host.mjs';
import {observedCallableRequest} from '../agentic/module_function.mjs';
import {deriveCompleteSourceRequest} from '../agentic/module_function/complete-source-preflight.mjs';
import {planChatStep} from '../agentic/planner.mjs';
import {drive} from '../../experiments/js_dogfood/drive.mjs';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
/** Local constructor-owned transport; external receipt fields are not authority. */
export async function prepareOwnedNodeSourceAuthoring(authoring) {
  if (authoring.model && authoring.model !== 'formal-ai/javascript') return null;
  const request = authoring.task;
  if (!observedCallableRequest(request)) return null;
  await installDefaultNodeSourceHost(new WorkerHost());
  const installed = host();
  const frame = deriveCompleteSourceRequest(request,
    installed.readText('data/seed/source-authoring-grammar.lino'));
  if (!frame || authoring.commit === true) {
    throw Error(serverMessage('authoring-session-incomplete'));
  }
  const produced = [...authoring.produces];
  const destinations = authoring.into ? [...authoring.into] : produced;
  const required = [...frame.request.inputs, ...frame.request.acceptance];
  const context = [...(authoring.context ?? [])];
  if (produced.length !== 1 || produced[0] !== frame.request.destination
      || destinations.length !== 1 || destinations[0] !== produced[0]
      || new Set(context).size !== context.length
      || context.length !== required.length
      || required.some(operand => !context.includes(operand))) {
    throw Error(serverMessage('authoring-artifacts-required'));
  }
  const canonicalRoot = root => {
    let current = path.resolve(root);
    const suffix = [];
    while (true) {
      try {
        fs.lstatSync(current);
      } catch (error) {
        if (error.code !== 'ENOENT' || path.dirname(current) === current) throw error;
        suffix.unshift(path.basename(current));
        current = path.dirname(current);
        continue;
      }
      const resolved = fs.realpathSync(current);
      if (!fs.statSync(resolved).isDirectory()) {
        throw Error(serverMessage('authoring-isolated-workspace-required'));
      }
      return path.resolve(resolved, ...suffix);
    }
  };
  const repository = canonicalRoot(authoring.repository);
  const workspace = canonicalRoot(authoring.workspace);
  if (workspace === repository || workspace.startsWith(repository + path.sep)
      || repository.startsWith(workspace + path.sep)) {
    throw Error(serverMessage('authoring-isolated-workspace-required'));
  }
  const sourceSession = installed.sourceSession;
  let completion = null;
  let identity = null;
  let executed = false;
  return Object.freeze({
    async execute() {
      if (executed || host() !== installed) {
        throw Error(serverMessage('authoring-session-incomplete'));
      }
      executed = true;
      const result = await sourceSession.run({request, workspace,
        tools: ['read', 'write', 'bash']}, async () => {
        const dialogue = await drive(planChatStep, workspace, request, {
          tools: ['read', 'write', 'bash'], steps: 12,
          allowedCommands: [frame.request.command],
        });
        completion = sourceSession.completedDisposition();
        if (!completion || host() !== installed) {
          throw Error(serverMessage('authoring-session-incomplete'));
        }
        identity = hash(fs.readFileSync(path.resolve(workspace, produced[0])));
        return dialogue;
      });
      const sessionIdentifier = 'ses_' + randomUUID();
      return {exit_code: 0, timed_out: false, missing: false, stderr: '',
        stdout: JSON.stringify({session_id: sessionIdentifier, result}) + '\n'};
    },
    verify() {
      if (!completion || completion.verified !== true || identity === null
          || hash(fs.readFileSync(path.resolve(workspace, produced[0]))) !== identity) {
        throw Error(serverMessage('authoring-artifact-content-unobserved'));
      }
    },
  });
}
