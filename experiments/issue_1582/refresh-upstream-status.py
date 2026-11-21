"""Retain fresh upstream status and every comment without assuming a fix landed."""
from pathlib import Path
import json, subprocess
root = Path('docs/case-studies/issue-1582/raw-data/research')
for repo, number, name in [
    ('google-gemini/gemini-cli', 30333, 'gemini-final'),
    ('link-foundation/command-stream', 238, 'command-stream-final'),
    ('link-assistant/calculator', 265, 'calculator-final'),
]:
    for suffix, filename in [('', name), ('/comments', name + '-comments')]:
        raw = subprocess.check_output(['gh', 'api', f'repos/{repo}/issues/{number}{suffix}', '--paginate'], text=True)
        (root / (filename + '.json')).write_text(raw)
        value = json.loads(raw)
        for item in value if isinstance(value, list) else [value]:
            print(json.dumps({key: item.get(key) for key in ['html_url', 'state', 'updated_at', 'body']}, ensure_ascii=False))
