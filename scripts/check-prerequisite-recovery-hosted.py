#!/usr/bin/env python3
"""Explicit hosted-only real archive check; never invoked by local node:test."""
import json, os, pathlib, shutil, subprocess, sys, tempfile
repository=pathlib.Path(__file__).resolve().parents[1]
if os.environ.get('GITHUB_ACTIONS')!='true': raise SystemExit('hosted-only guard: no local installer execution')
resolver=(repository/'js/server/prerequisite-grant.mjs').as_uri()
program='import {resolvePrerequisiteGrantText} from '+json.dumps(resolver)+';process.stdout.write(resolvePrerequisiteGrantText(process.env.FORMAL_AI_PREREQUISITE_GRANT_JSON??null,process.env.FORMAL_AI_INSTALL_GRANT)??"null");'
resolved=subprocess.run(['node','--input-type=module','-e',program],env=os.environ,check=True,stdout=subprocess.PIPE,text=True)
grant=json.loads(resolved.stdout)
assert grant is not None, 'hosted caller must explicitly grant workspace recovery'
language=sys.argv[1] if len(sys.argv)>1 else 'kotlin'
cases={'kotlin':{'program':'kotlinc','publisher':{'program':'kotlinc','source_id':'kotlin_official','host':'kotlinlang.org','documentation':'https://kotlinlang.org/docs/command-line.html'},'name':'Independent.kt','source':'fun main() { println("Hello, World!") }\n'},'scala':{'program':'scalac','publisher':{'program':'scalac','source_id':'scala_official','host':'api.github.com','documentation':'https://api.github.com/repos/scala/scala3/releases/latest'},'name':'Independent.scala','source':'object Independent extends App { println("Hello, World!") }\n'}}
case=cases[language];program=case['program'];publisher=case['publisher']
request={'program':program,'publisher':publisher,'dependencies':['java'],'grant':grant}
if os.environ.get('GITHUB_ACTIONS')!='true': raise SystemExit('hosted-only guard: no local installer execution')
with tempfile.TemporaryDirectory(prefix='formal-ai-prerequisite-hosted-') as temporary:
    workspace=pathlib.Path(temporary); binaries=workspace/'host-bin'; binaries.mkdir()
    for directory in os.environ['PATH'].split(os.pathsep):
        parent=pathlib.Path(directory)
        if not parent.is_dir(): continue
        for executable in parent.iterdir():
            target=binaries/executable.name
            if executable.name==program or target.exists() or target.is_symlink() or not executable.is_file() or not os.access(executable,os.X_OK): continue
            target.symlink_to(executable.resolve())
    environment={**os.environ,'PATH':str(binaries)}
    assert shutil.which(program,path=environment['PATH']) is None
    script=repository/'data/meta/prerequisite-installer.py'
    completed=subprocess.run([shutil.which('python3'),str(script),json.dumps(request)],cwd=workspace,env=environment,check=False,timeout=480)
    assert completed.returncode==0,'real hosted discovery/install/probe failed'
    prefix=(workspace/('.formal-ai/toolchains/active-'+program+'.bin')).read_text().strip()
    assert pathlib.Path(prefix).is_relative_to(workspace)
    source=workspace/case['name']; source.write_text(case['source'])
    if language=='kotlin':
        compile_command=[prefix+'/'+program,str(source),'-include-runtime','-d',str(workspace/'independent.jar')]
        run_command=['java','-jar',str(workspace/'independent.jar')]
    else:
        compile_command=[prefix+'/'+program,'-d',str(workspace),str(source)]
        help_result=subprocess.run([prefix+'/scala','-help'],cwd=workspace,env=environment,check=False,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30)
        help_text=(help_result.stdout+help_result.stderr).decode('utf-8',errors='replace')
        print(json.dumps({'phase':'runner-help','runtime':language,'exit_code':help_result.returncode,'output':help_text}),flush=True)
        assert help_result.returncode==0,'runner help failed'
        if '--main-class' in help_text:
            run_command=[prefix+'/scala','run','--classpath',str(workspace),'--main-class','Independent']
        else:
            assert '-classpath' in help_text or '-cp' in help_text,'unrecognized JVM runner command grammar'
            run_command=[prefix+'/scala','-classpath',str(workspace),'Independent']
    subprocess.run(compile_command,cwd=workspace,env=environment,check=True,timeout=90)
    observed=subprocess.run(run_command,cwd=workspace,env=environment,check=False,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30)
    if observed.returncode:
        sys.stderr.buffer.write(observed.stderr)
        sys.stderr.buffer.flush()
    observed.check_returncode()
    assert observed.stdout==b'Hello, World!\n' and observed.stderr==b'', 'original independent one-line oracle'
    print(json.dumps({'schema':'formal-ai-hosted-prerequisite-result-v1','runtime':language,'stdout':'Hello, World!\\n','actualInstall':True,'threeClientAcceptance':False}))
