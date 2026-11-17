"""Trace the lockfile advisory without a Rust build and retain primary data."""
from pathlib import Path
import json, subprocess, tomllib

root = Path('docs/case-studies/issue-1582/raw-data/research')
lock = tomllib.loads(Path('rust/Cargo.lock').read_text())['package']
pending = ['chrono']
seen = set()
while pending:
    name = pending.pop(0)
    if name in seen:
        continue
    seen.add(name)
    for package in lock:
        if any(dependency.split()[0] == name for dependency in package.get('dependencies', [])):
            print(f"{name} <- {package['name']}@{package['version']}")
            pending.append(package['name'])
for endpoint, filename in [
    ('repos/RustSec/advisory-db/contents/crates/chrono/RUSTSEC-2026-0543.md', 'chrono-advisory.md'),
    ('repos/chronotope/chrono', 'chrono-repository.json'),
]:
    result = subprocess.run(['gh', 'api', endpoint, '-H', 'Accept: application/vnd.github.raw+json'], text=True, capture_output=True)
    (root / filename).write_text(result.stdout if result.returncode == 0 else result.stderr)
    print(filename, result.returncode)
    if filename.endswith('.md'):
        print(result.stdout or result.stderr)
