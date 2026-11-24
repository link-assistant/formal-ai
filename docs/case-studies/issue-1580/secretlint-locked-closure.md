# Locked default scanner closure

The maintained scanner pinned Secretlint 14.0.0, while npm still resolved its
transitive dependency ranges at each installation. The Rust template comparison
(row 22) requires a dedicated dependency lock. This repair gives the existing
default scanner an isolated private manifest and npm lock without changing its
rules, masking, archive handling or filename selection.

## Reproduction and repair

Run the five cases in
[the focused regression](../../../experiments/issue-1580/secretlint-locked-install.test.mjs)
with the previous shell entry point: three behavioral cases fail because the
default uses an unlocked npm package request. The override and lock-identity
controls pass. With the repair all five pass, including literal NUL filenames,
failed-install propagation, finding propagation and removal of temporary files.

The default now copies the manifest and lock into a temporary directory and uses
npm ci --ignore-scripts --no-audit --no-fund --engine-strict. Installation reads
from /dev/null, preserving scanner stdin. The existing masked engine resolves
its binaries through that installation. The EXIT trap removes it on both success
and failure. An explicit nondefault version retains its existing separate npm
request and earns no default-lock qualification.

The committed lock contains 29 installed packages, with exact npm registry URLs
and SHA-512 integrity digests. The private root requires Node 22 or later;
.github/workflows/release.yml already selects Node 22 for the scanner.
Regenerate the lock deliberately together with its manifest and shell version.

## Verification and limits

- The maintained boundary suite passes all 44 cases, including the five imported
  regressions. The fast-profile suite passes all 29 cases. Shell syntax and
  whitespace checks and eleven affected JavaScript gates pass.
- The actual locked npm installation and scanner pass clean plain/gzip inputs,
  masked synthetic findings in both formats, and corrupt-gzip failure. Every
  invocation removes its scanner installation; findings and corrupt input never
  print a clean result.
- An offline npm ci control rejects an intentionally inconsistent Secretlint
  lock version with EUSAGE before installation. npm audit --package-lock-only
  reports zero vulnerabilities for this observed lock.
- Actual local execution uses Node 26.13.0 and npm 11.20.0. Hosted Node 22
  execution of this local repair remains pending.

An initial mock fixture escaped its recorder newline incorrectly; that failed
draft is retained. A subsequent shell draft used a function-local temporary
directory whose value was unavailable at EXIT on failure; removing that local
scope fixes both cleanup regressions. The first actual probe counted Node's own
compile cache as a leaked installation. A physical control identifies that cache;
disabling Node's compile cache leaves no temporary entries in any of the five
actual scanner cases. Those initial observations remain evidence, not passes.

The complete observations and source snapshots are sealed in
[the locked-closure receipt](../../../experiments/issue-1580/evidence/checks/secretlint-locked-closure-20261123/manifest.json).
The [post-commit checks](../../../experiments/issue-1580/evidence/checks/secretlint-locked-final-checks-20261123/manifest.json)
include the passing actual secret scan of all local commits since bc08222b5.
Archive readback checks every indexed member and digest. This verifies the local
scanner repair; it does not complete issue #1580, hosted qualification, independent
authorship, or the remaining template-practice comparison.
