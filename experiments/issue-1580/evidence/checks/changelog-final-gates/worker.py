from pathlib import Path
import subprocess,json,datetime
root=Path('/tmp/gh-issue-solver-1794900398048');out=Path('/tmp/issue-1580-research/changelog-final-gates')
args=['node','--max-old-space-size=256','experiments/formal_ai_subagent/local-gates.mjs']
with (out/'all-gates.stdout').open('w') as stdout,(out/'all-gates.stderr').open('w') as stderr:r=subprocess.run(args,cwd=root,stdout=stdout,stderr=stderr)
(out/'outcome.json').write_text(json.dumps({'argv':args,'exitCode':r.returncode,'finishedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'nativeExecutionClaim':False},indent=2)+'\n')
print('Completed',r.returncode,flush=True)
