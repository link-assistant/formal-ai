"""Refresh feedback and preserve the official final Codex configuration docs."""
from pathlib import Path
import json, subprocess, urllib.request

root = Path('docs/case-studies/issue-1582/raw-data')
for endpoint, filename in [
    ('issues/1582/comments', 'issue-comments-before-push.json'),
    ('issues/1583/comments', 'pr-comments-before-push.json'),
    ('pulls/1583/comments', 'inline-before-push.json'),
    ('pulls/1583/reviews', 'reviews-before-push.json'),
]:
    result = subprocess.run(['gh', 'api', 'repos/link-assistant/formal-ai/' + endpoint, '--paginate'], check=True, capture_output=True, text=True)
    (root / filename).write_text(result.stdout)
    for item in json.loads(result.stdout):
        print(json.dumps({key: item.get(key) for key in ['html_url', 'created_at', 'state', 'body']}, ensure_ascii=False))
for name, url in [
    ('codex-config-reference.md', 'https://learn.chatgpt.com/docs/config-file/config-reference.md'),
    ('codex-permissions.md', 'https://learn.chatgpt.com/docs/permissions.md'),
]:
    with urllib.request.urlopen(url, timeout=30) as response:
        content = response.read(2_000_001)
    if len(content) > 2_000_000:
        raise ValueError('Unexpectedly large documentation page')
    (root / 'research' / name).write_bytes(content)
    print(name, len(content))
subprocess.run(['git', 'fetch', 'origin', 'main'], check=True)
subprocess.run(['git', 'merge-base', '--is-ancestor', 'origin/main', 'HEAD'], check=True)
subprocess.run(['git', 'diff', '--check'], check=True)
