"""Fix review findings without dropping established JSON merge behavior."""
from pathlib import Path
p=Path('data/meta/dependency-direction.lino')
s=p.read_text();assert 'outward-references-ceiling 34' in s
p.write_text(s.replace('outward-references-ceiling 34','outward-references-ceiling 33'))
p=Path('js/agentic/node-host.mjs')
p.write_text(p.read_text().replace('/** `cli_env::flag_enabled`: a true spelling of the variable. */\n\n',''))
p=Path('data/meta/worker-line-budget/formal_ai_worker_obligations.lino')
p.write_text(p.read_text().replace('The existing source contains the requirement-evidence identifier handling; this PR does not change that module.', 'The base origin/main source is also 488 lines; this PR does not change that module.'))
p=Path('data/seed/client-integrations.lino')
p.write_text(p.read_text().replace('{model_selector}.max_tokens=json:{max_output_tokens}', '{model_selector}.max_tokens=json:{context_window_tokens}'))
p=Path('rust/src/client_integrations/global_config.rs')
s=p.read_text()
old='''    for (path, value) in settings {
        set_json_setting(&mut base, path, value, context)?;
    }
'''
new='''    for setting in settings {
        if setting.1.starts_with("json+:") {
            set_json_setting(&mut base, &setting.0, &setting.1, context)?;
        } else {
            let overlay = json_settings_value(std::slice::from_ref(setting), context)?;
            merge_json_value(&mut base, overlay);
        }
    }
'''
assert old in s;s=s.replace(old,new)
helper='''fn merge_json_value(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                match base_map.get_mut(&key) {
                    Some(base_value) => merge_json_value(base_value, overlay_value),
                    None => { base_map.insert(key, overlay_value); }
                }
            }
        }
        (base_value, overlay_value) => *base_value = overlay_value,
    }
}

'''
assert 'fn merge_json_value(' not in s
s=s.replace('fn merge_shell_env_config(',helper+'fn merge_shell_env_config(')
p.write_text(s)
