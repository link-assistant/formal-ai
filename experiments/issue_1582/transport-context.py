"""Supplied repair recipe for #1582; executing it earns zero authorship credit."""
from pathlib import Path
import json

def edit(name, before, after):
    path = Path(name)
    text = path.read_text()
    assert before in text, (name, before[:100])
    path.write_text(text.replace(before, after))

path = Path('js/runtime/read-access.mjs')
path.write_text(path.read_text() + '''
// The serving worker can read HTTP(S) sources without acquiring write access.
export function createReadOnlyFetch({env=process.env,fetchImpl=globalThis.fetch}={}) {
  return (url,options={})=>{
    if(!readNetworkAllowed(env))return Promise.reject(new Error('read_network_denied'));
    const parsed=new URL(url);
    if(!['http:','https:'].includes(parsed.protocol))return Promise.reject(new Error('read_network_protocol'));
    if(!['GET','HEAD'].includes(String(options.method??'GET').toUpperCase()))return Promise.reject(new Error('read_network_method'));
    return fetchImpl(url,options);
  };
}
''')
edit('js/server/worker-host.mjs', '{ reorderModules = null, provenance=false } = {}', '{ reorderModules = null, provenance=false, networkFetch=null } = {}')
edit('js/server/worker-host.mjs', 'fetch: (url) => {\n      const relative', '''fetch: (url, options) => {
      const address = new URL(String(url), 'http://localhost/worker/formal_ai_worker.js');
      if (address.origin !== 'http://localhost' && networkFetch) return networkFetch(url, options);
      const relative''')
edit('js/server/worker-host.mjs', 'constructor(overrides = {},{provenance=false}={}) {', 'constructor(overrides = {},{provenance=false,networkFetch=null}={}) {\n    this.networkFetch = networkFetch;')
edit('js/server/worker-host.mjs', 'createWorkerContext(this.overrides,{provenance})', 'createWorkerContext(this.overrides,{provenance,networkFetch:this.networkFetch})')
edit('js/server/main.mjs', "import { WorkerHost } from './worker-host.mjs';", "import { WorkerHost } from './worker-host.mjs';\nimport {createReadOnlyFetch} from '../runtime/read-access.mjs';")
edit('js/server/main.mjs', 'new WorkerHost({ formalAiProverHost: createProverHost({ path: env.PATH }) })', 'new WorkerHost({ formalAiProverHost: createProverHost({ path: env.PATH }), navigator:{onLine:readNetworkAllowed(env)} }, {networkFetch:createReadOnlyFetch({env})})')
edit('js/server/main.mjs', 'import {createReadOnlyFetch}', 'import {createReadOnlyFetch,readNetworkAllowed}')
edit('rust/src/solver_handlers/web_requests.rs', '    if is_page_formalization_prompt(prompt, &normalize_prompt(prompt)) {', '    let offline = offline || !crate::cli_env::read_network_allowed();\n    if is_page_formalization_prompt(prompt, &normalize_prompt(prompt)) {')
edit('rust/src/cli_env.rs', 'debug traces, live fetch', 'debug traces')

edit('rust/src/client_integrations.rs', 'codex_model_catalog(&context.model)', 'codex_model_catalog(&context)')
edit('rust/src/client_integrations/global_config.rs', 'codex_model_catalog(&context.model)', 'codex_model_catalog(&context)')
edit('rust/src/client_integrations.rs', '''fn codex_model_catalog(model: &str) -> Result<String, Box<dyn Error>> {
    let context = ContextCapacity::current()?;''', '''fn codex_model_catalog(render: &RenderContext) -> Result<String, Box<dyn Error>> {
    let context = ContextCapacity::current()?;
    let (window, _) = render.model_limits();
    let window: u64 = window.parse()?;
    let model = &render.model;''')
edit('rust/src/client_integrations.rs', '"context_window": context.context_window_tokens,\n            "max_context_window": context.context_window_tokens,', '"context_window": window,\n            "max_context_window": window,')

# Array append settings preserve existing permission rules and model overrides.
edit('rust/src/client_integrations/global_config.rs', '''    let overlay = json_settings_value(settings, context)?;
    merge_json_value(&mut base, overlay);''', '''    for (path, value) in settings {
        set_json_setting(&mut base, path, value, context)?;
    }''')
edit('rust/src/client_integrations/global_config.rs', '''    let setting = if let Some(literal) = rendered.strip_prefix("json:") {''', '''    if let Some(literal) = rendered.strip_prefix("json+:") {
        let additions: Vec<Value> = serde_json::from_str(literal)
            .map_err(|error| invalid_typed_json_setting_error(&rendered, &error))?;
        let target = object.entry(last.clone()).or_insert_with(|| Value::Array(Vec::new()));
        let target = target.as_array_mut().ok_or("JSON append setting conflicts with a scalar value")?;
        for value in additions {
            if !target.contains(&value) { target.push(value); }
        }
        return Ok(());
    }
    let setting = if let Some(literal) = rendered.strip_prefix("json:") {''')
# The recursive merge helper has no callers once settings apply directly.
p=Path('rust/src/client_integrations/global_config.rs');s=p.read_text()
a=s.index('fn merge_json_value(');b=s.index('fn merge_shell_env_config(',a);s=s[:a]+s[b:];p.write_text(s)

p=Path('data/seed/client-integrations.lino');s=p.read_text()
def tool_edit(tool,before,after):
    global s
    a=s.index('  tool "'+tool+'"');b=s.find('\n  tool ',a+1);b=len(s) if b<0 else b
    part=s[a:b];assert before in part,(tool,before);s=s[:a]+part.replace(before,after)+s[b:]
