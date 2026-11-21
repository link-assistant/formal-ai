"""Supplied permission guidance repair; execution is not autonomous authorship."""
from pathlib import Path
questions={
 'en':['how do i enable internet access','how to enable internet','how can i enable internet','enable internet access','grant internet access','how do i grant permissions','allow read only internet access'],
 'ru':['как включить доступ к интернету','как включить интернет','разрешить доступ к интернету','разрешаю доступ к интернету','как выдать разрешения'],
 'hi':['इंटरनेट की अनुमति कैसे दें','इंटरनेट कैसे सक्षम करें','इंटरनेट की अनुमति दें'],
 'zh':['如何启用互联网访问','如何开启网络访问','允许访问互联网','如何授予权限'],
 'es':['cómo habilitar el acceso a internet','cómo activar internet','permitir acceso a internet','cómo conceder permisos'],
}
p=Path('data/seed/meanings-feature-capability.lino')
s=p.read_text()+'''  internet_access_help
    defined-by action
    role internet_access_help
'''
for lang,phrases in questions.items():
 s+='    lexeme '+lang+' '+' '.join('"'+phrase+'"' for phrase in phrases)+'\n'
p.write_text(s)
responses={
'en':'Read-only internet retrieval is enabled by default. If your client blocks a tool, approve its permission request; in Claude Code use `/permissions` to allow Read, Glob, Grep, WebFetch and WebSearch. Existing ask/deny rules still apply. For an explicitly offline Formal AI server, unset FORMAL_AI_OFFLINE and FORMAL_AI_NETWORK=denied, then restart with `FORMAL_AI_LIVE_FETCH=1 formal-ai with claude`. A chat message cannot override an operator restriction. Writes and command execution use the client’s permission controls. If retrieval still fails, check connectivity, proxy settings and provider availability; inspect the log file or enable FORMAL_AI_LOG_STDERR=1 for diagnostics.',
'ru':'Чтение интернет-источников включено по умолчанию. Если клиент блокирует инструмент, подтвердите запрос разрешения; в Claude Code откройте `/permissions` и разрешите Read, Glob, Grep, WebFetch и WebSearch. Существующие правила ask/deny сохраняются. Если сервер Formal AI явно переведён в автономный режим, уберите FORMAL_AI_OFFLINE и FORMAL_AI_NETWORK=denied и перезапустите: `FORMAL_AI_LIVE_FETCH=1 formal-ai with claude`. Сообщение в чате не отменяет ограничения оператора. Запись и выполнение команд управляются разрешениями клиента. Если запросы не проходят, проверьте соединение, прокси и доступность источников; изучите файл журнала или включите FORMAL_AI_LOG_STDERR=1 для диагностики.',
'hi':'केवल पढ़ने के लिए इंटरनेट स्रोत डिफ़ॉल्ट रूप से उपलब्ध हैं। क्लाइंट किसी टूल को रोके तो उसका अनुमति अनुरोध स्वीकार करें; Claude Code में `/permissions` से Read, Glob, Grep, WebFetch और WebSearch की अनुमति दें। मौजूदा ask/deny नियम लागू रहते हैं। Formal AI सर्वर स्पष्ट रूप से ऑफ़लाइन हो तो FORMAL_AI_OFFLINE और FORMAL_AI_NETWORK=denied हटाकर `FORMAL_AI_LIVE_FETCH=1 formal-ai with claude` से पुनः शुरू करें। चैट संदेश ऑपरेटर के प्रतिबंध नहीं बदलता। लिखने और कमांड चलाने के लिए क्लाइंट की अनुमति सेटिंग उपयोग करें। अनुरोध विफल हों तो कनेक्शन, प्रॉक्सी और स्रोत की उपलब्धता जाँचें; लॉग फ़ाइल देखें या निदान के लिए FORMAL_AI_LOG_STDERR=1 सक्षम करें।',
'zh':'默认允许只读访问互联网来源。如果客户端阻止工具，请批准其权限请求；在 Claude Code 中用 `/permissions` 允许 Read、Glob、Grep、WebFetch 和 WebSearch。现有 ask/deny 规则仍然生效。如果 Formal AI 服务器被显式设为离线，取消 FORMAL_AI_OFFLINE 和 FORMAL_AI_NETWORK=denied，然后用 `FORMAL_AI_LIVE_FETCH=1 formal-ai with claude` 重启。聊天消息无法覆盖操作者的限制。写入和执行命令由客户端权限设置控制。如果检索仍失败，请检查连接、代理和来源可用性；查看日志文件或启用 FORMAL_AI_LOG_STDERR=1 进行诊断。',
'es':'La lectura de fuentes de internet está habilitada por defecto. Si el cliente bloquea una herramienta, aprueba su solicitud de permiso; en Claude Code usa `/permissions` para permitir Read, Glob, Grep, WebFetch y WebSearch. Se conservan las reglas ask/deny existentes. Si Formal AI está explícitamente sin conexión, elimina FORMAL_AI_OFFLINE y FORMAL_AI_NETWORK=denied y reinicia con `FORMAL_AI_LIVE_FETCH=1 formal-ai with claude`. Un mensaje no anula restricciones del operador. La escritura y ejecución de comandos usan los permisos del cliente. Si falla la consulta, comprueba conexión, proxy y disponibilidad de las fuentes; consulta el archivo de registro o activa FORMAL_AI_LOG_STDERR=1 para diagnosticar.',
}
p=Path('data/seed/multilingual-responses-capabilities.lino'); s=p.read_text()
for lang,text in responses.items():
 s+=f'  response response_internet_access_help_{lang}\n    intent internet_access_help\n    language {lang}\n    text "{text}"\n'
p.write_text(s)
p=Path('js/worker/formal_ai_worker_capabilities_and_runtime_rules.js');s=p.read_text().replace('function tryFeatureCapabilityStatus(prompt, normalized, language, preferences) {','''function tryFeatureCapabilityStatus(prompt, normalized, language, preferences) {
  if (lexiconMentionsRole("internet_access_help", normalized)) return {
    intent: "capabilities", content: capabilityResponse("internet_access_help", language),
    confidence: 1, evidence: ["handler:capabilities", "permission:guidance", `language:${language}`],
  };''');p.write_text(s)
p=Path('rust/src/solver_handlers/feature_capability.rs');s=p.read_text().replace('    let language = detect_language(prompt);','''    let language = detect_language(prompt);
    if seed::lexicon().mentions_role("internet_access_help", normalized) {
        log.append("permission:guidance", "internet_access_help");
        let body = capability_response("internet_access_help", language.slug(), &[]);
        return Some(finalize_simple(prompt, log, "capabilities", "response:capabilities", &body, 1.0));
    }''',1);p.write_text(s)
