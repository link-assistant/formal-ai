import os,pathlib,json,subprocess
root='/Users/konard/Code/Archive/link-assistant/formal-ai';lock=pathlib.Path('/private/tmp/formal-ai-pr1188-commit.lock');lock.mkdir()
try:
 env=dict(os.environ,GIT_INDEX_FILE='/private/tmp/pr1188-ci-T4078/private-index')
 def git(*a):return subprocess.check_output(['git',*a],cwd=root,env=env)
 head=git('rev-parse','HEAD').strip();git('read-tree',head.decode())
 paths=[]
 packet='experiments/formal_ai_subagent/evidence/ci-1188/archive-selection-correction-0f701';paths+=sorted(str(p.relative_to(root)) for p in pathlib.Path(root,packet).rglob('*') if p.is_file())
 subprocess.run(['git','--literal-pathspecs','add','-f','--pathspec-from-file=-','--pathspec-file-nul'],input=b'\0'.join(p.encode() for p in paths)+b'\0',cwd=root,env=env,check=True)
 staged=set(git('diff','--cached','--name-only','-z').decode().split('\0'))-{''};assert staged==set(paths),(staged,set(paths));assert git('rev-parse','HEAD').strip()==head
 git('commit','-m','docs(ci): restore exact workflow and debug repair evidence')
 new=git('rev-parse','HEAD').decode().strip();subprocess.run(['git','read-tree',new],cwd=root,check=True)
 pathlib.Path('/private/tmp/pr1188-ci-T4078/commit-sha.txt').write_text(new+'\n');print(new)
finally:lock.rmdir()
