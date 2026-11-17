#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/collect-repair-artifacts.py
python3 experiments/issue_1582/collect-repair-artifacts.py --download
python3 experiments/issue_1582/refresh-upstream-status.py
