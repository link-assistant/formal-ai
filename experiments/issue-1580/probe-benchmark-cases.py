"""Inspect six failing benchmark requests with a downloaded main binary; never build."""
import gzip, json, os, subprocess, time, urllib.request
from pathlib import Path

root=Path('/tmp/issue-1580-research')
sources={
 'humaneval':('https://raw.githubusercontent.com/openai/human-eval/6d43fb980f9fee3c892a914eda09951f772ad10d/data/HumanEval.jsonl.gz',True),
 'mbpp':('https://raw.githubusercontent.com/google-research/google-research/1fa17414f56c3703d5adb3818338b6e35e0fd550/mbpp/mbpp.jsonl',False),
 'gsm8k':('https://raw.githubusercontent.com/openai/grade-school-math/3101c7d5072418e28b9008a6636bde82a006892c/grade_school_math/data/test.jsonl',False),
}
prompts=[]
for name,(url,compressed) in sources.items():
    cache=root/(name+'-upstream.jsonl')
    if not cache.exists():
        with urllib.request.urlopen(url,timeout=30) as response: raw=response.read(4*1024*1024)
        cache.write_bytes(gzip.decompress(raw) if compressed else raw)
    rows=[json.loads(line) for line in cache.read_text().splitlines()]
    for index in {'humaneval':[1,4],'mbpp':[0,6],'gsm8k':[0,1]}[name]:
        row=rows[index]
        if name=='humaneval': prompt='Complete this Python function. Reply with the full implementation in a ```python code block.\n\n'+row['prompt']
        elif name=='mbpp': prompt=row['text']+'\nReply with the Python code in a ```python code block. It must pass these tests:\n'+'\n'.join(row['test_list'])
        else: prompt=row['question']
        prompts.append((name,index,prompt))
with (root/'benchmark-probe-server.log').open('w') as log:
    server=subprocess.Popen([str(root/'formal-ai-main'),'serve','--silent','--port','18983'],stdout=log,stderr=log,env={**os.environ,'FORMAL_AI_OFFLINE':'true'})
    try:
        for _ in range(50):
            try: urllib.request.urlopen('http://127.0.0.1:18983/',timeout=1).read(1); break
            except Exception:
                if server.poll() is not None: raise RuntimeError('server stopped')
                time.sleep(.1)
        for name,index,prompt in prompts:
            request=urllib.request.Request('http://127.0.0.1:18983/v1/chat/completions',data=json.dumps({'model':'formal-ai','temperature':0,'messages':[{'role':'user','content':prompt}]}).encode(),headers={'Content-Type':'application/json'})
            with urllib.request.urlopen(request,timeout=50) as response: value=json.load(response)
            (root/f'benchmark-probe-{name}-{index}.json').write_text(json.dumps({'prompt':prompt,'response':value},ensure_ascii=False,indent=2)+'\n')
            print(name,index,json.dumps(value.get('choices'),ensure_ascii=False),flush=True)
    finally:
        server.terminate()
        try: server.wait(5)
        except subprocess.TimeoutExpired: server.kill(); server.wait()
