"""Supplied follow-up after the first gate refresh, zero authorship credit."""
from pathlib import Path
path = Path('docs/requirements-traceability.md')
text = path.read_text()
rows = text.splitlines()
for index, row in enumerate(rows):
    if row.startswith('| R1582-3 |'):
        rows[index] = row.replace('rust/tests/integration/with_formal_ai/issue_1582.rs; ', '')
path.write_text('\n'.join(rows)+'\n')
path = Path('rust/tests/unit/issue_1173_fallback_executes_search.rs')
text = path.read_text().replace(
    'fetches stay behind the `FORMAL_AI_LIVE_FETCH` opt-in, so this suite never',
    'fetches use explicit offline solver configuration, so this suite never')
path.write_text(text)
path = Path('rust/src/solver_handlers/web_requests/live_search.rs')
path.write_text(path.read_text().replace('js/runtime/read-access.mjs', 'js/server/read-access.mjs'))
