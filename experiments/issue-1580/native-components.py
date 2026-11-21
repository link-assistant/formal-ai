import json, os, subprocess, time, urllib.request
from pathlib import Path

root=Path('/tmp/issue-1580-research')
prompts={
    'zh-greeting': '你好',
    'zh-definition': 'Redis是什么？',
    'en-definition': 'What is Redis?',
    'zh-compound': '你好, Redis是什么？',
    'algebra-ci': 'Given y = 4 and x = 3, evaluate (x^2 + 2y).',
}
with (root/'native-components-server.log').open('w') as log:
    server=subprocess.Popen([str(root/'formal-ai-main'),'serve','--silent','--port','18982'], stdout=log, stderr=log, env={**os.environ,'FORMAL_AI_OFFLINE':'true'})
    try:
        for _ in range(50):
            try:
                urllib.request.urlopen('http://127.0.0.1:18982/',timeout=1).read(1)
                break
            except Exception:
                if server.poll() is not None: raise RuntimeError('server stopped')
                time.sleep(.1)
        for name,prompt in prompts.items():
            request=urllib.request.Request('http://127.0.0.1:18982/v1/chat/completions', data=json.dumps({'model':'formal-ai','temperature':0,'messages':[{'role':'user','content':prompt}]}).encode(), headers={'Content-Type':'application/json'})
            with urllib.request.urlopen(request,timeout=50) as response: value=json.load(response)
            (root/(name+'.json')).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
            print(name,json.dumps(value.get('choices'),ensure_ascii=False))
    finally:
        server.terminate()
        try: server.wait(5)
        except subprocess.TimeoutExpired: server.kill(); server.wait()
