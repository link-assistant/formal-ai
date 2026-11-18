"""Preserve the existing overlay behavior when a parent was a scalar."""
from pathlib import Path
p=Path('rust/src/client_integrations/global_config.rs')
s=p.read_text();start=s.index('fn set_json_setting(');end=s.index('fn invalid_typed_json_setting_error',start)
part=s[start:end].replace('    for part in parents {\n', '''    for part in parents {
        if merge_objects && !current.is_object() {
            *current = Value::Object(serde_json::Map::new());
        }
''',1).replace('    let object = current\n        .as_object_mut()', '''    if merge_objects && !current.is_object() {
        *current = Value::Object(serde_json::Map::new());
    }
    let object = current
        .as_object_mut()''',1)
s=s[:start]+part+s[end:];p.write_text(s)