tool_edit('claude', '      env "ANTHROPIC_BASE_URL={endpoint_base_url}"', '''      env "ANTHROPIC_BASE_URL={endpoint_base_url}"
      env "CLAUDE_CODE_MAX_CONTEXT_TOKENS={context_window_tokens}"
      env "CLAUDE_CODE_MAX_OUTPUT_TOKENS={max_output_tokens}"
      arg "--allowedTools"
      arg "Read,Glob,Grep,WebFetch,WebSearch"''')
tool_edit('claude', '      shell_env "ANTHROPIC_BASE_URL={endpoint_base_url}"', '''      shell_env "ANTHROPIC_BASE_URL={endpoint_base_url}"
      shell_env "CLAUDE_CODE_MAX_CONTEXT_TOKENS={context_window_tokens}"
      shell_env "CLAUDE_CODE_MAX_OUTPUT_TOKENS={max_output_tokens}"
      companion ".claude/settings.json"
        kind "json"
        backup_suffix ".formal-ai.bak"
        json_set "permissions.allow=json+:[\\"Read\\",\\"Glob\\",\\"Grep\\",\\"WebFetch\\",\\"WebSearch\\"]"''')
tool_edit('qwen', '      temp_home_json_set "tools.approvalMode=auto-edit"', '''      temp_home_json_set "tools.approvalMode=auto-edit"
      temp_home_json_set "model.generationConfig.contextWindowSize=json:{context_window_tokens}"
      temp_home_json_set "model.generationConfig.maxTokens=json:{max_output_tokens}"''')
tool_edit('qwen', '      shell_env "OPENAI_MODEL={model}"', '''      shell_env "OPENAI_MODEL={model}"
      companion ".qwen/settings.json"
        kind "json"
        backup_suffix ".formal-ai.bak"
        json_set "model.generationConfig.contextWindowSize=json:{context_window_tokens}"
        json_set "model.generationConfig.maxTokens=json:{max_output_tokens}"''')
override=json.dumps([{'match':{'model':'{model}'},'modelConfig':{'generateContentConfig':{'maxOutputTokens':'OUTPUT'}}}],separators=(',',':')).replace('"OUTPUT"','{max_output_tokens}').replace('"','\\"')
tool_edit('gemini', '      temp_home_json_set "security.auth.selectedType={google_auth_type}"', '      temp_home_json_set "security.auth.selectedType={google_auth_type}"\n      temp_home_json_set "modelConfigs.overrides=json+:'+override+'"')
tool_edit('gemini', '        json_set "security.auth.selectedType={google_auth_type}"', '        json_set "security.auth.selectedType={google_auth_type}"\n        json_set "modelConfigs.overrides=json+:'+override+'"')
tool_edit('aider', '      temp_home_env "HOME"', '''      temp_home_env "HOME"
      temp_home_config_path ".aider.model.metadata.json"
      temp_home_json_set "{model_selector}.max_input_tokens=json:{context_window_tokens}"
      temp_home_json_set "{model_selector}.max_output_tokens=json:{max_output_tokens}"
      temp_home_json_set "{model_selector}.max_tokens=json:{max_output_tokens}"
      temp_home_json_set "{model_selector}.litellm_provider=openai"
      temp_home_json_set "{model_selector}.mode=chat"''')
tool_edit('aider', '      shell_env "OPENAI_API_BASE={endpoint_base_url}"', '''      shell_env "OPENAI_API_BASE={endpoint_base_url}"
      companion ".aider.model.metadata.json"
        kind "json"
        backup_suffix ".formal-ai.bak"
        json_set "{model_selector}.max_input_tokens=json:{context_window_tokens}"
        json_set "{model_selector}.max_output_tokens=json:{max_output_tokens}"
        json_set "{model_selector}.max_tokens=json:{max_output_tokens}"
        json_set "{model_selector}.litellm_provider=openai"
        json_set "{model_selector}.mode=chat"''')
p.write_text(s)

p=Path('data/seed/multilingual-responses-agentic.lino');s=p.read_text()
replacements={
'Live fetching is off by default; enable it explicitly to populate the replayable source cache.':'Read-only retrieval is enabled by default. Check connectivity, provider errors, and explicit offline or deny settings; ask how to enable internet access for harness permission controls.',
'Получение из сети по умолчанию отключено; включите его явно, чтобы заполнить воспроизводимый кэш источников.':'Чтение из сети включено по умолчанию. Проверьте соединение, ошибки провайдера и явные запреты; спросите, как включить доступ к интернету, для настройки разрешений оболочки.',
'लाइव फ़ेच डिफ़ॉल्ट रूप से बंद है; पुनः चलाए जा सकने वाले स्रोत कैश को भरने के लिए इसे स्पष्ट रूप से चालू करें।':'नेटवर्क से पढ़ना डिफ़ॉल्ट रूप से चालू है। कनेक्शन, प्रदाता की त्रुटियाँ और स्पष्ट ऑफ़लाइन प्रतिबंध जाँचें; हार्नेस अनुमतियों के लिए पूछें कि इंटरनेट की अनुमति कैसे दें।',
'实时抓取默认关闭；请明确启用它以填充可重放的来源缓存。':'默认允许只读网络访问。请检查连接、提供商错误和显式离线或拒绝设置；询问如何启用互联网访问以查看工具权限设置。',
'La obtención en vivo está desactivada por defecto; actívala explícitamente para llenar la caché de fuentes reproducible.':'La lectura de red está habilitada por defecto. Comprueba la conexión, los errores del proveedor y los bloqueos explícitos; pregunta cómo habilitar el acceso a internet para configurar los permisos de la herramienta.'}
for before,after in replacements.items(): assert before in s;s=s.replace(before,after)
p.write_text(s)
