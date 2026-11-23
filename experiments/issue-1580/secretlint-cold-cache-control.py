"""Replay the maintained lock-mismatch phase with a private empty npm cache."""
from pathlib import Path
import ast,hashlib,json,os,shutil,subprocess,sys,tempfile

repository=Path(__file__).resolve().parents[2]
output=Path(sys.argv[1]).resolve();output.mkdir(parents=True,exist_ok=True)
source=repository/'experiments/issue-1580/secretlint-locked-live.py'
text=source.read_text();tree=ast.parse(text,filename=str(source))
def starts_mismatch(statement):
 return isinstance(statement,ast.Assign) and any(isinstance(target,ast.Name) and target.id=='mismatched' for target in statement.targets)
body=next(statement.body for statement in tree.body if isinstance(statement,ast.Try)
 and any(starts_mismatch(child) for child in statement.body))
start=next(index for index,statement in enumerate(body) if isinstance(statement,ast.Assign)
 and any(isinstance(target,ast.Name) and target.id=='mismatched' for target in statement.targets))
end=next(index for index,statement in enumerate(body[start+1:],start+1)
 if isinstance(statement,ast.Assign) and isinstance(statement.value,ast.List)
 and [getattr(value,'value',None) for value in statement.value.elts[:2]]==['npm','audit'])
phase=ast.fix_missing_locations(ast.Module(body=body[start:end],type_ignores=[]))
directory=Path(tempfile.mkdtemp(prefix='formal-ai-secretlint-cold-cache-'))
records=[]
cache_initially_absent=all(not (directory/name).exists() for name in ['empty-cache','mismatched/cache'])
original=dict(os.environ)
try:
 os.environ.update(npm_config_registry=sys.argv[2],npm_config_cache=str(directory/'empty-cache'),
  npm_config_fetch_retries='0',npm_config_fetch_timeout='5000',npm_config_update_notifier='false')
 namespace=dict(repository=repository,output=output,directory=directory,records=records,
  Path=Path,json=json,shutil=shutil,subprocess=subprocess)
 exec(compile(phase,str(source),'exec'),namespace)
 result={'records':records,'probeSha256':hashlib.sha256(source.read_bytes()).hexdigest(),
  'cacheInitiallyAbsent':cache_initially_absent,
  'nodeModulesCreated':(directory/'mismatched/node_modules').exists(),
  'actualNpm':subprocess.run(['npm','--version'],capture_output=True,text=True).stdout.strip()}
 (output/'control-result.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps(result))
finally:
 os.environ.clear();os.environ.update(original);shutil.rmtree(directory)
