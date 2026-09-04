#!/usr/bin/env bash
# Issue #921: drive a real Hive Mind `solve` against a formal-ai server and
# assert the full circle closes -- the gate for hive-mind#2158's vision.
#
# The CI caller pins `@link-assistant/hive-mind@2.15.1`, the current release
# carrying hive-mind#2159's boundary fix. This gate runs against that boundary,
# so the pin may not fall back below the fixed release.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN="${BIN:-${ROOT}/target/release/formal-ai}"
OUT="${1:-${OUT:-/tmp/formal-ai-issue-921-evidence}}"
PORT="${PORT:-8921}"
ISSUE_URL="https://github.com/link-assistant/formal-ai/issues/921"
RUNNER="${ROOT}/experiments/issue_921_hive_mind_full_circle/run-hive-executor.mjs"
FAILURE_AGENT="${ROOT}/experiments/issue_921_hive_mind_full_circle/failure-agent.sh"
GH_PREPARE_WRAPPER="${ROOT}/experiments/issue_921_hive_mind_full_circle/github-readonly-prepare-wrapper.sh"

for program in agent curl gh git node solve; do
  command -v "${program}" >/dev/null || {
    echo "missing required program: ${program}" >&2
    exit 2
  }
done
test -x "${BIN}" || {
  echo "formal-ai binary is not executable: ${BIN}" >&2
  exit 2
}

if [[ -d "${OUT}" ]] && [[ -n "$(find "${OUT}" -mindepth 1 -print -quit)" ]]; then
  echo "evidence output must be absent or empty: ${OUT}" >&2
  exit 2
fi

HIVE_MIND_ROOT="${HIVE_MIND_ROOT:-$(cd "$(dirname "$(readlink -f "$(command -v solve)")")/.." && pwd)}"
test -f "${HIVE_MIND_ROOT}/src/agent.lib.mjs" || {
  echo "Hive Mind source package not found under ${HIVE_MIND_ROOT}" >&2
  exit 2
}

SCRATCH="$(mktemp -d /tmp/formal-ai-issue-921.XXXXXX)"
REAL_GH="$(command -v gh)"
SERVER_PID=""
export NPM_CONFIG_PREFIX="${SCRATCH}/npm-global"
export PATH="${NPM_CONFIG_PREFIX}/bin:${PATH}"
mkdir -p "${NPM_CONFIG_PREFIX}"
cleanup() {
  if [[ -n "${SERVER_PID}" ]]; then
    kill "${SERVER_PID}" 2>/dev/null || true
    wait "${SERVER_PID}" 2>/dev/null || true
  fi
  rm -rf "${SCRATCH}"
}
trap cleanup EXIT

mkdir -p \
  "${OUT}/hive-mind-to-formal-ai" \
  "${OUT}/formal-ai-to-hive-mind" \
  "${OUT}/raw-logs"

FORMAL_AI_MEMORY_PATH="${SCRATCH}/memory.lino" \
FORMAL_AI_DREAMING=0 \
FORMAL_AI_TRACE_REQUESTS=1 \
"${BIN}" serve --agent-mode --host 127.0.0.1 --port "${PORT}" \
  >"${OUT}/raw-logs/formal-ai-server.log" 2>&1 &
SERVER_PID=$!
for _ in $(seq 1 100); do
  if curl -fsS "http://127.0.0.1:${PORT}/api/openai/v1/models" >/dev/null; then
    break
  fi
  if ! kill -0 "${SERVER_PID}" 2>/dev/null; then
    echo "formal-ai server exited before becoming ready" >&2
    exit 1
  fi
  sleep 0.1
done
curl -fsS "http://127.0.0.1:${PORT}/api/openai/v1/models" >/dev/null
export HIVE_MIND_FORMAL_AI_BASE_URL="http://127.0.0.1:${PORT}"
export HIVE_MIND_FORMAL_AI_PATH="${BIN}"

commit_fixture() {
  local workspace="$1"
  local message="$2"
  GIT_AUTHOR_NAME="Issue 921 Replay" \
  GIT_AUTHOR_EMAIL="issue-921@example.invalid" \
  GIT_AUTHOR_DATE="2026-08-13T21:00:00Z" \
  GIT_COMMITTER_NAME="Issue 921 Replay" \
  GIT_COMMITTER_EMAIL="issue-921@example.invalid" \
  GIT_COMMITTER_DATE="2026-08-13T21:00:00Z" \
    git -C "${workspace}" commit --quiet -m "${message}"
}

init_fixture() {
  local workspace="$1"
  mkdir -p "${workspace}"
  git -C "${workspace}" init --quiet --initial-branch main
  git -C "${workspace}" config user.name "Issue 921 Replay"
  git -C "${workspace}" config user.email "issue-921@example.invalid"
  printf '%s\n' '.formal-ai-orchestration/' >"${workspace}/.gitignore"
  git -C "${workspace}" add .gitignore
  commit_fixture "${workspace}" "fixture: initialize replay workspace"
}

