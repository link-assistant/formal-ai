from pathlib import Path
import json, subprocess

root=Path('docs/case-studies/issue-1582/raw-data/research')
def gh(args):
    return subprocess.check_output(['gh', *args])
sha=json.loads(gh(['api', 'repos/pingdotgg/t3code/commits/main']))['sha']
(root/'t3-source-sha.txt').write_text(sha+'\n')
old=root/'t3-ClaudeAdapterV2.ts'
if old.exists():old.rename(old.with_suffix('.ts.error'))
for source in ['docs/user/providers-claude.md', 'apps/server/src/provider/ClaudeModelCatalog.ts']:
    result=subprocess.run(['gh', 'api', f'repos/pingdotgg/t3code/contents/{source}?ref={sha}', '-H', 'Accept: application/vnd.github.raw+json'],capture_output=True)
    (root/('t3-'+Path(source).name+('' if result.returncode==0 else '.error'))).write_bytes(result.stdout if result.returncode==0 else result.stderr)
    print(source, result.returncode, len(result.stdout))
    if result.returncode==0 and source.endswith('.ts'):
        lines=result.stdout.decode().splitlines()
        for i,line in enumerate(lines):
            if any(term in line for term in ['function claudeContextWindow', 'MAX_CONTEXT_TOKENS', 'maxTokens: claudeContextWindow', 'process.env']):
                print('\n'.join(f'{n+1}: {lines[n]}' for n in range(max(0,i-2),min(len(lines),i+16))))
