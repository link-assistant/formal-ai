from pathlib import Path
root=Path('/tmp/gh-issue-solver-1794900398048');out=Path('/tmp/issue-1580-research/php-upstream-reports');out.mkdir(exist_ok=False)
base='https://github.com/link-assistant/formal-ai/blob/ac9cd6a55e3ea7d8738e6122f623d7676666630d/'
test=base+'experiments/issue-1580/template-php-input-integrity.test.mjs'
source='https://github.com/link-foundation/php-ai-driven-development-pipeline-template/blob/b61e334703d93a15e08bf1a278e0df0a9d6e98a8/'
(out/'version.md').write_text(f'''The pull-request version guard can report success without a valid comparison.

Reproduced in [scripts/check-version-modification.php]({source}scripts/check-version-modification.php)
at b61e334703d93a15e08bf1a278e0df0a9d6e98a8. Its blob is unchanged from the
pinned 3a034b27a56067971ca74dd4c74c7707056b215d fixture.

`versionFromJson` converts invalid JSON, missing versions, null and numeric
versions to the same null. The final comparison only rejects a difference when
both values are non-null. In addition, failed `git fetch`/`git show` can be
treated as a newly introduced manifest, rather than unavailable base evidence.

Finite local Git controls execute the actual CLI, without Composer installation
or remote network calls. Removing the head version, invalid head/base JSON,
null/numeric head versions, adding a version to an existing manifest and a missing
base repository all return zero on the original. The unavailable-base case even
has a changed head version. Unchanged and genuinely new manifests return zero;
an ordinary changed version returns one, as expected. A stale remote-tracking
ref also lets a failed fetch be ignored.

The [maintained original/candidate tests]({test}) include all those controls:

```sh
node --test --test-concurrency=1 experiments/issue-1580/template-php-input-integrity.test.mjs
```

Run from Formal AI commit ac9cd6a55e3ea7d8738e6122f623d7676666630d with PHP and
Node installed. Each local child has a five-second boundary, bounded output and
a 32 MiB PHP allowance. The test deliberately asserts the original bad result,
then verifies a disposable candidate rejects it. No candidate is applied to this
template by that test.

The candidate distinguishes an object with an absent version from malformed or
wrong-type input, requires a successful fetch and resolved commit, and proves
base-file absence with checked ls-tree before permitting bootstrap. It compares
the two nullable versions strictly, so removal/addition cannot bypass admission.
Positive unchanged/new-file/changed-version behavior is preserved. Please retain
those observation-failure and bootstrap distinctions in the upstream repair.

Found during the full CI/source comparison for
https://github.com/link-assistant/formal-ai/pull/1581.
''')
(out/'notes.md').write_text(f'''Release-note truncation can emit invalid UTF-8 at its default byte limit.

Reproduced in [ReleaseNotes::capBytes]({source}scripts/src/ReleaseNotes.php)
at b61e334703d93a15e08bf1a278e0df0a9d6e98a8; the relevant blob is identical to
the pinned 3a034b27a56067971ca74dd4c74c7707056b215d source.

After substr cuts the body to its byte budget, the loop removes trailing UTF-8
continuation bytes. It leaves the leading byte. This corrupts a complete final
multibyte character as well as a partially cut one. The retained test places é
exactly at the full body budget under the default 120000-byte cap; the original
output fails strict UTF-8 decoding. Separate two-, three- and four-byte cases
cover every partial cut and a complete-character boundary. ASCII and short
untruncated Unicode remain positive controls.

There is also a smaller-cap case: when max is below the notice length, the
original returns the entire notice and exceeds the requested cap. Controls use
caps 0, 1, 7 and notice-length minus one.

The [15-case maintained original/candidate file]({test}) executes the pinned PHP
class directly without Composer installation:

```sh
node --test --test-concurrency=1 experiments/issue-1580/template-php-input-integrity.test.mjs
```

Run from Formal AI commit ac9cd6a55e3ea7d8738e6122f623d7676666630d. Each PHP child
has a five-second boundary, 32 MiB memory limit and bounded input/output. Original
invalid output is checked with a fatal UTF-8 decoder. A disposable candidate
trims to an actual valid boundary and caps an oversized ASCII fixture notice;
it preserves the existing ASCII/short Unicode controls. That candidate is not an
upstream patch and does not claim arbitrary Unicode notice/URL qualification.

Please preserve a valid UTF-8 boundary and the requested byte limit for both
body and notice. Existing ASCII-only ReleaseNotesTest coverage misses this case.
Found during https://github.com/link-assistant/formal-ai/pull/1581.
''')
(out/'budget-comment.md').write_text(f'''Additional bounded evidence at current source
b61e334703d93a15e08bf1a278e0df0a9d6e98a8: the wrapper blob remains
71d906ad4a716902bafdf02423d8fde27f0081c2, identical to our pinned comparison.

[Seven finite process-group observations]({base}experiments/issue-1580/evidence/checks/php-budget-parent-exit/verification.json)
all see a live worker when this template wrapper exits. Expired parent-success
returns 0 and parent-failure returns 7 instead of the expected 124; a TERM-resistant
worker survives too. The positive short-worker/warning-only status controls also
exit while their worker is still live. Unavailable process inspection is retained
as a separate case. The current Formal AI counterpart passes the same seven
unchanged controls.

The [probe]({base}experiments/issue-1580/probe-budget-process-lifetime.py) uses a
Linux subreaper, finite four-second workers, a ten-second observation boundary,
bounded stack/address space, and cleanup of only its own process group. It checks
liveness before cleanup, so cleanup cannot create a false success. No sudo or
privileged process is launched; these results do not qualify Windows or EPERM
behavior. stdout/stderr, original status and source hashes are retained per case.

The existing PipelineShellScriptsTest expects zero while a background worker is
still live and kills the worker itself afterward; it does not enforce the stronger
ownership contract. These controls supplement the original report without
duplicating the closed issue. They do not establish an upstream repair.
Context: https://github.com/link-assistant/formal-ai/pull/1581.
''')
print('Prepared two issue bodies and one additional-evidence comment; no external message sent.')
