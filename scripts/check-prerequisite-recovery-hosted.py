#!/usr/bin/env python3
"""Explicit hosted-only real archive check; never invoked by local node:test."""
import json, os, pathlib, shutil, subprocess, tempfile
repository=pathlib.Path(__file__).resolve().parents[1]
grant=json.loads((repository/'data/benchmarks/hive-prerequisite-workspace-grant.json').read_text())
publisher={'program':'kotlinc','source_id':'kotlin_official','host':'kotlinlang.org','documentation':'https://kotlinlang.org/docs/command-line.html'}
request={'program':'kotlinc','publisher':publisher,'dependencies':['java'],'grant':grant}
if os.environ.get('GITHUB_ACTIONS')!='true': raise SystemExit('hosted-only guard: no local installer execution')
with tempfile.TemporaryDirectory(prefix='formal-ai-prerequisite-hosted-') as temporary:
    workspace=pathlib.Path(temporary); binaries=workspace/'host-bin'; binaries.mkdir()
    for directory in os.environ['PATH'].split(os.pathsep):
        parent=pathlib.Path(directory)
        if not parent.is_dir(): continue
        for executable in parent.iterdir():
            target=binaries/executable.name
            if executable.name=='kotlinc' or target.exists() or target.is_symlink() or not executable.is_file() or not os.access(executable,os.X_OK): continue
            target.symlink_to(executable.resolve())
    environment={**os.environ,'PATH':str(binaries)}
    assert shutil.which('kotlinc',path=environment['PATH']) is None
    script=repository/'data/meta/prerequisite-installer.py'
    completed=subprocess.run([shutil.which('python3'),str(script),json.dumps(request)],cwd=workspace,env=environment,check=False,timeout=480)
    assert completed.returncode==0,'real hosted discovery/install/probe failed'
    prefix=(workspace/'.formal-ai/toolchains/active-kotlinc.bin').read_text().strip()
    assert pathlib.Path(prefix).is_relative_to(workspace)
    source=workspace/'Independent.kt'; source.write_text('fun main() { println("Hello, World!") }\n')
    subprocess.run([prefix+'/kotlinc',str(source),'-include-runtime','-d',str(workspace/'independent.jar')],cwd=workspace,env=environment,check=True,timeout=90)
    observed=subprocess.run(['java','-jar',str(workspace/'independent.jar')],cwd=workspace,env=environment,check=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30)
    assert observed.stdout==b'Hello, World!\n' and observed.stderr==b'', 'original independent one-line oracle'
    print(json.dumps({'schema':'formal-ai-hosted-prerequisite-result-v1','runtime':'Kotlin','stdout':'Hello, World!\\n','actualInstall':True,'threeClientAcceptance':False}))
