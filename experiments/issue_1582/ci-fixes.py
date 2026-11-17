"""Refresh generated roles and record a graph-checked upstream advisory limit."""
from pathlib import Path

p = Path('.cargo/audit.toml')
s = p.read_text()
assert 'RUSTSEC-2026-0543' not in s
s = s.replace('[advisories]', '''# RUSTSEC-2026-0543 blocked-upstream = "chrono@0.4.45" report = "https://github.com/link-foundation/command-stream/issues/238"
#   chrono is an unconditional dependency of command-stream 1.5.3 and
#   link-calculator 0.22.0. The latest published versions (1.13.0 and 0.24.0)
#   still require it. RustSec marks all chrono versions unmaintained and lists
#   no patched release; upgrading chrono cannot resolve this notice.
#   Both introducing projects have reproducible migration reports:
#   https://github.com/link-foundation/command-stream/issues/238
#   https://github.com/link-assistant/calculator/issues/265
#   This acknowledges only the informational maintenance notice. All other
#   warnings remain fatal. The existing cargo-tree proof rejects this entry
#   once chrono leaves the graph, requiring removal of the stale exception.
[advisories]''')
s = s.replace('ignore = ["RUSTSEC-2025-0057"]', 'ignore = ["RUSTSEC-2025-0057", "RUSTSEC-2026-0543"]')
p.write_text(s)

p = Path('docs/case-studies/issue-1582/README.md')
s = p.read_text()
s += '''
## First repair-head CI findings

The first pushed repair is `c8de9bf3c2b404787357986bbefdb46948d21a1d`.
GitHub's combined PR diff endpoint rejects its file count (845 files); the
paginated file API is archived and its blob identities are checked against the
local tree. Most paths are the requested retained evidence.

The [CI/CD run](https://github.com/link-assistant/formal-ai/actions/runs/44096924617)
reports a stale reserved-role registry at log lines 750–752. The maintained role
generator refreshes the new permission role and its package mirror. This gate
is in the hosted fast-governance check, beyond the 138-entry local inventory.

The [Security run](https://github.com/link-assistant/formal-ai/actions/runs/44096923794)
fails at lines 741–748 because `chrono 0.4.45` is unmaintained under
RUSTSEC-2026-0543. It is introduced by command-stream and link-calculator;
their latest published versions still depend on chrono. Primary advisory and
crate metadata are retained. Reproductions, workarounds and migration proposals
are reported in [command-stream #238](https://github.com/link-foundation/command-stream/issues/238)
and [calculator #265](https://github.com/link-assistant/calculator/issues/265).
The existing audit mechanism can acknowledge this exact informational notice
with a live dependency-graph proof; it rejects the exception after chrono leaves
the graph. Other vulnerabilities, unsoundness, yanked crates and warnings stay
fatal. This is a recorded upstream maintenance limit, not a claim that chrono
has been repaired. The audit and native compilation execute on CI.
'''
p.write_text(s)
