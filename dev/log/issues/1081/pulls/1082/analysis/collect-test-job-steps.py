#!/usr/bin/env python3
"""Per-step and per-job durations for the `test` matrix legs on `main`.

Evidence for issue #1081. The budgeted `Run specification tests` step failed at
100% of its 1400s budget (run 34095902681). Sizing its replacement needs three
numbers the repository has never recorded: how long the compile inside that step
takes, how long the tests themselves take, and how much of the job clock is
spent outside the budgeted step. This prints all of them.

Usage: collect-test-job-steps.py [RUNS] > test-job-steps.tsv
"""
import datetime
import json
import subprocess
import sys

REPO = "link-assistant/formal-ai"
WANT = ("Test (macos-15-intel / specification)", "Test (ubuntu-latest / full)")
LIMIT = int(sys.argv[1]) if len(sys.argv) > 1 else 40


def api(path):
    out = subprocess.run(["gh", "api", path], capture_output=True, text=True)
    return json.loads(out.stdout) if out.returncode == 0 else None


def ts(value):
    return datetime.datetime.fromisoformat(value.replace("Z", "+00:00"))


runs, page = [], 1
while len(runs) < LIMIT and page <= 8:
    data = api(f"repos/{REPO}/actions/runs?branch=main&per_page=100&page={page}")
    if not data or not data["workflow_runs"]:
        break
    runs += [r for r in data["workflow_runs"] if r["name"] == "CI/CD Pipeline"]
    page += 1
runs = runs[:LIMIT]

print("run_id\tcreated_at\tjob\tjob_conclusion\tjob_seconds\tstep\tstep_conclusion\tstep_seconds")
for run in runs:
    data = api(f"repos/{REPO}/actions/runs/{run['id']}/jobs?per_page=100&filter=latest")
    if not data:
        continue
    for job in data["jobs"]:
        if job["name"] not in WANT or not job.get("started_at") or not job.get("completed_at"):
            continue
        job_seconds = (ts(job["completed_at"]) - ts(job["started_at"])).total_seconds()
        for step in job.get("steps", []):
            if not step.get("started_at") or not step.get("completed_at"):
                continue
            seconds = (ts(step["completed_at"]) - ts(step["started_at"])).total_seconds()
            print(
                f"{run['id']}\t{run['created_at']}\t{job['name']}\t{job['conclusion']}"
                f"\t{job_seconds:.0f}\t{step['name']}\t{step['conclusion']}\t{seconds:.0f}",
                flush=True,
            )
