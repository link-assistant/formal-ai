from pathlib import Path

p=Path('docs/case-studies/issue-1582/README.md')
s=p.read_text()
anchor='## Primary-source research and choices\n'
section='''## Client coverage and verification boundary

| Registry client | Context/read configuration path | Evidence boundary |
| --- | --- | --- |
| Codex | Served model catalog; live web-search setting in ephemeral and global TOML | Native generated-config regression added; execution pending. |
| T3 Code | Codex catalog plus inherited Claude context/output variables; Anthropic global profile | Fake-T3 numeric-variable regression added; installed UI not exercised. |
| OpenCode, VS Code and desktop variants; Agent | Existing numeric `limit.context` / `limit.output` seed paths retained | Existing model-metadata tests retained; remote-limit native probe includes Agent. |
| Claude Code | Documented input/output variables; named read permissions | Native launch/environment and global-preservation regressions added; execution pending. |
| Qwen Code | Numeric `model.generationConfig` limits in ephemeral and global JSON | Native remote-limit probe added; installed CLI not exercised. |
| Grok | Numeric model limits in current TOML, preserving older global JSON | Native remote-limit/undo probe added; installed CLI not exercised. |
| Aider | OpenAI model metadata in ephemeral and global homes | Native remote-limit probe added; metadata is not an enforced cap. |
| Gemini CLI | Model-specific output setting; upstream input fallback remains fixed | Bounded upstream-source probe reproduced the input limitation; unfinished. |
| Cursor | Existing MCP server integration retained | This adapter exposes Formal AI as a tool, not as Cursor's model provider; it does not configure the unrelated host model's context window. |

The passing Node probes establish actual server read transport, permission
answers and metadata behavior. Source/configuration inspection and unexecuted
native tests are not presented as installed-client end-to-end proof.

'''
assert anchor in s
p.write_text(s.replace(anchor,section+anchor))
