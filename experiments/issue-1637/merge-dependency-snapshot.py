"""Issue #1637: fill the entries the committed dependency snapshot cannot verify.

`check-dependencies-latest.rs --offline` exits 3 when a manifest dependency is
absent from data/meta/dependency-registry-snapshot.lino (or an action's latest
release commit is unresolved). A full `--refresh-snapshot` also advances every
other registry answer, turning the gate red on dozens of unrelated bumps. This
script copies only the records the old snapshot lacks (or lacks fields for)
from a freshly refreshed snapshot, leaving every existing answer unchanged.

usage: python3 experiments/issue-1637/merge-dependency-snapshot.py OLD FRESH OUT
"""
import sys


def records(text):
    head, recs, cur = [], {}, None
    order = []
    for line in text.splitlines(keepends=True):
        if line.startswith("registry_entry "):
            cur = line.split()[1]
            recs[cur] = [line]
            order.append(cur)
        elif cur is None:
            head.append(line)
        else:
            recs[cur].append(line)
    return head, recs, order


old_head, old, old_order = records(open(sys.argv[1], encoding="utf-8").read())
_, fresh, _ = records(open(sys.argv[2], encoding="utf-8").read())
for key, lines in fresh.items():
    if key not in old:
        old[key] = lines
        old_order.append(key)
    else:
        have = {l.split()[0] for l in old[key][1:] if l.strip()}
        extra = [l for l in lines[1:] if l.strip() and l.split()[0] not in have]
        # Only add evidence fields (release-commit, tag-source-url) for the same latest.
        if extra and old[key][1] == lines[1]:
            old[key].extend(extra)
        elif extra:
            print(f"skip {key}: latest differs ({old[key][1].strip()} vs {lines[1].strip()})")
added = sorted(k for k in fresh if k not in records(open(sys.argv[1], encoding='utf-8').read())[1])
with open(sys.argv[3], "w", encoding="utf-8") as out:
    out.writelines(old_head)
    # The writer groups by ecosystem (crates_io, npm, github_action) and sorts
    # names by bytes inside each group.
    ecosystems = list(dict.fromkeys(key.split("/", 1)[0] for key in old_order))
    for key in sorted(old_order, key=lambda k: (ecosystems.index(k.split("/", 1)[0]), k.encode())):
        out.writelines(old[key])
print("added:", " ".join(added))
