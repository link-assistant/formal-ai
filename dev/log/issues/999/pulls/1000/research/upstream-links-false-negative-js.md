## Problem

`.github/workflows/links.yml` reports success when Lychee finds broken live links if every broken URL has a Wayback snapshot. The repository still contains and serves broken links; the helper only prints suggested replacements and does not rewrite them.

Current terminal condition:

```yaml
if: steps.lychee.outputs.exit_code != 0 && steps.webarchive.outputs.all_archived != 'true'
```

This makes an archived-but-not-replaced dead link a CI false negative.

## Reproduction

1. Put a known dead URL with an existing Wayback snapshot in a checked Markdown file.
2. Run the Broken Link Checker workflow.
3. Lychee returns a nonzero `exit_code`.
4. `scripts/check-web-archive.mjs` finds the snapshot and outputs `all_archived=true`.
5. The terminal failure step is skipped, so the workflow is green although the checked Markdown still points to the dead live URL.

The condition can also be reproduced without depending on a live site: feed the helper a fixture `lychee/out.md` containing `[404] <archived-url>`, observe `all_archived=true`, and evaluate the condition above.

## Workaround

Fail on every nonzero Lychee result while retaining the Wayback step for actionable replacement diagnostics:

```yaml
- name: Fail if broken links were found
  if: always() && steps.lychee.outputs.exit_code != 0
  run: |
    echo "::error::Broken live links were detected."
    exit 1
```

`always()` ensures the explanation runs even if the Wayback API/helper itself fails.

## Suggested code fix

Change the terminal condition as above and update its message to explain that an archive is a suggested replacement, not a reason to accept the still-broken source link. Add a workflow-structure test asserting the failure condition depends only on Lychee's nonzero exit and does not exempt `all_archived=true`.

Found while auditing link-assistant/formal-ai#999 against template revision `9af528fb034643c03b4354e5273a8a20d830ee02`.
