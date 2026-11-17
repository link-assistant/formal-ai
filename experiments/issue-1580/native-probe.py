import json, subprocess, time, urllib.request
from pathlib import Path

root = Path('/tmp/issue-1580-research')
with (root / 'native-probe-server.log').open('w') as log:
    server = subprocess.Popen(['/usr/local/bin/formal-ai', 'serve', '--silent', '--agent-mode', '--port', '18981'], stdout=log, stderr=log)
    try:
        for attempt in range(50):
            try:
                urllib.request.urlopen('http://127.0.0.1:18981/health', timeout=1).read()
                break
            except Exception:
                if server.poll() is not None: raise RuntimeError('server stopped')
                time.sleep(.1)
        prompts = {
            'g102': 'Create the file abbreviations.test.mjs. It imports node:assert/strict as assert and node:test as test, and imports wordsOfIdentifier from «./scripts/measure-abbreviations.mjs». Write one test:\n1. test «a declared name splits at underscores»: assert.deepEqual(wordsOfIdentifier(«parseArgs»), [«parse», «args»]).',
            'prime': 'Pick a prime number between 14 and 18',
            'zh': '你好, Redis是什么？',
            'algebra': 'If x = 3 and y = 4, what is x*x + 2*y?',
        }
        for name, prompt in prompts.items():
            tools = [{'type':'function','function':{'name':name,'parameters':{'type':'object','properties':{}}}} for name in ['bash','edit','glob','grep','list','read','write']] if name == 'g102' else []
            request = urllib.request.Request('http://127.0.0.1:18981/v1/chat/completions', data=json.dumps({'model':'formal-ai','messages':[{'role':'user','content':prompt}], 'tools':tools}).encode(), headers={'Content-Type':'application/json'})
            with urllib.request.urlopen(request, timeout=50) as response:
                value = json.load(response)
            (root / ('native-probe-' + name + '.json')).write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n')
            print(name, json.dumps(value.get('choices'), ensure_ascii=False))
    finally:
        server.terminate()
        try: server.wait(5)
        except subprocess.TimeoutExpired:
            server.kill(); server.wait()
