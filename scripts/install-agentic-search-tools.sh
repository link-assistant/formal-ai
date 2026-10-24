#!/usr/bin/env bash
# Provision the real search adapter for JavaScript agentic checks on Debian CI runners.
set -euo pipefail
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
if ! command -v rg >/dev/null 2>&1; then
  echo 'Provisioning ripgrep with bounded APT attempts.'
  FORMAL_AI_APT_ATTEMPTS="${FORMAL_AI_APT_ATTEMPTS:-2}" \
  FORMAL_AI_APT_ATTEMPT_SECONDS="${FORMAL_AI_APT_ATTEMPT_SECONDS:-60}" \
  FORMAL_AI_APT_RETRY_DELAY_SECONDS="${FORMAL_AI_APT_RETRY_DELAY_SECONDS:-5}" \
  FORMAL_AI_APT_OFFICIAL_MIRROR_FALLBACK="${FORMAL_AI_APT_OFFICIAL_MIRROR_FALLBACK:-1}" \
  TEST_BUDGET_SECONDS="${TEST_BUDGET_SECONDS:-150}" \
    bash "$script_directory/apt-install-with-retry.sh" --no-install-recommends ripgrep
fi
bash "$script_directory/run-with-deadline.sh" 10 rg --version
