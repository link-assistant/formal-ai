#!/usr/bin/env python3
"""How often is a dotted token in prose a file path, and how often is it a number?

`is_workspace_path` in `src/agentic_coding/structured_edit.rs` accepts any token
that splits on a final `.` into a non-empty stem and a 1-8 character
alphanumeric extension. `1.1.2.2.1` -- the recursive ladder's node id -- passes
every one of those conditions, so a leaf prompt that names both
`src/engine_responses.rs` and node `1.1.2.2.1` sends the planner to read the
node id as if it were a file.

This surveys the two populations that predicate has to separate, using the
repository's own committed bytes rather than an assumption:

  1. every tracked file name, to see whether a real path ever carries an
     all-digit extension;
  2. every dotted token appearing in committed Markdown prose, classified by
     whether the repository actually tracks a file by that name.

    python3 experiments/issue_1069_dotted_tokens/survey.py
"""

import re
import subprocess
from collections import Counter

TOKEN = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_./-]*")


def tracked():
    out = subprocess.run(["git", "ls-files"], capture_output=True, text=True,
                         check=True).stdout
    return [p for p in out.splitlines() if p]


def is_workspace_path(token):
    """A transcription of the committed Rust predicate."""
    if "." not in token:
        return False
    stem, _, extension = token.rpartition(".")
    return (
        stem != ""
        and not token.startswith("/")
        and ".." not in token.split("/")
        and 1 <= len(extension) <= 8
        and extension.isalnum()
        and all(c.isalnum() or c in "_-./" for c in token)
    )


def main():
    paths = tracked()
    names = {p.rsplit("/", 1)[-1] for p in paths}
    all_paths = set(paths)

    extensions = Counter(n.rsplit(".", 1)[1] for n in names if "." in n)
    numeric = {e: c for e, c in extensions.items() if e.isdigit()}
    print(f"tracked files: {len(paths)}")
    print(f"distinct extensions: {len(extensions)}")
    print(f"extensions that are all digits: {len(numeric)} {numeric or ''}")

    # Population 2: dotted tokens in committed prose.
    markdown = [p for p in paths if p.endswith(".md")]
    accepted = Counter()
    for path in markdown:
        try:
            text = open(path, encoding="utf-8", errors="ignore").read()
        except OSError:
            continue
        for match in TOKEN.finditer(text):
            token = match.group(0).rstrip(".")
            if is_workspace_path(token):
                accepted[token] += 1

    real = Counter()
    numeric_token = Counter()
    other = Counter()
    for token, count in accepted.items():
        if token in all_paths or token.rsplit("/", 1)[-1] in names:
            real[token] = count
        elif token.rsplit(".", 1)[1].isdigit():
            numeric_token[token] = count
        else:
            other[token] = count

    total = sum(accepted.values())
    print(f"\ndotted tokens the predicate accepts in committed Markdown: {total}"
          f" ({len(accepted)} distinct)")
    for label, bucket in (("name a tracked file", real),
                          ("end in an all-digit extension", numeric_token),
                          ("neither", other)):
        share = 0 if total == 0 else 100 * sum(bucket.values()) / total
        print(f"  {label:<32}{sum(bucket.values()):>8} ({share:5.2f}%)"
              f"  {len(bucket)} distinct")

    print("\nmost frequent all-digit-extension tokens accepted as paths:")
    for token, count in numeric_token.most_common(15):
        print(f"  {count:>6}  {token}")

    overlap = [t for t in numeric_token if t in all_paths or
               t.rsplit("/", 1)[-1] in names]
    print(f"\nall-digit-extension tokens that are in fact tracked files: "
          f"{len(overlap)} {overlap}")


if __name__ == "__main__":
    main()
