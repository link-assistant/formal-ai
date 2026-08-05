#!/usr/bin/env bash
# Issue #908 reproduction: replay the reported run's shell envelopes through the
# planner and print the plan after each step. Run from the repository root.
#
#   experiments/issue_908_exit_code_verdict/repro.sh > after.txt
#
# Before the fix the second step ("Exit Code: 0", no output) ended the run with
# "The agentic CLI harness could not complete `main.py`" — see before.txt.
set -eu

cargo run --quiet --example issue_908_exit_code_verdict
