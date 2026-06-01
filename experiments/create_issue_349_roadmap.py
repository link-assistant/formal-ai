#!/usr/bin/env python3
"""Create the issue #349 roadmap as dependency-linked GitHub issues.

Single source of truth is docs/case-studies/issue-349/ROADMAP.md: each
`## #N — Title` section becomes one GitHub issue. Cross-references to logical
ids #1..#11 are tokenised, the issues are created (capturing real numbers),
then the tokens are substituted and the dependency edges wired via the GitHub
issue *dependencies* API (`POST .../dependencies/blocked_by`, body
`{"issue_id": <database id>}`).

Usage:
  python3 experiments/create_issue_349_roadmap.py --dry-run   # parse + render, create nothing
  python3 experiments/create_issue_349_roadmap.py --create    # create issues, save map, fix bodies, wire deps
  python3 experiments/create_issue_349_roadmap.py --wire       # (re)wire deps from saved map only
"""
import json
import os
import re
import subprocess
import sys

REPO = "link-assistant/formal-ai"
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROADMAP = os.path.join(ROOT, "docs/case-studies/issue-349/ROADMAP.md")
MAPFILE = os.path.join(ROOT, "experiments/issue-349-issue-map.json")
BODYDIR = "/tmp/issue-bodies"
BRANCH = "issue-349-4243100887a0"
CS_URL = f"https://github.com/{REPO}/blob/{BRANCH}/docs/case-studies/issue-349/README.md"
RM_URL = f"https://github.com/{REPO}/blob/{BRANCH}/docs/case-studies/issue-349/ROADMAP.md"

# logical id -> labels
LABELS = {
    1: ["bug"],
    2: ["documentation", "enhancement"],
    3: ["enhancement", "bug"],
    4: ["enhancement", "bug"],
    5: ["enhancement"],
    6: ["enhancement"],
    7: ["bug", "enhancement"],
    8: ["enhancement"],
    9: ["enhancement"],
    10: ["enhancement"],
    11: ["documentation", "enhancement"],
}
# logical id -> blockers (logical ids)
DEPS = {
    1: [], 2: [1], 3: [1], 4: [2, 3], 5: [2, 4], 6: [1],
    7: [4, 5], 8: [5, 6, 7], 9: [5, 7], 10: [8, 9],
    11: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
}


def parse_roadmap():
    text = open(ROADMAP, encoding="utf-8").read()
    parts = re.split(r"(?m)^## #(\d+) — (.+)$", text)
    sections = {}
    for i in range(1, len(parts), 3):
        num = int(parts[i])
        title = parts[i + 1].strip()
        body = parts[i + 2].strip()
        sections[num] = (title, body)
    return sections


def tokenize(body):
    """Replace logical issue refs #1..#11 with @@ISSUE_n@@; leave #341/#349/#350 etc."""
    def repl(m):
        n = int(m.group(1))
        return f"@@ISSUE_{n}@@" if 1 <= n <= 11 else m.group(0)
    return re.sub(r"#(\d+)", repl, body)


def make_body(num, sections):
    title, body = sections[num]
    blockers = DEPS[num]
    blk = ", ".join(f"@@ISSUE_{b}@@" for b in blockers) if blockers else "_none_"
    header = (
        f"> 🤖 Auto-created from the [issue #349 roadmap]({RM_URL}) "
        f"(logical step **{num} of 11**). "
        f"Deep root-cause analysis: [case study]({CS_URL}). Tracked in PR #350.\n"
        f">\n"
        f"> **Blocked by:** {blk}  (wired via the GitHub issue dependencies API)\n\n"
    )
    return header + tokenize(body)


def substitute(text, mapping):
    def repl(m):
        n = m.group(1)
        return f"#{mapping[n]}"
    return re.sub(r"@@ISSUE_(\d+)@@", repl, text)


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, text=True, capture_output=True, **kw)


def create_all(sections):
    os.makedirs(BODYDIR, exist_ok=True)
    mapping = {}
    for num in range(1, 12):
        title, _ = sections[num]
        bf = os.path.join(BODYDIR, f"create-{num}.md")
        open(bf, "w", encoding="utf-8").write(make_body(num, sections))
        cmd = ["gh", "issue", "create", "--repo", REPO, "--title", title, "--body-file", bf]
        for lab in LABELS[num]:
            cmd += ["--label", lab]
        out = run(cmd).stdout.strip()
        m = re.search(r"/issues/(\d+)", out)
        gh_num = m.group(1)
        mapping[str(num)] = gh_num
        print(f"  logical #{num:>2} -> GitHub #{gh_num}  {out}")
        json.dump(mapping, open(MAPFILE, "w"), indent=2)
    return mapping


def fix_bodies(sections, mapping):
    for num in range(1, 12):
        body = substitute(make_body(num, sections), mapping)
        bf = os.path.join(BODYDIR, f"final-{num}.md")
        open(bf, "w", encoding="utf-8").write(body)
        run(["gh", "issue", "edit", mapping[str(num)], "--repo", REPO, "--body-file", bf])
        print(f"  fixed body of GitHub #{mapping[str(num)]} (logical #{num})")


def db_id(number):
    return int(run(["gh", "api", f"repos/{REPO}/issues/{number}", "--jq", ".id"]).stdout.strip())


def wire(mapping):
    for num in range(1, 12):
        blocked = mapping[str(num)]
        for b in DEPS[num]:
            blocker = mapping[str(b)]
            bid = db_id(blocker)
            payload = json.dumps({"issue_id": bid})
            p = subprocess.run(
                ["gh", "api", "-X", "POST",
                 f"repos/{REPO}/issues/{blocked}/dependencies/blocked_by", "--input", "-"],
                input=payload, text=True, capture_output=True,
            )
            ok = p.returncode == 0
            note = "ok" if ok else p.stderr.strip().replace("\n", " ")[:160]
            print(f"  #{blocked} blocked_by #{blocker} (id {bid}): {note}")


def verify(mapping):
    print("\n=== verification: blocked_by per issue ===")
    for num in range(1, 12):
        n = mapping[str(num)]
        out = run(["gh", "api", f"repos/{REPO}/issues/{n}/dependencies/blocked_by",
                   "--jq", "[.[].number]|sort"]).stdout.strip()
        want = sorted(int(mapping[str(b)]) for b in DEPS[num])
        print(f"  GitHub #{n} (logical #{num}): blocked_by={out}  expected={want}")


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "--dry-run"
    sections = parse_roadmap()
    missing = [n for n in range(1, 12) if n not in sections]
    if missing:
        sys.exit(f"ROADMAP parse error: missing logical issues {missing}")
    if mode == "--dry-run":
        print(f"Parsed {len(sections)} issues from ROADMAP.md\n")
        for num in range(1, 12):
            title, _ = sections[num]
            body = make_body(num, sections)
            print(f"--- logical #{num}: {title}")
            print(f"    labels={LABELS[num]} blocked_by(logical)={DEPS[num]} body_chars={len(body)}")
            print(f"    first line: {body.splitlines()[2][:100]}")
        return
    if mode == "--create":
        print("Creating issues...")
        mapping = create_all(sections)
        print("\nFixing cross-reference bodies...")
        fix_bodies(sections, mapping)
        print("\nWiring dependencies...")
        wire(mapping)
        verify(mapping)
        print(f"\nMap saved to {MAPFILE}")
        return
    if mode == "--wire":
        mapping = json.load(open(MAPFILE))
        fix_bodies(sections, mapping)
        wire(mapping)
        verify(mapping)
        return
    sys.exit(f"unknown mode {mode}")


if __name__ == "__main__":
    main()
