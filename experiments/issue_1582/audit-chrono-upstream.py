"""Inspect the two direct dependencies that introduce unmaintained chrono."""
from pathlib import Path
import json, subprocess, urllib.parse, urllib.request

root = Path('docs/case-studies/issue-1582/raw-data/research')
for name in ['command-stream', 'link-calculator']:
    request = urllib.request.Request(f'https://crates.io/api/v1/crates/{name}', headers={'User-Agent': 'formal-ai-issue-1582-investigation'})
    with urllib.request.urlopen(request, timeout=30) as response:
        data = response.read(2_000_001)
    assert len(data) <= 2_000_000
    (root / f'{name}-crate.json').write_bytes(data)
    crate = json.loads(data)['crate']
    print(json.dumps({key: crate.get(key) for key in ['name', 'newest_version', 'repository', 'updated_at']}))
    repository = crate['repository'].removeprefix('https://github.com/').removesuffix('.git').rstrip('/')
    query = urllib.parse.quote(f'chrono repo:{repository}')
    result = subprocess.run(['gh', 'api', f'search/issues?q={query}&per_page=100', '--paginate'], capture_output=True, text=True, check=True)
    (root / f'{name}-chrono-issues.json').write_text(result.stdout)
    for issue in json.loads(result.stdout)['items']:
        print(json.dumps({key: issue.get(key) for key in ['html_url', 'title', 'state', 'body']}))
    source = subprocess.run(['gh', 'api', f'repos/{repository}/contents/rust/Cargo.toml', '-H', 'Accept: application/vnd.github.raw+json'], text=True, capture_output=True)
    (root / f'{name}-Cargo.toml').write_text(source.stdout or source.stderr)
    print(source.stdout or source.stderr)
    with urllib.request.urlopen(urllib.request.Request(f'https://crates.io/api/v1/crates/{name}/{crate["newest_version"]}/dependencies', headers={'User-Agent': 'formal-ai-issue-1582-investigation'}), timeout=30) as response:
        dependencies = response.read(2_000_001)
    assert len(dependencies) <= 2_000_000
    (root / f'{name}-latest-dependencies.json').write_bytes(dependencies)
    print(dependencies.decode())
