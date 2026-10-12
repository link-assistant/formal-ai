#!/usr/bin/env bash
# Provision the real search adapter for JavaScript agentic checks on Debian CI runners.
set -euo pipefail
if ! command -v rg >/dev/null 2>&1; then
  sudo apt-get update -qq
  sudo apt-get install -y --no-install-recommends ripgrep
fi
rg --version
