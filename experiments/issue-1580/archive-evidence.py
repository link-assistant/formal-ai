"""Preserve full command transcripts compactly, with hashes of both representations."""
import gzip, hashlib, json
from pathlib import Path
root=Path('experiments/issue-1580/evidence/commands')
index=root/'index.jsonl'
rows=[json.loads(line) for line in index.read_text().splitlines()] if index.exists() else []
known={row['file'] for row in rows}
for file in sorted(root.glob('*.json')):
    data=file.read_bytes()
    value=json.loads(data)
    compressed=gzip.compress(data,mtime=0)
    target=file.with_suffix('.json.gz')
    target.write_bytes(compressed)
    row={'file':target.name,'sha256':hashlib.sha256(data).hexdigest(),'compressedSha256':hashlib.sha256(compressed).hexdigest(),'source':value['source'],'status':value['status'],'signal':value['signal'],'autonomousAuthorshipCredit':0}
    if target.name not in known: rows.append(row)
    file.unlink()
index.write_text(''.join(json.dumps(row,separators=(',',':'))+'\n' for row in rows))
print('Archived',len(rows),'complete Formal AI command records; supplied commands earn zero authorship credit.')
