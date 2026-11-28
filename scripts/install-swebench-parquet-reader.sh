#!/usr/bin/env bash
set -euo pipefail

# Install the dataset reader. Formal AI acquires the evaluator separately.
python3 -m pip install --require-hashes --only-binary=:all: --force-reinstall \
  -r .github/benchmarks/swebench-requirements.txt
python3 -c 'import pyarrow.parquet'
