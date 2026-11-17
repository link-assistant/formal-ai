The documented release-formatting CLI corrupts literal escape sequences inside release-note examples and then submits the changed body successfully. GitHub CLI `api --jq .body` has already decoded the API response; replacing backslash sequences again changes the note content.

Affected source: [`scripts/format_release_notes.py` lines 119–122](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/blob/c532c173801dfb12e681b0730c4abbf93aefd0ca/scripts/format_release_notes.py#L119-L122), current `main` rechecked at `c532c173801dfb12e681b0730c4abbf93aefd0ca`. Full 266-line source Git blob: `8e6bdb61e4da36068e2dd931135588c46c8926f0`; SHA-256: `1d33ffb5d87a6b9a3036231d41ff79e085fd645e8c2611fac959c23c6fd75254`. Its complete 104-line `release_naming.py` helper is retained unchanged too.

Reproduce the content transformation from the template root without writing a real release:

```bash
python3 -B <<'PY'
import sys
sys.path.insert(0, 'scripts')
from format_release_notes import format_release_body
body = r'Example: {"message": "hello\nworld", "quote": "\"", "path": "C:\repo"}'
print(repr(format_release_body(body, '1.2.3', 'owner/repo', None, 'example-package')))
PY
```

Expected: the note receives its version badge and retains the example's literal `\n`, escaped quote and Windows-path backslash. Actual: `hello\nworld` becomes a real line break, `\"` loses the escape, and `C:\repo` becomes `C:epo`. This also happens inside fenced examples. These are ordinary note bytes, rather than JSON transport escaping that still needs decoding.

The complete unchanged CLI was also executed with an offline `gh` boundary. It reads the input body through the actual `get_release_body` path, calls the normal formatter and submits the corrupted text through its actual PATCH path, returning exit 0 and `Release notes updated successfully!`. No network request or real release mutation is made by the regression.

Suggested minimal repair: remove the three `cleaned_body.replace(...)` operations at lines 120–122. A disposable candidate makes only that change and leaves the existing version-heading/whitespace policy, badge insertion and API failure handling in place. Four full-CLI regression cases verify:

- the unchanged source reproduces corruption and successful submission;
- the candidate preserves literal escape sequences and Unicode while adding the version badge;
- an already formatted body remains idempotent, with no PATCH;
- API read/write failures remain fatal before and after the candidate.

The preservation assertion fails when the candidate is disabled (3 pass, 1 fail), then all four focused cases and all 38 cases in the maintained downstream entrypoint pass without skips. Python children are limited to 128 MiB address space and three CPU seconds, each child invocation has a five-second wall deadline, and Node's heap is capped at 96 MiB. Full unchanged source identities and complete before/after logs are retained in the downstream evidence.

Duplicate/history review covered the complete current 158-item issue/PR inventory, issue #6, issue #124, and merged PRs #88/#17 with their conversation comments, inline comments and reviews. Those concern naming/badges, fenced changelog extraction and note-size limits. The current badge tests cover URL-host recognition and idempotence, without literal-escape examples. This report concerns the standalone formatting CLI; the current release workflow invokes `create_github_release.py`, and no hosted release failure or automatic workflow invocation of this formatter is claimed.

Found during https://github.com/link-assistant/formal-ai/pull/1581. No upstream implementation change is claimed.
