from pathlib import Path
import gzip,hashlib,json,shutil,subprocess
root=Path('/tmp/gh-issue-solver-1794900398048');scratch=Path('/tmp/issue-1580-research')
source=scratch/'php-input-integrity-upstream-publication';outcome=json.loads((source/'outcome.json').read_text());assert outcome['complete']
assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=root,text=True).strip()
archive=root/'experiments/issue-1580/evidence/checks/php-input-integrity-upstream-publication';archive.mkdir(exist_ok=False)
for path in sorted(source.iterdir()):
    if path.is_file():shutil.copyfile(path,archive/path.name)
for name in ['report-php-input-integrity-upstream.py','retain-php-upstream-publication.py']:shutil.copyfile(scratch/name,archive/name)
reports={row['kind']:row['url'] for row in outcome['reports']}
doc=root/'docs/case-studies/issue-1580/php-input-integrity-checkpoint.md'
doc.write_text('''# PHP input and process ownership checkpoint

The pinned template at 3a034b27a56067971ca74dd4c74c7707056b215d and fresh source
b61e334703d93a15e08bf1a278e0df0a9d6e98a8 have identical version-guard,
release-note and budget-wrapper blobs. The maintained fifteen-case input test
executes actual PHP without installing packages or making remote Git requests.
Seven bounded process-group cases separately compare the pinned wrapper with
the current Formal AI counterpart.

- Version comparison converts malformed, missing and wrong-type evidence into
  successful unchanged/bootstrap decisions. Checked fetch/commit/path controls
  in the disposable candidate reject unknown evidence while retaining genuine
  first-file, unchanged-version and ordinary changed-version behavior.
- Release-note truncation removes UTF-8 continuation bytes but leaves the leading
  byte, including when the original cut ends at a complete character. The actual
  default 120000-byte cap reproduces invalid UTF-8. Caps smaller than the notice
  also exceed their requested limit. The candidate preserves tested boundaries;
  arbitrary Unicode notices and URLs remain outside that qualification.
- All seven original process controls leave a worker alive at wrapper exit.
  Expired parent-success/failure cases return 0/7 rather than 124. The existing
  template test kills the worker itself after expecting success. Formal AI's
  wrapper passes the same seven unchanged controls. No privileged process is
  launched; Windows and EPERM behavior remain unqualified.

Upstream reports, with immutable reproduction links:

'''+''.join('- ['+name+']('+url+')\n' for name,url in [('Version guard',reports['version']),('UTF-8 release notes',reports['notes']),('Additional evidence on existing process ownership issue',reports['budget'])])+'''
The two reports and the comment establish reporting, not upstream repair. The
complete PHP tree comparison and runtime/practice acceptance remain unfinished.
All supplied commands, candidates and reviews retain zero independent authorship
or reasoning credit. PR #1581 remains draft and unmerged.

Run the maintained reproducer from the Formal AI repository:

```sh
node --test --test-concurrency=1 experiments/issue-1580/template-php-input-integrity.test.mjs
```

Original/candidate logs, bounded input definitions and source hashes are retained
under `experiments/issue-1580/evidence/checks/php-input-integrity/`. Actual posted
bodies, URLs, paginated deduplication observations and command records are under
`experiments/issue-1580/evidence/checks/php-input-integrity-upstream-publication/`.
''')
body_path=scratch/'query-import-php-publication/pull-request-body.md';body=body_path.read_text()
insert='- PHP input and ownership reproductions: [version guard #31]('+reports['version']+'), [UTF-8 release notes #32]('+reports['notes']+'), and [additional bounded evidence on existing issue #15]('+reports['budget']+'). Fifteen actual PHP original/candidate input tests and seven process-group controls retain their limits and positive controls. Reporting does not establish upstream repair.\n'
assert insert not in body;body=body.replace('Upstream reproductions and reports:\n\n','Upstream reproductions and reports:\n\n'+insert)
assert body.count('Fixes #1580')==1;new_body=source/'pull-request-body-with-upstream.md';new_body.write_text(body)
previous=subprocess.run(['gh','pr','view','1581','--repo','link-assistant/formal-ai','--json','body,isDraft,state'],cwd=root,capture_output=True)
assert previous.returncode==0;data=json.loads(previous.stdout);assert data['isDraft'] and data['state']=='OPEN' and data['body']==body_path.read_text()
updated=subprocess.run(['gh','pr','edit','1581','--repo','link-assistant/formal-ai','--body-file',str(new_body)],cwd=root,capture_output=True)
for name,result in [('description-before',previous),('description-update',updated)]:
    for label in ['stdout','stderr']:(archive/(name+'.'+label+'.gz')).write_bytes(gzip.compress(getattr(result,label),mtime=0))
assert updated.returncode==0,updated.stderr.decode();body_path.write_text(body)
(archive/'description-identity.json').write_text(json.dumps({'sha256':hashlib.sha256(body.encode()).hexdigest(),'bytes':len(body.encode()),'fixesOriginalIssue':True,'draftPreserved':True,'fullIssueComplete':False},indent=2)+'\n')
shutil.copyfile(new_body,archive/new_body.name)
subprocess.run(['git','add','--',str(doc.relative_to(root)),str(archive.relative_to(root))],cwd=root,check=True)
subprocess.run(['git','diff','--cached','--check'],cwd=root,check=True)
result=subprocess.run(['git','commit','-m','Retain PHP upstream input and process ownership reports','-m','Include immutable reproductions and posted report identities without claiming an upstream repair or complete template qualification.','-m','Formal-AI-Independent-Authorship: 0'],cwd=root,capture_output=True)
assert result.returncode==0,result.stderr.decode()
print(json.dumps({'committed':True,'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'reports':reports,'descriptionUpdated':True,'pushExecuted':False,'fullIssueComplete':False}))
