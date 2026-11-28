from pathlib import Path
import json,os,types

source=(Path(__file__).parent/'fixtures/rust-runner-recovery/unacquired-jobs.py').read_text()
old='''            found = gh_json('api', f"repos/{repository}/check-runs/{job['id']}/annotations")
            annotations[job['id']] = [item.get('message', '') for item in found or []]'''
new='''            pages = gh_json('api', '--paginate', '--slurp',
                            f"repos/{repository}/check-runs/{job['id']}/annotations?per_page=100")
            found = [item for page in pages or [] for item in page]
            annotations[job['id']] = [item.get('message', '') for item in found]'''
assert source.count(old)==1
candidate=source if os.environ.get('ISSUE_1580_DISABLE_ANNOTATION_PAGINATION')=='1' else source.replace(old,new)
results=[]
cases=[('first-annotation',0,'',[]),('default-page-boundary',29,'',[]),
       ('second-default-page',30,'',[]),('second-full-page',100,'',[]),
       ('no-acquisition-message',None,'',[]),
       ('actual-code-failure',100,'runner-1',[{'name':'test','conclusion':'failure'}])]
for implementation,text in [('baseline',source),('candidate',candidate)]:
    module=types.ModuleType('unacquired_'+implementation)
    exec(compile(text,'pinned-unacquired-jobs.py','exec'),module.__dict__)
    for label,position,runner,steps in cases:
        job={'id':314,'name':'native-tests','conclusion':'failure','runner_name':runner,'steps':steps}
        messages=[{'message':'unrelated diagnostic '+str(i)} for i in range(101)]
        if position is not None:messages[position]={'message':'The job was not acquired by Runner of type hosted even after multiple attempts'}
        calls=[]
        def fake(*args):
            calls.append(list(args));endpoint=args[-1]
            if '/jobs?' in endpoint:return [{'jobs':[job]}]
            assert '/annotations' in endpoint
            page_size=100 if 'per_page=100' in endpoint else 30
            if '--paginate' in args:
                assert '--slurp' in args
                return [messages[i:i+page_size] for i in range(0,len(messages),page_size)]
            return messages[:page_size]
        module.gh_json=fake
        names=module.fetch_unacquired('example/repository',81,1)
        results.append({'implementation':implementation,'case':label,'observed':names,'calls':calls})
print(json.dumps(results))
