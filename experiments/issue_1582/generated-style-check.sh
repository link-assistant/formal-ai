#!/usr/bin/env bash
set -eu
projection=rust/tests/fixtures/js-rust-translation/crate/version_resolution.rs
git show "origin/main:$projection" > /tmp/issue-1582-base-projection.rs
if rustfmt --edition 2024 --check /tmp/issue-1582-base-projection.rs > /tmp/issue-1582-base-projection-format.log 2>&1; then base_status=0; else base_status=$?; fi
if rustfmt --edition 2024 --check "$projection" > /tmp/issue-1582-current-projection-format.log 2>&1; then current_status=0; else current_status=$?; fi
printf 'Base generated projection rustfmt status: %s\nCurrent generated projection rustfmt status: %s\n' "$base_status" "$current_status"
test "$base_status" = "$current_status"
bash experiments/issue_1582/native-final-audit.sh
python3 - <<'PY'
from pathlib import Path
p=Path('docs/case-studies/issue-1582/README.md')
s=p.read_text().replace('requirements/status documents are refreshed with their maintained generators.',
    'requirements/status documents are refreshed with their maintained generators.\nChanged handwritten Rust source and tests pass `rustfmt --check`. The generated\nJavaScript-to-Rust fixture retains upstream presentation: rustfmt rejects both\nits base and current versions with the same formatting-only result. Its sole\nchange is the regenerated source-identity comment; the maintained projection\ncurrency check verifies it instead of hand-formatting generated output.')
p.write_text(s)
p=Path('experiments/issue_1582/pull-request.md')
p.write_text(p.read_text().replace('All changed Rust files pass `rustfmt --check`;', 'Changed handwritten Rust source/tests pass `rustfmt --check`; the generated projection retains its upstream layout (the same formatting-only rejection on base and current) and passes its maintained currency check;'))
PY
