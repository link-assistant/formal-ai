"""Prefer the harness's read-only live-search tool over wider shell networking."""
from pathlib import Path
p=Path('data/seed/client-integrations.lino');s=p.read_text()
s=s.replace('      temp_home_toml_set "model_catalog_json={model_catalog_path}"', '      temp_home_toml_set "model_catalog_json={model_catalog_path}"\n      temp_home_toml_set "web_search=live"')
s=s.replace('      toml_set "model_catalog_json={model_catalog_path}"', '      toml_set "model_catalog_json={model_catalog_path}"\n      toml_set "web_search=live"')
p.write_text(s)
p=Path('rust/tests/integration/with_formal_ai/issue_1582.rs');s=p.read_text()
needle='    let catalog: serde_json::Value ='
s=s.replace(needle, '''    let codex = std::fs::read_to_string(home.join(".codex/config.toml"))
        .expect("Codex config").parse::<toml_edit::DocumentMut>().expect("TOML");
    assert_eq!(codex["web_search"].as_str(), Some("live"));
'''+needle)
p.write_text(s)
p=Path('docs/case-studies/issue-1582/README.md');s=p.read_text().replace('Read-only transport does not grant arbitrary shell execution or filesystem', '''Codex/T3 Codex configuration also selects the dedicated `web_search = "live"`
tool. Upstream's read-only permission profile restricts command networking;
live search provides the read-only source path through Formal AI without
granting a shell unrestricted network access. The native adapter regression
checks the actual generated setting. Archived official source is pinned in
`raw-data/research/codex-source-sha.txt`.

Read-only transport does not grant arbitrary shell execution or filesystem''')
s=s.replace('## Primary-source research and choices\n', '''## Primary-source research and choices

- [Codex web-search configuration](https://developers.openai.com/codex/config-reference)
  supports `web_search = "live"`; [permission profiles](https://learn.chatgpt.com/docs/permissions)
  document read-only filesystem profiles and separate network controls. The
  dedicated read tool avoids granting arbitrary network-capable shell commands.
''')
p.write_text(s)
