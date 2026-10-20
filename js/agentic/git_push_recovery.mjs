// Re-read the checked-out branch for an observed local refspec failure.
import { planOne, jsonText, finalAnswer } from './plan.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { Progress } from './progress.mjs';
import { fill } from './work_item_steps.mjs';
import { requestTargetBinding, validBranchTarget } from './git_commit.mjs';
export const CURRENT_BRANCH_PUSH = 'branch=$(git branch --show-current) && test -n "$branch" && git check-ref-format --branch "$branch" >/dev/null && git push -q origin "HEAD:refs/heads/$branch" && git rev-parse HEAD';
export function gitPushRecovery(messages, tools, failure) {
  const markers = JSON.parse(fill('refspec-failure-markers', []));
  const command = failure.failed_command ?? '';
  if (!markers.every(marker => failure.reported.includes(marker))
    || !new RegExp(fill('git-push-command-pattern', []), 'u').test(command)) return null;
  if (!new RegExp(fill('origin-push-command-pattern', []), 'u').test(command)) return finalAnswer('The push failed with an invalid local refspec. Inspect the current branch and the originally requested remote before retrying; no alternative remote is authorized.');
  if (messages.some(message => message.role === 'user' && requestTargetBinding(typeof message.content === 'string' ? message.content : '', 'branch-binding-patterns', validBranchTarget).kind !== 'absent')) return finalAnswer('The refspec failed while an explicit branch binding exists. Inspect that local ref and preserve the requested destination; the current branch was not pushed automatically.');
  const progress = Progress.scan(messages);
  if (progress.hasRun(CURRENT_BRANCH_PUSH)) return finalAnswer('The current-branch push recovery was attempted. Inspect its Git receipt; any remaining task requirements are incomplete.');
  const tool = toolFor(tools, Capability.Run);
  return tool ? planOne(tool, jsonText({command: CURRENT_BRANCH_PUSH})) : finalAnswer('The push failed with an invalid local refspec. Read git branch --show-current in the client workspace before retrying; no shell tool is available.');
}
