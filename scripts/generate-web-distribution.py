#!/usr/bin/env python3
"""Prepare offline and npm artifacts after build:web without running a compiler."""
from hashlib import sha256
import json
from pathlib import Path
import re
import shutil

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / 'js'
SEED = ROOT / 'data/seed'
PACKAGE = ROOT / 'packages/formal-ai-engine'
EXCLUDED = {'precache-manifest.js', 'service-worker.js', 'deployment.json'}
assets = {}
for path in sorted(WEB.rglob('*')):
    if path.is_file() and path.suffix in {'.js', '.css', '.html', '.wasm', '.lino', '.png', '.svg', '.webmanifest'}:
        name = path.relative_to(WEB).as_posix()
        # vendor/ holds the Node-side tree-sitter runtime and grammar (issue
        # #1180 R11); no page loads them, so they are not precached.
        if path.name not in EXCLUDED and not name.startswith(('app/', 'distribution/', 'seed/', 'vendor/')):
            assets[name] = path
# A selected CI build supplies a source-bound receipt; local source-only builds stay compiler-free.
receipt = WEB / 'formal_ai_worker.receipt.json'
if receipt.is_file():
    assets[receipt.name] = receipt
assets['app/index.html'] = WEB / 'app/index.html'
for path in sorted(SEED.rglob('*.lino')):
    assets['seed/' + path.relative_to(SEED).as_posix()] = path
for required in ['app.js', 'vendor.bundle.js', 'formal_ai_worker.wasm', 'seed-files.js']:
    if required not in assets:
        raise SystemExit('Required built asset missing: ' + required)
digest = sha256()
for name, path in sorted(assets.items()):
    digest.update(name.encode()); digest.update(b'\0'); digest.update(path.read_bytes()); digest.update(b'\0')
manifest = {'version': digest.hexdigest(), 'files': sorted(assets)}
(WEB / 'precache-manifest.js').write_text('self.FORMAL_AI_PRECACHE = ' + json.dumps(manifest, separators=(',', ':')) + ';\n')
shutil.copyfile(WEB / 'distribution/service-worker.js', WEB / 'service-worker.js')
output = PACKAGE / 'assets'
if output.exists():
    shutil.rmtree(output)
# Producer receipts remain in offline assets; public npm bytes are independent of run identity.
package_assets = {name: path for name, path in assets.items() if name != receipt.name}
for name, path in package_assets.items():
    destination = output / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(path, destination)
shutil.copyfile(ROOT / 'LICENSE', PACKAGE / 'LICENSE')
version = re.search(r'^version\s*=\s*"([^\"]+)"', (ROOT / 'rust/Cargo.toml').read_text(), re.M).group(1)
metadata_path = PACKAGE / 'package.json'
metadata = json.loads(metadata_path.read_text())
metadata['version'] = version
metadata_path.write_text(json.dumps(metadata, indent=2) + '\n')
print('Prepared offline cache ' + manifest['version'] + ' and engine package ' + version)
