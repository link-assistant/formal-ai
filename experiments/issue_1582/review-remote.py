"""Preserve paginated PR patches when GitHub rejects the combined diff."""
from pathlib import Path
import json, subprocess, sys

root = Path('docs/case-studies/issue-1582/raw-data')
for argument in sys.argv[1:]:
    if argument.startswith('--output-root='):
        root = Path(argument.split('=', 1)[1])
root.mkdir(parents=True, exist_ok=True)
repo = 'link-assistant/formal-ai'
pr = json.loads(subprocess.check_output(['gh', 'pr', 'view', '1583', '--repo', repo, '--json', 'url,title,isDraft,headRefOid,mergeStateStatus,changedFiles,body'], text=True))
assert pr['headRefOid'] == subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
suffix = pr['headRefOid'][:12]
result = subprocess.check_output(['gh', 'api', f'repos/{repo}/pulls/1583/files?per_page=100', '--paginate', '--slurp'], text=True)
pages = json.loads(result)
files = [file for page in pages for file in page]
(root / f'pr-files-{suffix}.json').write_text(json.dumps(files, indent=2) + '\n')
(root / f'pr-{suffix}.json').write_text(json.dumps(pr, indent=2) + '\n')
assert len(files) == pr['changedFiles'], (len(files), pr['changedFiles'])
present = [file for file in files if file['status'] != 'removed']
hashes = subprocess.check_output(['git', 'hash-object', '--stdin-paths'], input=''.join(file['filename'] + '\n' for file in present), text=True).splitlines()
assert all(blob == file['sha'] for blob, file in zip(hashes, present, strict=True)), 'remote/local file content differs'
with Path('/tmp/issue-1582-remote-production.diff').open('w') as output:
    for file in files:
        if file['filename'].startswith(('rust/', 'js/', 'ts/', 'data/', 'changelog.d/', '.cargo/')):
            output.write(file['filename'] + '\n' + file.get('patch', '<no patch>') + '\n')
print(json.dumps({key: pr[key] for key in ['url', 'title', 'isDraft', 'headRefOid', 'mergeStateStatus', 'changedFiles']}))
print(json.dumps({'checkedRemoteBlobIdentities': len(present), 'allMatchLocal': True}))
