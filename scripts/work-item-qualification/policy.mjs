import { createHash } from 'node:crypto';
import { taskNames } from './fixtures.mjs';
export const workItemRoots = ['.github/workflows/work-item-qualification.yml', 'scripts/work-item-qualification/prepare-work-item-qualification.mjs', 'scripts/work-item-qualification/run-work-item-qualification.mjs', 'scripts/work-item-qualification/transport-client.mjs', 'scripts/work-item-qualification/push-policy.mjs', 'scripts/work-item-qualification/transport-broker.mjs', 'scripts/work-item-qualification/independent-oracle.mjs', 'scripts/work-item-qualification/manifest-claims.mjs', 'scripts/work-item-qualification/manifest-observations.py', 'scripts/work-item-qualification/trace-oracle.mjs', 'scripts/work-item-qualification/observe-writes.py', 'scripts/work-item-qualification/fixtures.mjs', 'scripts/work-item-qualification/original-task.json', 'scripts/work-item-qualification/cleanup-contract.mjs', 'scripts/work-item-qualification/workspace-snapshot.mjs', 'scripts/work-item-qualification/command-capture.mjs', 'scripts/work-item-qualification/policy.mjs'];
const sha = value => createHash('sha256').update(value).digest('hex');
export function suiteContract() {
  return {
    schema: 'formal-ai-work-item-suite/v1',
    clients: ['agent', 'claude', 'codex'],
    model: 'formal-ai',
    tasks: taskNames,
    remainingAcceptance: ['a genuine production logic repair selected before execution', 'a useful regression-test task selected before execution'],
    originalSnapshot: '4c18fa47059e19963601a35d749db5c493d4245f',
    taskInput: 'ordinary issue prose, no final patch or grader answers',
    transport: 'frozen source mirror plus bounded GitHub broker; solver network disabled',
    allIssueAcceptanceComplete: false
  };
}
export async function workItemIdentity({
  read
}) {
  const roots = [];
  for (const name of workItemRoots) roots.push({
    path: name,
    sha256: sha(await read(name))
  });
  const suite = suiteContract();
  return {
    policyHash: sha(JSON.stringify(roots)),
    inputHash: sha(JSON.stringify(suite)),
    suite
  };
}
