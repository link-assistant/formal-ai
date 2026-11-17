// Issue #1162 acceptance item 3: grade one Hello World pull request obligation
// by obligation. The hosted oracle (scripts/check-hive-original-delivery.mjs)
// proves the program's exact output by compiling it independently; this grader
// checks the rest of the delivery from a snapshot of the pull request, so the
// same function grades a live pull request, a released-version run and a
// historical replay (docs/case-studies/issue-1162/raw-data/test-repositories/).
//
// A snapshot is { pull_request: { title, body, isDraft, state, headRefName,
// checks: [{ name, conclusion }] }, files: [{ filename, status }] }.
// A task is { language, issueNumber, issueTitle?, workflowPath?,
// suggestedWorkflowName?, branchPattern?, stdout? } where workflowPath is a
// literal path the request requires (#1450's ci/test-hello-world.yml) and
// stdout is an independently observed program output.
import fs from 'node:fs';

const POLICY_PATH = new URL('../../data/benchmarks/hello-world-delivery-obligations.json', import.meta.url);
export const deliveryPolicy = JSON.parse(fs.readFileSync(POLICY_PATH, 'utf8'));

const escape = text => text.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&');
const present = files => files.filter(file => file.status !== 'removed');
const matchesAny = (patterns, path) => patterns.some(pattern => new RegExp(pattern, 'u').test(path));

function isWorkflow(path, policy) {
  return policy.workflowDirectories.some(directory => path.startsWith(directory))
    && policy.workflowExtensions.some(extension => path.endsWith(extension));
}

function result(id, passed, detail) {
  return { id, status: passed === null ? 'unobserved' : passed ? 'pass' : 'fail', detail };
}

// Exact output with an explicit newline rule: one line, the expected text,
// one LF. Returns null when no independent observation exists.
export function exactOutputObligation(stdout, policy = deliveryPolicy) {
  if (stdout === undefined || stdout === null) return result('exact-output', null, 'no independent run observed');
  const lines = stdout.split('\n');
  const detail = stdout === policy.expectedStdout ? 'exactly one line'
    : !stdout.endsWith('\n') ? 'missing final newline'
      : stdout.includes('\r') ? 'carriage return in output'
        : lines.length > 2 ? `${lines.length - 1} lines`
          : `different text ${JSON.stringify(stdout)}`;
  return result('exact-output', stdout === policy.expectedStdout, detail);
}

export function gradeHelloWorldDelivery(snapshot, task, policy = deliveryPolicy) {
  const files = present(snapshot.files ?? []);
  const paths = files.map(file => file.filename);
  const pull = snapshot.pull_request ?? {};
  const extension = policy.programExtensions[task.language];
  if (!extension) throw new Error(`unsupported language ${task.language}`);
  const programs = paths.filter(path => path.endsWith(extension));
  const foreign = Object.values(policy.programExtensions)
    .filter(other => other !== extension)
    .flatMap(other => paths.filter(path => path.endsWith(other)));
  const workflows = paths.filter(path => isWorkflow(path, policy));
  const workflowPathPassed = workflows.length === 1 && (task.workflowPath
    ? workflows[0] === task.workflowPath
    : workflows[0].startsWith('.github/workflows/')
      && (!task.suggestedWorkflowName || workflows[0].split('/').at(-1) === task.suggestedWorkflowName));
  const artifacts = paths.filter(path => matchesAny(policy.buildArtifactPatterns, path));
  const evidence = paths.filter(path => matchesAny(policy.sessionEvidencePatterns, path));
  const checks = pull.checks ?? [];
  const failing = checks.filter(check => !policy.passingCheckConclusions.includes(String(check.conclusion).toUpperCase()));
  const title = String(pull.title ?? '').trim();
  const quoted = /^(['"`]).*\1$/u.test(title);
  const body = String(pull.body ?? '');
  const keywords = policy.closingKeywords.map(escape).join('|');
  const issue = `(?:#${task.issueNumber}|https://github\\.com/[^/\\s]+/[^/\\s]+/issues/${task.issueNumber})(?!\\d)`;
  const closes = new RegExp(`(?:^|\\W)(?:${keywords}):?\\s+${issue}`, 'iu').test(body);
  const placeholder = policy.placeholderBodyMarkers.filter(marker => body.toLowerCase().includes(marker.toLowerCase()));
  const obligations = [
    result('program-file', programs.length === 1, programs.length === 1 ? programs[0] : `${programs.length} ${task.language} program files`),
    result('program-language', foreign.length === 0, foreign.length ? `other-language programs: ${foreign.join(', ')}` : 'only the requested language'),
    exactOutputObligation(task.stdout, policy),
    result('verify-script', paths.some(path => new RegExp(policy.verifyScriptPattern, 'u').test(path)), 'a verify script is part of the delivery'),
    result('workflow-count', workflows.length === 1, workflows.length === 1 ? workflows[0] : `${workflows.length} workflow files: ${workflows.join(', ') || 'none'}`),
    result('workflow-path', workflowPathPassed, task.workflowPath
      ? `required literal path ${task.workflowPath}`
      : task.suggestedWorkflowName ? `suggested name ${task.suggestedWorkflowName}` : 'a native .github/workflows/ file'),
    result('workflow-runs', checks.length > 0 && failing.length === 0, checks.length === 0 ? 'no check runs' : failing.length ? `${failing.length} of ${checks.length} check runs fail` : `${checks.length} check runs pass`),
    result('no-build-artifacts', artifacts.length === 0, artifacts.length ? `committed: ${artifacts.join(', ')}` : 'none committed'),
    result('no-session-evidence', evidence.length === 0, evidence.length ? `committed: ${evidence.join(', ')}` : 'none committed'),
    result('branch', !task.branchPattern || new RegExp(task.branchPattern, 'u').test(String(pull.headRefName ?? '')), String(pull.headRefName ?? '')),
    result('title', title.length > 0 && !quoted && (!task.issueTitle || title === task.issueTitle), quoted ? `quoted placeholder ${title}` : title),
    result('description', closes && placeholder.length === 0, !closes ? `no closing reference to issue ${task.issueNumber}` : placeholder.length ? `placeholder text: ${placeholder.join('; ')}` : 'closes the issue'),
    result('ready', pull.isDraft === false && String(pull.state).toUpperCase() === 'OPEN', pull.isDraft ? 'draft' : String(pull.state)),
  ];
  return {
    passed: obligations.every(obligation => obligation.status === 'pass'),
    failed: obligations.filter(obligation => obligation.status === 'fail').map(obligation => obligation.id),
    unobserved: obligations.filter(obligation => obligation.status === 'unobserved').map(obligation => obligation.id),
    obligations,
  };
}
