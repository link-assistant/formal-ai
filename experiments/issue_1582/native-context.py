"""Supplied Grok adapter and native regressions; zero authorship credit."""
from pathlib import Path

p=Path('rust/src/client_integrations/global_config.rs');s=p.read_text()
s=s.replace('''            &render_template(path, context),
            &render_template(value, context),''','''            path,
            &render_template(value, context),
            context,''')
s=s.replace('''    value: &str,
) -> Result<(), Box<dyn Error>> {
    let parts = dotted_path''','''    value: &str,
    context: &RenderContext,
) -> Result<(), Box<dyn Error>> {
    let parts = dotted_path''',1)
s=s.replace('''        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();''','''        .filter(|part| !part.is_empty())
        .map(|part| render_template(part, context))
        .collect::<Vec<_>>();''',1)
s=s.replace('''    let parent = table_at_path_mut(table, parents);
    parent[*last] = toml_value(value);''','''    let parent_keys = parents.iter().map(String::as_str).collect::<Vec<_>>();
    let parent = table_at_path_mut(table, &parent_keys);
    parent[last.as_str()] = if let Some(integer) = value.strip_prefix("integer:") {
        toml_value(integer.parse::<i64>()?)
    } else { toml_value(value) };''')
p.write_text(s)
p=Path('data/seed/client-integrations.lino');s=p.read_text();a=s.index('  tool "grok"');b=s.index('  tool "aider"',a);part=s[a:b]
entries=[('model','{model}'),('base_url','{endpoint_base_url}'),('api_key','{api_key}'),('api_backend','chat_completions'),('context_window','integer:{context_window_tokens}'),('max_completion_tokens','integer:{max_output_tokens}')]
part=part.replace('      temp_home_env "HOME"','      temp_home_env "HOME"\n      temp_home_config_path ".grok/config.toml"\n'+'\n'.join('      temp_home_toml_set "model.{model}.'+key+'='+value+'"' for key,value in entries))
part=part.rstrip()+'\n      companion ".grok/config.toml"\n        kind "toml"\n        backup_suffix ".formal-ai.bak"\n'+'\n'.join('        toml_set "model.{model}.'+key+'='+value+'"' for key,value in entries)+'\n'
p.write_text(s[:a]+part+s[b:])

p=Path('rust/tests/integration/with_formal_ai.rs');s=p.read_text();s=s.replace('mod model_metadata;', 'mod model_metadata;\nmod issue_1582;')
s=s.replace('    echo "ANTHROPIC_BASE_URL=$ANTHROPIC_BASE_URL"','''    echo "ANTHROPIC_BASE_URL=$ANTHROPIC_BASE_URL"
    echo "CLAUDE_CODE_MAX_CONTEXT_TOKENS=$CLAUDE_CODE_MAX_CONTEXT_TOKENS"
    echo "CLAUDE_CODE_MAX_OUTPUT_TOKENS=$CLAUDE_CODE_MAX_OUTPUT_TOKENS"''')
s=s.replace('''                assert!(captured.contains("arg[0]=--model"));
                assert!(captured.contains("arg[1]=formal-ai"));
            }
            "qwen"''','''                assert!(captured.contains("--allowedTools"));
                assert!(captured.contains("Read,Glob,Grep,WebFetch,WebSearch"));
                assert!(captured_args(&captured).windows(2).any(|args| args == ["--model", "formal-ai"]));
            }
            "qwen"''')
p.write_text(s)

