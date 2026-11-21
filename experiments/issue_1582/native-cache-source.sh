#!/usr/bin/env bash
set -eu
exec > /tmp/issue-1582-native-cache-source.log 2>&1
sed -n '1,105p' rust/src/meta_reasoner/integration.rs
sed -n '65,135p' rust/src/agentic_coding/formalization_recipe.rs
sed -n '1,82p' rust/src/solver_handlers/program_synthesis.rs
sed -n '300,330p' rust/src/solver_handlers/verifiable_task.rs
sed -n '665,710p' rust/src/anticipation.rs
sed -n '105,135p' rust/src/retrieval_method.rs