HIVE_OUT="${OUT}/hive-mind-to-formal-ai"
HIVE_WORKSPACE="${SCRATCH}/hive-mind-workspace"
init_fixture "${HIVE_WORKSPACE}"
printf '%s\n' \
  'Create file hive-mind-to-formal-ai.txt containing hive mind drove agent cli through formal ai' \
  >"${HIVE_OUT}/task.txt"
printf '%s' 'hive mind drove agent cli through formal ai' >"${HIVE_OUT}/expected.txt"

node "${RUNNER}" "${HIVE_MIND_ROOT}" "${HIVE_WORKSPACE}" \
  "${HIVE_OUT}/task.txt" "$(command -v agent)" \
  >"${OUT}/raw-logs/hive-executor.log" 2>&1
cmp -s "${HIVE_WORKSPACE}/hive-mind-to-formal-ai.txt" "${HIVE_OUT}/expected.txt"
cp "${HIVE_WORKSPACE}/hive-mind-to-formal-ai.txt" "${HIVE_OUT}/result.txt"
git -C "${HIVE_WORKSPACE}" add hive-mind-to-formal-ai.txt
commit_fixture "${HIVE_WORKSPACE}" "test: record Hive Mind workspace effect"
git -C "${HIVE_WORKSPACE}" rev-parse HEAD >"${HIVE_OUT}/commit.txt"
git -C "${HIVE_WORKSPACE}" show --format=fuller --no-ext-diff HEAD \
  >"${HIVE_OUT}/workspace-effect.patch"
awk 'match($0, /ses_[[:alnum:]_-]+/) { print substr($0, RSTART, RLENGTH); exit }' \
  "${OUT}/raw-logs/hive-executor.log" >"${HIVE_OUT}/agent-session-id.txt"
test -s "${HIVE_OUT}/agent-session-id.txt"

printf '%s\n' \
  "solve ${ISSUE_URL} --tool agent --model formal-ai --attach-logs --verbose" \
  >"${HIVE_OUT}/invocation.txt"
git clone --quiet --local "${ROOT}" "${SCRATCH}/prepare-repository"
git -C "${SCRATCH}/prepare-repository" config user.name "Issue 921 Replay"
git -C "${SCRATCH}/prepare-repository" config user.email "issue-921@example.invalid"
git -C "${SCRATCH}/prepare-repository" checkout -B main HEAD >/dev/null
git -C "${SCRATCH}/prepare-repository" update-ref refs/remotes/origin/main HEAD
mkdir -p "${SCRATCH}/readonly-prepare-bin"
ln -s "${GH_PREPARE_WRAPPER}" "${SCRATCH}/readonly-prepare-bin/gh"
set +e
(
  cd "${OUT}/raw-logs"
  GIT_CONFIG_COUNT=2 \
    GIT_CONFIG_KEY_0=user.name \
    GIT_CONFIG_VALUE_0="Issue 921 Replay" \
    GIT_CONFIG_KEY_1=user.email \
    GIT_CONFIG_VALUE_1="issue-921@example.invalid" \
    PATH="${SCRATCH}/readonly-prepare-bin:${PATH}" REAL_GH="${REAL_GH}" \
    solve "${ISSUE_URL}" --tool agent --model formal-ai \
    --attach-logs --verbose \
    --only-prepare-command \
    --working-directory "${SCRATCH}/prepare-repository" \
    --skip-tool-connection-check \
    --no-auto-pull-request-creation \
    --no-auto-continue \
    --no-auto-accept-invite \
    --no-playwright-mcp \
    --disable-report-issue \
    --disable-issue-auto-creation-on-error
) >"${OUT}/raw-logs/hive-solve-prepare.log" 2>&1
PREPARE_STATUS=$?
set -e
test "${PREPARE_STATUS}" -eq 0
grep -F 'agent --model formalai/formal-ai --verbose)' \
  "${OUT}/raw-logs/hive-solve-prepare.log" >/dev/null
grep -F 'Command prepared; AI execution skipped.' \
  "${OUT}/raw-logs/hive-solve-prepare.log" >/dev/null
printf '%s\n' \
  'agent --model formalai/formal-ai --verbose' \
  'execution=skipped' \
  >"${HIVE_OUT}/prepared-command.txt"

HIVE_FAILURE_WORKSPACE="${SCRATCH}/hive-failure-workspace"
init_fixture "${HIVE_FAILURE_WORKSPACE}"
set +e
node "${RUNNER}" "${HIVE_MIND_ROOT}" "${HIVE_FAILURE_WORKSPACE}" \
  "${HIVE_OUT}/task.txt" "${FAILURE_AGENT}" \
  >"${OUT}/raw-logs/hive-failure.log" 2>&1
HIVE_FAILURE_STATUS=$?
set -e
test "${HIVE_FAILURE_STATUS}" -eq 23
test "$(git -C "${HIVE_FAILURE_WORKSPACE}" rev-list --count HEAD)" -eq 1
printf '%s\n' \
  'agent_exit=23' \
  "hive_exit=${HIVE_FAILURE_STATUS}" \
  'workspace_commit=absent' \
  >"${HIVE_OUT}/failure-propagation.txt"

