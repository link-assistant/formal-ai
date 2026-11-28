from pathlib import Path
import gzip,hashlib,json
root=Path('/tmp/gh-issue-solver-1794900398048');scratch=Path('/tmp/issue-1580-research')
native=root/'experiments/issue-1580/evidence/ci/native-families-d48b0988';producer=root/'experiments/issue-1580/evidence/ci/codeql-producer-d48b0988'
for path in [native,producer]:
    outcome=json.loads((path/'outcome.json').read_text());assert outcome['complete'] and outcome['head']=='d48b09883ac91cd1d2319bea2afbaf0df2df8f86'
source=(scratch/'assess-artifact-and-child-memory-parent-ci.py').read_text().replace('native-families-90b51590','native-families-d48b0988').replace('jobs-44804037828','jobs-44843761970')
exec(compile(source,'current-native-assessment','exec'),{})
inventory=producer/'producer-files.json';original=inventory.read_bytes();files=json.loads(original);assert isinstance(files,list)
renames=[]
for row in files:
    old=row['file'];new=old.replace('_temp_','_temporary_');packed=(producer/old).read_bytes();raw=gzip.decompress(packed)
    assert hashlib.sha256(packed).hexdigest()==row['archiveSha256'] and hashlib.sha256(raw).hexdigest()==row['rawSha256']
    assert len(raw)==row['bytes'] and len(raw.splitlines())==row['lines']
    if new!=old:
        assert not (producer/new).exists();(producer/old).rename(producer/new);row['file']=new;renames.append({'old':old,'new':new,'archiveSha256':row['archiveSha256']})
(producer/'original-producer-inventory.json.gz').write_bytes(gzip.compress(original,mtime=0))
parts=[]
for start in range(0,len(files),40):
    name='producer-files-part-'+str(len(parts)+1)+'.json';raw=(json.dumps(files[start:start+40],indent=2)+'\n').encode();assert len(raw.splitlines())<=1400
    (producer/name).write_bytes(raw);parts.append({'file':name,'records':len(files[start:start+40]),'sha256':hashlib.sha256(raw).hexdigest()})
inventory.write_text(json.dumps({'format':'partitioned-producer-inventory','records':len(files),'parts':parts,'originalInventory':{'file':'original-producer-inventory.json.gz','rawSha256':hashlib.sha256(original).hexdigest(),'bytes':len(original),'lines':len(original.splitlines())}},indent=2)+'\n')
(producer/'archive-path-renames.json').write_text(json.dumps(renames,indent=2)+'\n')
source=(scratch/'assess-source-warning-macro-observations.py').read_text().replace('codeql-producer-f8b372ad','codeql-producer-d48b0988').replace("outcome['conclusion'] == 'success'","outcome['conclusion'] == 'failure'")
exec(compile(source,'current-successful-macro-profile-assessment','exec'),{})
lookup={row['relativePath']:row for row in files}
missing=[name for name in ['included-string-payloads/manifest.json','included-string-payloads/included-string-payloads.json','included-string-payloads/included-string-payloads.bqrs'] if name not in lookup]
assert len(missing)==3
lines=gzip.decompress((producer/'full-log.stdout.gz').read_bytes()).decode().splitlines()
errors=[{'line':n,'text':line} for n,line in enumerate(lines,1) if '##[error]' in line]
assert len(errors)==1 and errors[0]['line']==4819 and 'timed out after 4 minutes' in errors[0]['text']
result={'head':outcome['head'],'runId':outcome['runId'],'runConclusion':'failure','filesVerified':len(files),'includedStringQueryCompiled':True,'includedStringEvaluationComplete':False,'includedStringTableUnavailable':True,'missingExpectedFiles':missing,'originalErrors':errors,'standardSarifAndMacroProfileAssessment':'macro-observation-assessment.json','fullIssueComplete':False,'semanticCoverageProven':False,'independentAuthorshipCredit':0}
(producer/'included-string-failure-assessment.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
