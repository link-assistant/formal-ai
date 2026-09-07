#!/usr/bin/env python3
"""Compare this repository's GitHub Actions surface against the five
`link-foundation/*-ai-driven-development-pipeline-template` repositories along
the dimensions issue #1081 cares about: how a failure is made to report as a
failure, how a deadline is owned, and what the checkout hands to later steps.

Every number below is counted from the checked-out files, never asserted.

Usage: python3 experiments/template-ci-comparison.py [--tpl-root /tmp/tpl]
"""
import argparse, glob, os, re, sys

DIMS = [
    ("workflow files",            lambda t: len(t["files"])),
    ("jobs",                      lambda t: t["jobs"]),
    ("jobs with timeout-minutes", lambda t: t["job_timeouts"]),
    ("steps with timeout-minutes",lambda t: t["step_timeouts"]),
    ("TEST_BUDGET_SECONDS uses",  lambda t: t["budgets"]),
    ("budget-wrapper invocations",lambda t: t["budget_wrapper"]),
    ("checkout steps",            lambda t: t["checkouts"]),
    ("  ... persist-credentials: false", lambda t: t["persist_false"]),
    ("actions pinned by SHA",     lambda t: t["sha_pinned"]),
    ("actions pinned by tag",     lambda t: t["tag_pinned"]),
    ("docker:// image references",lambda t: t["docker_images"]),
    ("  ... pinned by digest",    lambda t: t["docker_digest"]),
    ("zizmor invocations",        lambda t: t["zizmor"]),
    ("ships .github/zizmor.yml",  lambda t: "yes" if t["zizmor_cfg"] else "no"),
    ("ships .github/dependabot.yml", lambda t: "yes" if t["dependabot"] else "no"),
    ("terminal status job",       lambda t: "yes" if t["status_job"] else "no"),
    ("concurrency groups",        lambda t: t["concurrency"]),
    ("  ... cancel-in-progress: true", lambda t: t["cancel_true"]),
]


def survey(root):
    files = sorted(glob.glob(os.path.join(root, ".github/workflows/*.y*ml")))
    files += sorted(glob.glob(os.path.join(root, ".github/actions/*/action.y*ml")))
    body = ""
    for path in files:
        with open(path, encoding="utf-8", errors="replace") as handle:
            body += handle.read().replace("\r\n", "\n") + "\n"
    uses = re.findall(r"^\s*(?:-\s*)?uses:\s*(\S+)", body, re.M)
    external = [u for u in uses if not u.startswith("./")]
    docker = [u for u in external if u.startswith("docker://")]
    actions = [u for u in external if not u.startswith("docker://")]
    return {
        "files": [p for p in files if "/workflows/" in p],
        "jobs": len(re.findall(r"^  [A-Za-z0-9_-]+:\s*$", body, re.M)),
        "job_timeouts": len(re.findall(r"^    timeout-minutes:", body, re.M)),
        "step_timeouts": len(re.findall(r"^        timeout-minutes:", body, re.M)),
        "budgets": len(re.findall(r"TEST_BUDGET_SECONDS", body)),
        "budget_wrapper": len(re.findall(r"run-with-budget-warning\.sh", body)),
        "checkouts": len(re.findall(r"actions/checkout@", body)),
        "persist_false": len(re.findall(r"persist-credentials:\s*false", body)),
        "sha_pinned": sum(1 for u in actions if re.search(r"@[0-9a-f]{40}$", u)),
        "tag_pinned": sum(1 for u in actions if not re.search(r"@[0-9a-f]{40}$", u)),
        "docker_images": len(docker),
        "docker_digest": sum(1 for u in docker if "@sha256:" in u),
        "zizmor": len(re.findall(r"zizmor", body, re.I)),
        "zizmor_cfg": os.path.exists(os.path.join(root, ".github/zizmor.yml")),
        "dependabot": os.path.exists(os.path.join(root, ".github/dependabot.yml")),
        "status_job": bool(re.search(r"^  (pipeline-status|status):\s*$", body, re.M)),
        "concurrency": len(re.findall(r"^\s*concurrency:", body, re.M)),
        "cancel_true": len(re.findall(r"cancel-in-progress:\s*true", body)),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tpl-root", default="/tmp/tpl")
    parser.add_argument("--repo", default=".")
    args = parser.parse_args()

    repos = {"formal-ai": args.repo}
    for path in sorted(glob.glob(os.path.join(args.tpl_root, "*-template"))):
        repos[os.path.basename(path).replace("-template", "")] = path
    missing = [k for k, v in repos.items() if not os.path.isdir(os.path.join(v, ".github/workflows"))]
    if missing:
        sys.exit(f"no .github/workflows in: {', '.join(missing)}")

    tables = {k: survey(v) for k, v in repos.items()}
    width = max(len(label) for label, _ in DIMS) + 2
    header = "| " + "dimension".ljust(width) + "|" + "|".join(f" {k} " for k in repos) + "|"
    print(header)
    print("|" + "-" * (width + 1) + "|" + "|".join("-" * (len(k) + 2) for k in repos) + "|")
    for label, fn in DIMS:
        cells = "|".join(f" {fn(tables[k])} ".ljust(len(k) + 2) for k in repos)
        print("| " + label.ljust(width) + "|" + cells + "|")


if __name__ == "__main__":
    main()