p=Path('rust/tests/integration/with_formal_ai/issue_1582.rs')
p.write_text('''//! Actual native wrapper regressions for #1582; no vendor CLI installation.
use super::*;
use std::io::Read as _;

#[test]
fn every_configurable_client_uses_the_remote_servers_limits_and_preserves_denies() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(home.join(".claude")).expect("home");
    std::fs::write(home.join(".claude/settings.json"), r#"{"permissions":{"allow":["Read(custom)"],"deny":["WebFetch(domain:private.example)"]},"theme":"dark"}"#).expect("existing permissions");
    let listener = TcpListener::bind("127.0.0.1:0").expect("metadata server");
    listener.set_nonblocking(true).expect("nonblocking");
    let address = listener.local_addr().expect("address");
    let server = std::thread::spawn(move || {
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(10) {
            if let Ok((mut stream, _)) = listener.accept() {
                stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).expect("read bound");
                let mut request = [0; 8192];
                let count = stream.read(&mut request).expect("metadata request");
                assert!(String::from_utf8_lossy(&request[..count]).starts_with("GET /v1/models "));
                let body = r#"{"data":[{"id":"formal-ai","context_window_tokens":1234567,"max_output_tokens":32768}]}"#;
                write!(stream,"HTTP/1.1 200 OK\\r\\nContent-Type: application/json\\r\\nContent-Length: {}\\r\\nConnection: close\\r\\n\\r\\n{body}",body.len()).expect("response");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the wrapper did not discover the remote context window");
    });
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with","--global","--all","--base-url",&format!("http://{address}")])
        .env("HOME",&home).env_remove("FORMAL_AI_LOG_STDERR").output().expect("configure all clients");
    server.join().expect("metadata worker");
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(),"routine diagnostics must stay out of the harness");
    let profile = std::fs::read_to_string(home.join(".profile")).expect("profile");
    assert!(profile.contains("CLAUDE_CODE_MAX_CONTEXT_TOKENS=\\"1234567\\""));
    assert!(profile.contains("CLAUDE_CODE_MAX_OUTPUT_TOKENS=\\"32768\\""));
    for (file, pointer) in [
        (".qwen/settings.json","/model/generationConfig/contextWindowSize"),
        (".aider.model.metadata.json","/openai~1formal-ai/max_input_tokens"),
        (".config/link-assistant-agent/opencode.json","/provider/formalai/models/formal-ai/limit/context"),
    ] {
        let value:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(home.join(file)).expect(file)).expect("JSON");
        assert_eq!(value.pointer(pointer),Some(&serde_json::json!(1234567)),"{file}: {value}");
    }
    let settings:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(home.join(".claude/settings.json")).expect("Claude settings")).expect("JSON");
    assert_eq!(settings["permissions"]["deny"],serde_json::json!(["WebFetch(domain:private.example)"]));
    for rule in ["Read(custom)","Read","Glob","Grep","WebFetch","WebSearch"] {
        assert!(settings["permissions"]["allow"].as_array().expect("allow array").contains(&serde_json::json!(rule)));
    }
    assert_eq!(settings["theme"],"dark");
    let grok=std::fs::read_to_string(home.join(".grok/config.toml")).expect("Grok config").parse::<toml_edit::DocumentMut>().expect("TOML");
    assert_eq!(grok["model"]["formal-ai"]["context_window"].as_integer(),Some(1234567));
    let undo=Command::new(env!("CARGO_BIN_EXE_formal-ai")).args(["with","--global","--undo","--all"]).env("HOME",&home).output().expect("undo");
    assert!(undo.status.success());
    let restored:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(home.join(".claude/settings.json")).expect("restore")).expect("JSON");
    assert_eq!(restored["permissions"]["allow"],serde_json::json!(["Read(custom)"]));
    assert!(!home.join(".grok/config.toml").exists());
    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn claude_ephemeral_receives_a_real_context_override_and_read_only_permissions() {
    let dir=tmpdir();let home=dir.join("home");let bin=dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");std::fs::create_dir_all(&bin).expect("bin");
    write_fake_cli(&bin,"claude");let capture=dir.join("capture");
    let output=run_with_capture(&home,&bin,&capture,&["with","claude","hi"]);
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty());
    let capture=std::fs::read_to_string(capture).expect("capture");
    let window=capture.lines().find_map(|line|line.strip_prefix("CLAUDE_CODE_MAX_CONTEXT_TOKENS=")).expect("context override").parse::<u64>().expect("numeric context");
    assert!(window>200_000,"a resource-derived limit replaces Claude's unknown-model guess");
    assert!(capture.contains("Read,Glob,Grep,WebFetch,WebSearch"));
    assert!(!capture.contains("bypassPermissions"));
    std::fs::remove_dir_all(dir).expect("cleanup");
}
''')
