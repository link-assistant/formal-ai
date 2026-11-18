"""Remove duplicate JSON prefix parsing; express multilingual test rows explicitly."""
from pathlib import Path
p=Path('rust/src/client_integrations/global_config.rs')
s=p.read_text()
start=s.index('    for setting in settings {',s.index('fn merge_json_settings('))
end=s.index('    Ok(format!',start)
s=s[:start]+'''    for (path, setting_value) in settings {
        set_json_setting(&mut base, path, setting_value, context, true)?;
    }
'''+s[end:]
s=s.replace('set_json_setting(&mut value, path, setting_value, context)?;', 'set_json_setting(&mut value, path, setting_value, context, false)?;')
start=s.index('fn set_json_setting(')
end=s.index('fn invalid_typed_json_setting_error',start)
part=s[start:end].replace('    context: &RenderContext,\n', '    context: &RenderContext,\n    merge_objects: bool,\n',1)
part=part.replace('    object.insert(last.clone(), setting);', '''    if let Some(previous) = object.get_mut(last).filter(|_| merge_objects) {
        merge_json_value(previous, setting);
    } else {
        object.insert(last.clone(), setting);
    }''')
s=s[:start]+part+s[end:];p.write_text(s)
p=Path('rust/tests/web/issue-1582-defaults.test.mjs')
s=p.read_text().replace("  ['en','How do I enable internet access?'],", "  {language:'en',prompt:'How do I enable internet access?'},").replace("  ['ru','Как включить доступ к интернету?'],", "  {language:'ru',prompt:'Как включить доступ к интернету?'},").replace("  ['hi','इंटरनेट की अनुमति कैसे दें?'],", "  {language:'hi',prompt:'इंटरनेट की अनुमति कैसे दें?'},").replace("  ['zh','如何启用互联网访问？'],", "  {language:'zh',prompt:'如何启用互联网访问？'},").replace("  ['es','¿Cómo habilitar el acceso a internet?'],", "  {language:'es',prompt:'¿Cómo habilitar el acceso a internet?'},").replace('for(const [language,prompt] of questions)', 'for(const {language,prompt} of questions)')
p.write_text(s)
# Preserve the first full round before replacing its per-gate logs.
import shutil
local=Path('docs/case-studies/issue-1582/raw-data/local')
if not (local/'gates-round-1').exists():shutil.copytree(local/'gates',local/'gates-round-1')
if not (local/'gates-round-2').exists():shutil.copytree('experiments/formal_ai_subagent/sandboxes/gates',local/'gates-round-2')
