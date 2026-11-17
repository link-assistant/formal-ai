#!/usr/bin/env bash
# Explicit external bootstrap after recorded WorkerSourceIdentityMismatch.
# Install only the two JavaScript parser tools, without changing package files.
set -eu
npm install --prefix /tmp/issue-1582-js-tools --ignore-scripts --no-audit --no-fund @babel/parser@8.0.7 @babel/traverse@8.0.7 > /tmp/issue-1582-parser-install.log 2>&1 &
install_pid=$!
wait "$install_pid"
if [ ! -e node_modules ]; then ln -s /tmp/issue-1582-js-tools/node_modules node_modules; fi
node scripts/generate-worker-prerequisites.mjs --write
