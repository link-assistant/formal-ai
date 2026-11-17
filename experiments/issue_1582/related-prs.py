from pathlib import Path
import json, subprocess

root=Path('docs/case-studies/issue-1582/raw-data/research')
for term in ['claude', 'context']:
    data=subprocess.check_output(['gh','pr','list','--repo','link-assistant/formal-ai','--state','merged','--search',term,'--limit','5','--json','number,title,url,mergedAt,body'],text=True)
    (root/('recent-'+term+'-prs.json')).write_text(data)
    print(data)
