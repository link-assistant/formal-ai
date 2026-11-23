"""Exercise the locked scanner with actual npm/engine; keep installed files temporary."""
from pathlib import Path
import datetime,gzip,json,os,shutil,subprocess,sys,tempfile
repository=Path(__file__).resolve().parents[2]
output=Path(sys.argv[1]).resolve();output.mkdir(parents=True,exist_ok=True)
directory=Path(tempfile.mkdtemp(prefix='secretlint-locked-live-',dir=repository/'experiments/issue-1580'))
records=[]
try:
 installations=directory/'installations';installations.mkdir()
 fixtures=directory/'fixtures';fixtures.mkdir()
 token='gh'+'p_'+'A'*36
 cases=[('clean','ordinary content',False,0),('clean-gzip','ordinary content',True,0),
        ('finding',token,False,1),('finding-gzip',token,True,1),('corrupt-gzip','not gzip',True,1)]
 for label,content,compressed,expected in cases:
  fixture=fixtures/(label+('.txt.gz' if compressed else '.txt'))
  fixture.write_bytes(gzip.compress(content.encode()) if compressed and label!='corrupt-gzip' else content.encode())
  env=dict(os.environ,TMPDIR=str(installations),SECRETLINT_VERSION='',NODE_DISABLE_COMPILE_CACHE='1')
  command=['bash','scripts/check-secrets.sh',str(fixture)]
  result=subprocess.run(command,cwd=repository,env=env,capture_output=True,text=True)
  raw=result.stdout+result.stderr
  (output/(label+'.log')).write_text(raw.replace(token,'[synthetic credential redacted]'))
  record={'label':label,'command':command,'status':result.returncode,'expectedStatus':expected,
          'unmaskedSyntheticCredential':token in raw,'installationDirectoriesRemaining':len(list(installations.iterdir())),
          'remainingEntries':[entry.name for entry in installations.iterdir()],
          'cleanResultReported':'No secrets found.' in raw,'findingMasked':'***' in raw,
          'actualScannerExecuted':'"secretsScan"' in raw}
  record['passed']=(result.returncode==expected and not record['unmaskedSyntheticCredential']
    and record['installationDirectoriesRemaining']==0 and record['actualScannerExecuted']
    and record['cleanResultReported']==(expected==0)
    and (label not in ['finding','finding-gzip'] or record['findingMasked']))
  records.append(record);print(json.dumps(record),flush=True)
 mismatched=directory/'mismatched';mismatched.mkdir()
 for name in ['package.json','package-lock.json']:shutil.copy2(repository/'.github/secretlint'/name,mismatched/name)
 lock=json.loads((mismatched/'package-lock.json').read_text())
 lock['packages']['node_modules/secretlint']['version']='13.0.5'
 (mismatched/'package-lock.json').write_text(json.dumps(lock)+chr(10))
 command=['npm','ci','--offline','--ignore-scripts','--no-audit','--no-fund','--engine-strict','--prefix',str(mismatched)]
 result=subprocess.run(command,cwd=repository,capture_output=True,text=True)
 (output/'mismatched-lock.log').write_text(result.stdout+result.stderr)
 records.append({'label':'mismatched-lock','command':command,'status':result.returncode,
                 'passed':result.returncode!=0 and 'EUSAGE' in result.stderr and 'lock file' in result.stderr and not (mismatched/'node_modules').exists()})
 command=['npm','audit','--package-lock-only','--prefix','.github/secretlint','--json']
 result=subprocess.run(command,cwd=repository,capture_output=True,text=True)
 (output/'locked-audit.json').write_text(result.stdout);(output/'locked-audit.stderr').write_text(result.stderr)
 records.append({'label':'locked-audit','command':command,'status':result.returncode,'passed':result.returncode==0})
finally:
 shutil.rmtree(directory)
 (output/'actual-result.json').write_text(json.dumps({'records':records,'complete':len(records)==7,
  'allPassed':len(records)==7 and all(row['passed'] for row in records),
  'finishedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),
  'fullIssueComplete':False,'independentAuthorshipCredit':0},indent=2)+'\n')
if len(records)!=7 or not all(row['passed'] for row in records):sys.exit(1)
