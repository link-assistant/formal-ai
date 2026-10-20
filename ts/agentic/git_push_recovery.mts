// Re-read the checked-out branch for an observed local refspec failure.
import { planOne, jsonText, finalAnswer } from './plan.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { Progress } from './progress.mjs';
import { requestTargetBinding, validBranchTarget } from './git_commit.mjs';
export const CURRENT_BRANCH_PUSH = 'branch=$(git branch --show-current) && test -n "$branch" && git check-ref-format --branch "$branch" >/dev/null && git push -q origin "HEAD:refs/heads/$branch" && git rev-parse HEAD';
export function gitPushRecovery(messages, tools, failure) {
  if (!/src refspec[^\n]*does not match any/u.test(failure.reported)
    || !/\bgit\s+push\b/u.test(failure.failed_command ?? '')) return null;
  if (!/\bgit\s+push\s+(?:(?:-q|--quiet)\s+)?origin\s+/u.test(failure.failed_command ?? '')) return finalAnswer('The push failed with an invalid local refspec. Inspect the current branch and the originally requested remote before retrying; no alternative remote is authorized.');
  if (messages.some(message => message.role === 'user' && requestTargetBinding(typeof message.content === 'string' ? message.content : '', 'branch-binding-patterns', validBranchTarget).kind !== 'absent')) return finalAnswer('The refspec failed while an explicit branch binding exists. Inspect that local ref and preserve the requested destination; the current branch was not pushed automatically.');
  const progress = Progress.scan(messages);
  if (progress.hasRun(CURRENT_BRANCH_PUSH)) return finalAnswer('The current-branch push recovery was attempted. Inspect its Git receipt; any remaining task requirements are incomplete.');
  const tool = toolFor(tools, Capability.Run);
  return tool ? planOne(tool, jsonText({command: CURRENT_BRANCH_PUSH})) : finalAnswer('The push failed with an invalid local refspec. Read git branch --show-current in the client workspace before retrying; no shell tool is available.');
}
