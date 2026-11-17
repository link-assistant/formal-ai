#!/usr/bin/env bash
set -eu
rg -n 'tool "|supported_protocols|context_window_tokens|max_output_tokens|contextWindow|context_window|limit.context|max_input_tokens|max_tokens' data/seed/client-integrations.lino
sed -n '1,150p' docs/case-studies/issue-1582/usage.md
sed -n '700,745p' /tmp/issue-1582-review-publish.log
