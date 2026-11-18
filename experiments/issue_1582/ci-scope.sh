#!/usr/bin/env bash
set -eu
rg -n 'native-type-check|library-and-binaries|native check' scripts --glob '*.mjs' --glob '!*.test.mjs'
sed -n '1,150p' scripts/ci-clippy-profile.sh
sed -n '1,100p' rust/tests/integration/mod.rs
git diff HEAD -- rust/src/structured_log.rs rust/src/cli_logs.rs
