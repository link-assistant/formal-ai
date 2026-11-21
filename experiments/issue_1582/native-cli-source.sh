#!/usr/bin/env bash
set -eu
exec > /tmp/issue-1582-native-cli-source.log 2>&1
sed -n '98,125p' rust/src/main.rs
sed -n '665,710p' rust/src/main.rs
rg -n 'run_suite\(' rust/tests/unit/specification/external_benchmarks.rs
sed -n '60,120p' rust/src/solver_handlers/web_requests.rs
sed -n '595,620p' rust/tests/unit/specification/external_benchmarks.rs
sed -n '650,670p' rust/tests/unit/specification/external_benchmarks.rs
sed -n '812,830p' rust/tests/unit/specification/external_benchmarks.rs
