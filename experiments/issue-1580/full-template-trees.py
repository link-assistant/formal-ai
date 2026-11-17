"""Expand pinned archives, record every blob, and classify all CI entry points."""
import hashlib, json, subprocess, tarfile
from pathlib import Path

root=Path('/tmp/issue-1580-research/templates')
for row in json.loads((root/'manifest.json').read_text()):
    language=row['repo'].split('/')[1].split('-')[0]
    archive=root/(language+'.tar.gz')
    if not archive.exists():
        with archive.open('wb') as output:
            subprocess.run(['gh','api',f"repos/{row['repo']}/tarball/{row['sha']}"],stdout=output,check=True)
    target=root/(language+'-complete'); target.mkdir(exist_ok=True)
    with tarfile.open(archive) as source:
        members=source.getmembers()
        prefix=members[0].name.split('/')[0]+'/'
        for member in members:
            if not member.isfile(): continue
            relative=Path(member.name.removeprefix(prefix))
            assert not relative.is_absolute() and '..' not in relative.parts
            path=target/relative; path.parent.mkdir(parents=True,exist_ok=True)
            path.write_bytes(source.extractfile(member).read())
    tree=json.loads((root/(language+'-tree.json')).read_text())['tree']
    selected=[]
    for blob in tree:
        if blob['type']!='blob': continue
        path=target/blob['path']
        # Git symlinks are not dereferenced: their link text is the blob content.
        if not path.exists():
            assert blob['mode']=='120000', blob['path']
            content=subprocess.check_output(['gh','api',f"repos/{row['repo']}/git/blobs/{blob['sha']}",'--jq','.content'])
            import base64
            path.parent.mkdir(parents=True,exist_ok=True); path.write_bytes(base64.b64decode(content))
        content=path.read_bytes()
        actual=hashlib.sha1(b'blob '+str(len(content)).encode()+b'\0'+content).hexdigest()
        assert actual==blob['sha'], (language,blob['path'])
        selected.append({'path':blob['path'],'gitBlob':actual,'bytes':len(content)})
    (root/(language+'-complete-inventory.json')).write_text(json.dumps(selected,indent=2)+'\n')
    print(language,len(selected),'complete blobs verified',flush=True)
