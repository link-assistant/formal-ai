from pathlib import Path

p=Path('data/seed/client-integrations.lino')
s=p.read_text()
start=s.index('  tool "t3code"');end=s.index('  tool "opencode"')
part=s[start:end]
part=part.replace('      env "ANTHROPIC_BASE_URL={anthropic_endpoint_base_url}"', '      env "ANTHROPIC_BASE_URL={anthropic_endpoint_base_url}"\n      env "CLAUDE_CODE_MAX_CONTEXT_TOKENS={context_window_tokens}"\n      env "CLAUDE_CODE_MAX_OUTPUT_TOKENS={max_output_tokens}"')
part=part.replace('      shell_env "ANTHROPIC_BASE_URL={endpoint_base_url}"', '      shell_env "ANTHROPIC_BASE_URL={endpoint_base_url}"\n      shell_env "CLAUDE_CODE_MAX_CONTEXT_TOKENS={context_window_tokens}"\n      shell_env "CLAUDE_CODE_MAX_OUTPUT_TOKENS={max_output_tokens}"')
s=s[:start]+part+s[end:];p.write_text(s)
p=Path('rust/tests/integration/with_formal_ai.rs')
s=p.read_text()
start=s.index('            "t3code" => {')
insertion=s.index('\n', start)+1
s=s[:insertion]+'                assert!(captured.contains("CLAUDE_CODE_MAX_CONTEXT_TOKENS="));\n                assert!(captured.contains("CLAUDE_CODE_MAX_OUTPUT_TOKENS="));\n'+s[insertion:]
p.write_text(s)
p=Path('docs/case-studies/issue-1582/README.md')
s=p.read_text().replace('Read-only transport does not grant', 'T3 Code also launches Claude Code, so its ephemeral environment and persistent\nAnthropic profile receive the same documented context/output overrides.\n[Its provider documentation](https://github.com/pingdotgg/t3code/blob/main/docs/user/providers-claude.md)\nconfirms that integration; the pinned documentation and catalog source are\nretained under `raw-data/research/`. Compaction settings are not substituted for\na model context window. The existing fake-T3 fixture records both overrides;\nan installed T3 UI session has not been exercised here.\n\nRead-only transport does not grant')
p.write_text(s)
p=Path('experiments/issue_1582/pull-request.md')
s=p.read_text().replace('Claude gets named read permissions and its documented context/output environment overrides.', 'Claude gets named read permissions and its documented context/output environment overrides; the T3 Claude path receives the same overrides.')
p.write_text(s)
