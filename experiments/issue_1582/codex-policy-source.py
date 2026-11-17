"""Inspect archived official permission configuration contracts."""
from pathlib import Path
import gzip
root=Path('docs/case-studies/issue-1582/raw-data/research')
for name in ['codex-config_toml.rs.gz','codex-models.rs.gz']:
 lines=gzip.decompress((root/name).read_bytes()).decode().splitlines()
 for i,line in enumerate(lines):
  if any(term in line for term in ['struct Permissions','struct Network','network_access:','permissions:', 'active_permissions', 'read-only-with']):
   print(name)
   for n in range(max(0,i-3),min(len(lines),i+16)):print(f'{n+1}: {lines[n]}')