ORCHESTRATOR_OUT="${OUT}/formal-ai-to-hive-mind"
ORCHESTRATOR_WORKSPACE="${SCRATCH}/formal-ai-workspace"
init_fixture "${ORCHESTRATOR_WORKSPACE}"
printf '%s\n' \
  '# Hive-mind-shaped issue fixture' \
  '' \
  "Source: ${ISSUE_URL}" \
  '' \
  'Problem: The issue runner needs a deterministic workspace effect.' \
  '' \
  'Acceptance criteria:' \
  '- Create formal-ai-to-hive-mind.txt.' \
  '- Its exact content is: formal ai dispatched a hive-mind-shaped issue' \
  '- Report failure if the file cannot be written.' \
  '' \
  'Dispatch payload:' \
  'Create file formal-ai-to-hive-mind.txt containing formal ai dispatched a hive-mind-shaped issue' \
  >"${ORCHESTRATOR_OUT}/task.md"
printf '%s' 'formal ai dispatched a hive-mind-shaped issue' \
  >"${ORCHESTRATOR_OUT}/expected.txt"
# shellcheck disable=SC2016 # Expanded by the container's inner shell.
VERIFY='["sh","-c","test \"$(cat formal-ai-to-hive-mind.txt)\" = \"formal ai dispatched a hive-mind-shaped issue\""]'
ORCHESTRATOR_TASK='Create file formal-ai-to-hive-mind.txt containing formal ai dispatched a hive-mind-shaped issue'
"${BIN}" agent run \
  --cli agent \
  --task "${ORCHESTRATOR_TASK}" \
  --workspace "${ORCHESTRATOR_WORKSPACE}" \
  --model formal-ai \
  --base-url "http://127.0.0.1:${PORT}" \
  --target formal-ai \
  --timeout-seconds 180 \
  --session "${ORCHESTRATOR_OUT}/orchestration-session.json" \
  --allow-command sh \
  --verify "${VERIFY}" \
  >"${OUT}/raw-logs/formal-ai-orchestrator.log" 2>&1
cmp -s "${ORCHESTRATOR_WORKSPACE}/formal-ai-to-hive-mind.txt" \
  "${ORCHESTRATOR_OUT}/expected.txt"
cp "${ORCHESTRATOR_WORKSPACE}/formal-ai-to-hive-mind.txt" \
  "${ORCHESTRATOR_OUT}/result.txt"
"${BIN}" agent replay "${ORCHESTRATOR_OUT}/orchestration-session.json" \
  >"${OUT}/raw-logs/formal-ai-replay.log"
printf '%s\n' \
  'schema=formal-ai-agent-session-v1' \
  'status=succeeded' \
  'event_chain=verified' \
  'workspace_effect=formal-ai-to-hive-mind.txt' \
  >"${ORCHESTRATOR_OUT}/replay.txt"
git -C "${ORCHESTRATOR_WORKSPACE}" add formal-ai-to-hive-mind.txt
commit_fixture "${ORCHESTRATOR_WORKSPACE}" "test: record Formal AI workspace effect"
git -C "${ORCHESTRATOR_WORKSPACE}" rev-parse HEAD >"${ORCHESTRATOR_OUT}/commit.txt"
git -C "${ORCHESTRATOR_WORKSPACE}" show --format=fuller --no-ext-diff HEAD \
  >"${ORCHESTRATOR_OUT}/workspace-effect.patch"

ORCHESTRATOR_FAILURE_WORKSPACE="${SCRATCH}/orchestrator-failure-workspace"
init_fixture "${ORCHESTRATOR_FAILURE_WORKSPACE}"
set +e
"${BIN}" agent run \
  --cli agent \
  --task 'Injected hive-mind-shaped failure' \
  --workspace "${ORCHESTRATOR_FAILURE_WORKSPACE}" \
  --target vendor \
  --timeout-seconds 30 \
  --session "${OUT}/raw-logs/formal-ai-failure-session.json" \
  --command '["sh","-c","exit 23 # {task}"]' \
  --allow-agent-command sh \
  >"${OUT}/raw-logs/formal-ai-failure.log" 2>&1
ORCHESTRATOR_FAILURE_STATUS=$?
set -e
test "${ORCHESTRATOR_FAILURE_STATUS}" -eq 1
grep -F '"status": "failed"' "${OUT}/raw-logs/formal-ai-failure-session.json" >/dev/null
grep -F '"exit_code": 23' "${OUT}/raw-logs/formal-ai-failure-session.json" >/dev/null
printf '%s\n' \
  'agent_exit=23' \
  "orchestrator_exit=${ORCHESTRATOR_FAILURE_STATUS}" \
  'session_status=failed' \
  >"${ORCHESTRATOR_OUT}/failure-propagation.txt"

printf '%s\n' \
  "formal-ai=$(${BIN} --version | head -n 1)" \
  "agent=$(agent --version | head -n 1)" \
  "hive-mind=$(solve --version | head -n 1)" \
  >"${OUT}/versions.txt"
echo "issue 921 full-circle gate passed: ${OUT}"
