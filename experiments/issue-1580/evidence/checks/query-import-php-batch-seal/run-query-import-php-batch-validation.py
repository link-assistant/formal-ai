from pathlib import Path
import gzip,hashlib,json,os,subprocess,sys,traceback
root=Path('/tmp/gh-issue-solver-1794900398048');scratch=Path('/tmp/issue-1580-research/query-import-php-batch-validation');archive=root/'experiments/issue-1580/evidence/checks/query-import-php-batch-validation'
if len(sys.argv)==1:
    scratch.mkdir(exist_ok=False)
    with (scratch/'worker.log').open('wb') as stream:
        child=subprocess.Popen([sys.executable,__file__,'worker'],cwd=root,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
    print(json.dumps({'pid':child.pid,'directory':str(scratch)}));sys.exit(0)
try:
    archive.mkdir(exist_ok=False);records=[];environment=dict(os.environ,NODE_OPTIONS='--max-old-space-size=256')
    def run(name,argv):
        with (scratch/(name+'.stdout')).open('wb') as stdout,(scratch/(name+'.stderr')).open('wb') as stderr:
            result=subprocess.run(argv,cwd=root,env=environment,stdout=stdout,stderr=stderr)
        record={'name':name,'argv':argv,'exitCode':result.returncode}
        for label in ['stdout','stderr']:
            raw=(scratch/(name+'.'+label)).read_bytes();packed=gzip.compress(raw,mtime=0);filename=name+'.'+label+'.gz';(archive/filename).write_bytes(packed)
            record[label]={'file':filename,'rawSha256':hashlib.sha256(raw).hexdigest(),'archiveSha256':hashlib.sha256(packed).hexdigest(),'bytes':len(raw),'lines':len(raw.splitlines())}
        records.append(record);(archive/'records.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps({'finished':name,'exitCode':result.returncode}),flush=True)
        return result.returncode
    run('template-observations',['node','--test','--test-reporter=tap','--test-concurrency=1','rust/tests/web/issue-1580-template-observations.test.mjs'])
    run('version-admission',['node','--test','--test-reporter=tap','--test-concurrency=1','--test-name-pattern=version|base manifest','rust/tests/web/default-branch-build-policy.test.mjs'])
    run('rust-format',['rustfmt','--edition','2024','--check','rust/src/seed.rs','rust/src/protocol.rs'])
    run('projection',['node','scripts/translate-js-rust.mjs','--verify'])
    run('all-local-gates',['node','experiments/formal_ai_subagent/local-gates.mjs'])
    logs=[]
    for path in sorted((root/'experiments/formal_ai_subagent/sandboxes/gates').glob('*.log')):
        raw=path.read_bytes();packed=gzip.compress(raw,mtime=0);name=('gate-'+path.name+'.gz').replace('-docs.','-documentation.');(archive/name).write_bytes(packed)
        logs.append({'file':name,'rawSha256':hashlib.sha256(raw).hexdigest(),'archiveSha256':hashlib.sha256(packed).hexdigest(),'bytes':len(raw),'lines':len(raw.splitlines())})
    run('whitespace',['git','diff','--check'])
    outcome={'finished':True,'allPassed':all(r['exitCode']==0 for r in records),'checks':len(records),'gateStreams':logs,'fullIssueComplete':False,'independentAuthorshipCredit':0}
    (archive/'outcome.json').write_text(json.dumps(outcome,indent=2)+'\n');(scratch/'outcome.json').write_text(json.dumps(outcome,indent=2)+'\n');print(json.dumps({'finished':True,'allPassed':outcome['allPassed']}),flush=True)
except BaseException:
    traceback.print_exc();(scratch/'failure.json').write_text(json.dumps({'finished':False,'traceback':traceback.format_exc()})+'\n');sys.exit(1)
