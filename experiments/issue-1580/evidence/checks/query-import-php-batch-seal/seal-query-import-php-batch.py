from pathlib import Path
from datetime import datetime, timezone
import gzip, hashlib, json, os, shutil, subprocess

root=Path('/tmp/gh-issue-solver-1794900398048')
scratch=Path('/tmp/issue-1580-research')
target=root/'experiments/issue-1580/evidence/checks/query-import-php-batch-seal'
target.mkdir(exist_ok=False)
records=[]
def run(name, argv):
    result=subprocess.run(argv,cwd=root,env=dict(os.environ,NODE_OPTIONS='--max-old-space-size=256'),capture_output=True)
    row={'name':name,'argv':argv,'exitCode':result.returncode}
    for label in ['stdout','stderr']:
        raw=getattr(result,label);name_=name+'.'+label+'.gz';packed=gzip.compress(raw,mtime=0)
        (target/name_).write_bytes(packed)
        row[label]={'file':name_,'rawSha256':hashlib.sha256(raw).hexdigest(),'archiveSha256':hashlib.sha256(packed).hexdigest(),'bytes':len(raw),'lines':len(raw.splitlines())}
    records.append(row);(target/'records.json').write_text(json.dumps(records,indent=2)+'\n')
    assert result.returncode==0,(name,result.stderr.decode(errors='replace')[-2000:])
    return result.stdout.decode()

assert run('branch',['git','branch','--show-current']).strip()=='issue-1580-8d61a32a61e1'
head=run('head',['git','rev-parse','HEAD']).strip()
assert head=='a0798663bcffd11a46cd2cb202bb76ef42002071'
batch=root/'experiments/issue-1580/evidence/checks/query-import-php-batch-validation'
outcome=json.loads((batch/'outcome.json').read_text());assert outcome['finished'] and not outcome['allPassed']
batch_records=json.loads((batch/'records.json').read_text())
assert [r['name'] for r in batch_records if r['exitCode']] == ['all-local-gates']
gate_output=gzip.decompress((batch/'all-local-gates.stdout.gz').read_bytes()).decode()
assert [line.split()[1] for line in gate_output.splitlines() if line.startswith('FAIL ')]==['check-abbreviations']
assert '140/141 passed or skipped' in gate_output
run('abbreviations-after-receipt-renames',['node','scripts/measure-abbreviations.mjs','--check'])
run('whitespace',['git','diff','--check'])
for name in ['data/meta/abbreviation-ratchet.lino','data/meta/source-size-policy.lino']:
    if (root/name).exists():
        assert subprocess.check_output(['git','diff','d48b0988','--',name],cwd=root)==b''

review=scratch/'query-import-php-diff-review';review_metadata=json.loads((review/'summary.json').read_text())
assert review_metadata['head']==head and review_metadata['lines']==606
for part in review_metadata['parts']:
    assert hashlib.sha256((review/part['file']).read_bytes()).hexdigest()==part['sha256']
for source in review.iterdir():
    shutil.copyfile(source,target/('repair-review-'+source.name))
review_metadata.update(literalReviewComplete=True,wholePrReviewComplete=False,reviewedAt=datetime.now(timezone.utc).isoformat(),reviewResult='All 606 lines across twelve source paths read. No additional repair identified. Hosted native compilation, query performance/equivalence and whole PR review remain unverified.')
(target/'repair-review-confirmation.json').write_text(json.dumps(review_metadata,indent=2)+'\n')
feedback=scratch/'query-import-php-feedback'
feedback_summary=json.loads((feedback/'summary.json').read_text())
assert feedback_summary['mainAncestor'] and not feedback_summary['newFeedbackKinds']
for source in feedback.iterdir():
    shutil.copyfile(source,target/('feedback-'+source.name))

for doc in [root/'experiments/issue-1580/work-plan.md',root/'docs/case-studies/issue-1580/source-warning-extraction-checkpoint.md',scratch/'query-import-php-publication/pull-request-body.md']:
    text=doc.read_text()
    old='Its final\noutcome and the focused repaired check are recorded with the batch seal.'
    new='The full replay finishes 140/141 passed or skipped; its only failure is\ncheck-abbreviations at 93 versus the unchanged ceiling of 90. Renaming three\nsource archive receipts preserves every payload and the focused check passes\nat 90 filenames, 718 JavaScript bindings and 908 Rust bindings. The failed full\nreplay and passing focused repair remain separate evidence in the batch seal.'
    text=text.replace(old,new)
    text=text.replace('Final replay/focused outcomes are retained in the batch seal.','The full replay is 140/141 passed or skipped, with only the 93-versus-90 filename check failing; after three payload-preserving source receipt renames the focused check passes at 90/718/908. Both outcomes remain separate in the batch seal.')
    doc.write_text(text)
report=scratch/'php-upstream-reports/version.md'
text=report.read_text();assert '`versionFromJson`' in text;report.write_text(text.replace('`versionFromJson`','`versionField`'))
for name in ['seal-query-import-php-batch.py','run-query-import-php-batch-validation.py','refresh-query-import-php-feedback.py','resume-query-import-php-feedback.py','prepare-current-repair-diff-review.py','prepare-query-import-php-checkpoint.py','prepare-php-upstream-reports.py']:
    shutil.copyfile(scratch/name,target/name)
(target/'README.md').write_text('The complete 141-entry local replay retains one failure caused by three abbreviated receipt filenames. The payload-preserving renames restore the existing ceiling; the focused check passes. No replay is relabeled as wholly green. The 606-line twelve-path repair diff was fully read, independently of the 486867-line whole-PR diff, whose review remains incomplete. Published d48b0988 has three fully collected actionable failures: producer fingerprint, extracted parent imports and included-string query timeout. The query specialization is an unverified performance hypothesis with opt-in hosted tracing. Native and query runtime results require the next hosted run. Full issue completion and independent authorship credit remain false/zero.\n')
result={'passed':True,'sourceHead':head,'observedAt':datetime.now(timezone.utc).isoformat(),'fullReplay':{'entries':141,'passedOrSkipped':140,'failed':['check-abbreviations']},'focusedRepairedCheckPassed':True,'ratchetsChanged':False,'templateCasesPassed':93,'selectedVersionCasesPassed':9,'nativeExecutionPerformedLocally':False,'includedStringQueryRuntimeQualified':False,'repairDiffReviewedLines':606,'wholePrReviewComplete':False,'fullIssueComplete':False,'independentAuthorshipCredit':0}
(target/'outcome.json').write_text(json.dumps(result,indent=2)+'\n')
(scratch/'query-import-php-publication/seal.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
