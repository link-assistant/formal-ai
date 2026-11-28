"""Finite execution of the pinned C++ template helper and an isolated pagination patch."""
import gzip,json,os,sys,types
from pathlib import Path
sys.dont_write_bytecode=True
fixture=Path(__file__).parent/'fixtures/shared-runner-pagination/cpp-unacquired_jobs.py.gz'
source=gzip.decompress(fixture.read_bytes()).decode()
old='run_gh("api", f"repos/{repository}/check-runs/{job[\'id\']}/annotations?per_page=100")'
new='run_gh("api", "--paginate", "--slurp", f"repos/{repository}/check-runs/{job[\'id\']}/annotations?per_page=100")'
assert source.count(old)==1
candidate=source.replace(old,new).replace('for item in found]','for page in found for item in page]')
assert candidate!=source
if os.environ.get('ISSUE_1580_DISABLE_ANNOTATION_PAGINATION')=='1':candidate=source
rows=[]
for label,position,ran in [('first',0,False),('last-default-page',29,False),('second-default-page',30,False),('second-hundred-page',100,False),('absent',None,False),('real-code-failure',100,True)]:
 for patched,text in [(False,source),(True,candidate)]:
  module=types.ModuleType('runner_replay');exec(compile(text,str(fixture),'exec'),module.__dict__)
  calls=[];annotations=[{'message':'ordinary diagnostic'} for _ in range(101)]
  if position is not None:annotations[position]={'message':'The job was not acquired by Runner of type hosted even after multiple attempts'}
  job={'id':42,'name':'finite runner-loss case','status':'completed','conclusion':'failure' if ran else 'cancelled','runner_name':'actual runner' if ran else '', 'steps':[{'conclusion':'failure'}] if ran else []}
  def api(*args):
   calls.append(list(args));url=args[-1]
   if '/jobs?' in url:return json.dumps([{'jobs':[job]}])
   assert '/annotations?' in url
   return json.dumps([annotations[:100],annotations[100:]] if '--paginate' in args else annotations[:100])
  observed=module.find_unacquired('owner/repo','7','1',api)
  rows.append({'case':label,'candidate':patched,'expected':['finite runner-loss case'] if position is not None and not ran else [],'actual':observed,'calls':calls})
print(json.dumps(rows))
