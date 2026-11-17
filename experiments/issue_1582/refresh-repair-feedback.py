"""Refresh every feedback channel and the main-branch relationship before push."""
from pathlib import Path
import json, subprocess
root = Path('docs/case-studies/issue-1582/raw-data')
for endpoint, name in [
    ('issues/1582', 'issue-after-repair'),
    ('issues/1582/comments', 'issue-comments-after-repair'),
    ('issues/1583/comments', 'pr-comments-after-repair'),
    ('pulls/1583/comments', 'inline-after-repair'),
    ('pulls/1583/reviews', 'reviews-after-repair'),
]:
    raw = subprocess.check_output(['gh', 'api', 'repos/link-assistant/formal-ai/' + endpoint, '--paginate'], text=True)
    (root / (name + '.json')).write_text(raw)
    print(name, raw)
subprocess.run(['git', 'fetch', 'origin', 'main'], check=True)
subprocess.run(['git', 'merge-base', '--is-ancestor', 'origin/main', 'HEAD'], check=True)
print('main', subprocess.check_output(['git', 'rev-parse', 'origin/main'], text=True).strip())
