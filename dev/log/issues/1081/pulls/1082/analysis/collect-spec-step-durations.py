#!/usr/bin/env python3
"""Measure the macOS specification shard's budgeted step across recent CI/CD runs.

Evidence for issue #1081: the step failed at 100% of its 1400s budget on
`main` (run 34095902681). This re-derives the trend from the Actions API so the
failure can be read as a trend rather than as one unlucky run.

Usage: python3 collect-spec-step-durations.py [RUNS] > spec-step-durations.tsv
"""
import json, subprocess, sys, datetime

REPO = "link-assistant/formal-ai"
WANT_JOB = "Test (macos-15-intel / specification)"
LIMIT = int(sys.argv[1]) if len(sys.argv) > 1 else 60

def api(path):
    out = subprocess.run(["gh", "api", path], capture_output=True, text=True)
    if out.returncode != 0:
        return None
    return json.loads(out.stdout)

def ts(v):
    return datetime.datetime.fromisoformat(v.replace("Z", "+00:00"))

runs = []
page = 1
while len(runs) < LIMIT:
    d = api(f"repos/{REPO}/actions/runs?branch=main&per_page=100&page={page}")
    if not d or not d["workflow_runs"]:
        break
    runs += [r for r in d["workflow_runs"] if r["name"] == "CI/CD Pipeline"]
    page += 1
    if page > 8:
        break
runs = runs[:LIMIT]

print("run_id\tcreated_at\thead_sha\tjob_conclusion\tstep\tstep_conclusion\tseconds", flush=True)
for r in runs:
    d = api(f"repos/{REPO}/actions/runs/{r['id']}/jobs?per_page=100&filter=latest")
    if not d:
        continue
    for j in d["jobs"]:
        if j["name"] != WANT_JOB:
            continue
        for s in j.get("steps", []):
            if s["conclusion"] in (None, "skipped") or not s.get("started_at"):
                continue
            if s["name"] not in ("Run specification tests", "Run tests"):
                continue
            secs = (ts(s["completed_at"]) - ts(s["started_at"])).total_seconds()
            print(f"{r['id']}\t{r['created_at']}\t{r['head_sha'][:8]}\t{j['conclusion']}\t{s['name']}\t{s['conclusion']}\t{secs:.0f}", flush=True)
