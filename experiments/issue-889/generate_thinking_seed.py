#!/usr/bin/env python3
"""Generate the localized thinking-step seed data for issue #889.

`src/thinking.rs` used to hold the English prose for every thinking step, so the
CLI, the OpenAI/Anthropic APIs and the Telegram bot narrated their reasoning in
English no matter which language the answer was written in. The prose now lives
in `data/seed/multilingual-responses-thinking.lino` (per-step templates) and
`data/seed/multilingual-responses-thinking-narrative.lino` (the human headline
plus the localized language names), one record per (intent, language).

The English column reproduces the sentences `src/thinking.rs` used to format so
the migration is behaviour-preserving for English; the ru/zh/hi columns are the
strings the browser already ships in `src/web/i18n-catalog-messages.lino`
(`message.thinkingStep.*`), so both halves of the product speak with one voice;
the es column is new, because the browser catalog has no Spanish locale yet
while `data/seed/languages.lino` registers `es` as a supported language.

Usage:
    python3 experiments/issue-889/generate_thinking_seed.py
"""

from __future__ import annotations

import pathlib

LANGUAGES = ["en", "ru", "hi", "zh", "es"]

# intent -> {language: text}
STEPS: dict[str, dict[str, str]] = {
    "thinking_step_impulse": {
        "en": 'Read the request: \\"{prompt}\\".',
        "ru": "Прочитать запрос: «{prompt}».",
        "hi": "अनुरोध पढ़ें: “{prompt}”।",
        "zh": "读取请求：“{prompt}”。",
        "es": "Leer la solicitud: «{prompt}».",
    },
    "thinking_step_impulse_plain": {
        "en": "Read the incoming request.",
        "ru": "Прочитать входящий запрос.",
        "hi": "आने वाला अनुरोध पढ़ें।",
        "zh": "读取传入的请求。",
        "es": "Leer la solicitud entrante.",
    },
    "thinking_step_detect_language": {
        "en": "Detect the request language: {language}.",
        "ru": "Определить язык запроса: {language}.",
        "hi": "अनुरोध की भाषा पहचानें: {language}।",
        "zh": "检测请求语言：{language}。",
        "es": "Detectar el idioma de la solicitud: {language}.",
    },
    "thinking_step_resolve_response_language": {
        "en": "Plan to answer in {language}.",
        "ru": "Планировать ответ на языке: {language}.",
        "hi": "{language} में उत्तर देने की योजना बनाएँ।",
        "zh": "计划用 {language} 回答。",
        "es": "Planear la respuesta en {language}.",
    },
    "thinking_step_formalize": {
        "en": "Formalize the request as {article} {task} task.",
        "ru": "Формализовать запрос как задачу «{task}».",
        "hi": "अनुरोध को «{task}» कार्य के रूप में औपचारिक रूप दें।",
        "zh": "将请求形式化为 {task} 任务。",
        "es": "Formalizar la solicitud como una tarea de tipo «{task}».",
    },
    "thinking_step_formalize_plain": {
        "en": "Formalize the request into a symbolic tuple.",
        "ru": "Формализовать запрос в символьный кортеж.",
        "hi": "अनुरोध को प्रतीकात्मक टपल में औपचारिक रूप दें।",
        "zh": "将请求形式化为符号元组。",
        "es": "Formalizar la solicitud en una tupla simbólica.",
    },
    "thinking_step_formalize_resolved": {
        "en": "Resolve the request to {entity}.",
        "ru": "Свести запрос к сущности «{entity}».",
        "hi": "अनुरोध को {entity} तक सुलझाएँ।",
        "zh": "将请求解析为 {entity}。",
        "es": "Resolver la solicitud a la entidad «{entity}».",
    },
    "thinking_step_formalize_resolved_plain": {
        "en": "Resolve the request to a concrete entity.",
        "ru": "Свести запрос к конкретной сущности.",
        "hi": "अनुरोध को किसी ठोस इकाई तक सुलझाएँ।",
        "zh": "将请求解析为具体实体。",
        "es": "Resolver la solicitud a una entidad concreta.",
    },
    "thinking_step_clarify_formalization": {
        "en": "Ask for clarification between {options}.",
        "ru": "Запросить уточнение между: {options}.",
        "hi": "इनके बीच स्पष्टीकरण माँगें: {options}।",
        "zh": "在以下选项之间请求澄清：{options}。",
        "es": "Pedir una aclaración entre: {options}.",
    },
    "thinking_step_clarify_formalization_plain": {
        "en": "Ask for clarification because the request was ambiguous.",
        "ru": "Запросить уточнение, потому что запрос был неоднозначным.",
        "hi": "स्पष्टीकरण माँगें, क्योंकि अनुरोध अस्पष्ट था।",
        "zh": "请求澄清，因为该请求含义不明确。",
        "es": "Pedir una aclaración porque la solicitud era ambigua.",
    },
    "thinking_step_dispatch_handler": {
        "en": "Route to the {route} handler.",
        "ru": "Направить к обработчику «{route}».",
        "hi": "«{route}» हैंडलर को भेजें।",
        "zh": "路由到 {route} 处理器。",
        "es": "Enrutar al manejador «{route}».",
    },
    "thinking_step_dispatch_handler_plain": {
        "en": "Route the request to a handler.",
        "ru": "Направить запрос к обработчику.",
        "hi": "अनुरोध को किसी हैंडलर को भेजें।",
        "zh": "将请求路由到处理器。",
        "es": "Enrutar la solicitud a un manejador.",
    },
    "thinking_step_route_attempt": {
        "en": "Try the {route} approach.",
        "ru": "Попробовать подход «{route}».",
        "hi": "«{route}» तरीका आज़माएँ।",
        "zh": "尝试 {route} 方案。",
        "es": "Probar el enfoque «{route}».",
    },
    "thinking_step_route_attempt_plain": {
        "en": "Try the next candidate approach.",
        "ru": "Попробовать следующий подход-кандидат.",
        "hi": "अगला संभावित तरीका आज़माएँ।",
        "zh": "尝试下一个候选方案。",
        "es": "Probar el siguiente enfoque candidato.",
    },
    "thinking_step_match_rule": {
        "en": "Match the {rule} rule.",
        "ru": "Применить правило «{rule}».",
        "hi": "«{rule}» नियम से मिलान करें।",
        "zh": "匹配 {rule} 规则。",
        "es": "Aplicar la regla «{rule}».",
    },
    "thinking_step_match_rule_plain": {
        "en": "Match a known rule.",
        "ru": "Применить известное правило.",
        "hi": "किसी ज्ञात नियम से मिलान करें।",
        "zh": "匹配一条已知规则。",
        "es": "Aplicar una regla conocida.",
    },
    "thinking_step_compute": {
        "en": "Compute {expression}.",
        "ru": "Вычислить {expression}.",
        "hi": "{expression} की गणना करें।",
        "zh": "计算 {expression}。",
        "es": "Calcular {expression}.",
    },
    "thinking_step_compute_plain": {
        "en": "Compute the result.",
        "ru": "Вычислить результат.",
        "hi": "परिणाम की गणना करें।",
        "zh": "计算结果。",
        "es": "Calcular el resultado.",
    },
    "thinking_step_compute_engine": {
        "en": "Evaluate with the {engine}.",
        "ru": "Вычислить с помощью {engine}.",
        "hi": "{engine} से मूल्यांकन करें।",
        "zh": "用 {engine} 求值。",
        "es": "Evaluar con {engine}.",
    },
    "thinking_step_compute_engine_plain": {
        "en": "Evaluate with the calculator.",
        "ru": "Вычислить с помощью калькулятора.",
        "hi": "कैलकुलेटर से मूल्यांकन करें।",
        "zh": "用计算器求值。",
        "es": "Evaluar con la calculadora.",
    },
    "thinking_step_compute_expression": {
        "en": "Reduce the expression {expression}.",
        "ru": "Упростить выражение {expression}.",
        "hi": "व्यंजक {expression} को सरल करें।",
        "zh": "化简表达式 {expression}。",
        "es": "Reducir la expresión {expression}.",
    },
    "thinking_step_compute_steps": {
        "en": "Apply {count} reduction step(s).",
        "ru": "Применить {count} шаг(ов) упрощения.",
        "hi": "{count} सरलीकरण चरण लागू करें।",
        "zh": "应用 {count} 个化简步骤。",
        "es": "Aplicar {count} paso(s) de reducción.",
    },
    "thinking_step_lookup_fact": {
        "en": "Look up {fact}.",
        "ru": "Найти факт: {fact}.",
        "hi": "{fact} देखें।",
        "zh": "查找 {fact}。",
        "es": "Buscar el dato: {fact}.",
    },
    "thinking_step_lookup_fact_plain": {
        "en": "Look up the relevant fact.",
        "ru": "Найти подходящий факт.",
        "hi": "संबंधित तथ्य देखें।",
        "zh": "查找相关事实。",
        "es": "Buscar el dato correspondiente.",
    },
    "thinking_step_invoke_tool": {
        "en": "Use the {tool} capability.",
        "ru": "Использовать возможность «{tool}».",
        "hi": "{tool} क्षमता का उपयोग करें।",
        "zh": "使用 {tool} 能力。",
        "es": "Usar la capacidad «{tool}».",
    },
    "thinking_step_invoke_tool_plain": {
        "en": "Use an available capability.",
        "ru": "Использовать доступную возможность.",
        "hi": "किसी उपलब्ध क्षमता का उपयोग करें।",
        "zh": "使用一项可用的能力。",
        "es": "Usar una capacidad disponible.",
    },
    "thinking_step_rule_verification": {
        "en": "Verify the result against the {rule} rule.",
        "ru": "Проверить результат по правилу «{rule}».",
        "hi": "परिणाम को «{rule}» नियम से जाँचें।",
        "zh": "用 {rule} 规则验证结果。",
        "es": "Verificar el resultado con la regla «{rule}».",
    },
    "thinking_step_rule_verification_plain": {
        "en": "Verify the result against the rules.",
        "ru": "Проверить результат по правилам.",
        "hi": "परिणाम को नियमों से जाँचें।",
        "zh": "用规则验证结果。",
        "es": "Verificar el resultado con las reglas.",
    },
    "thinking_step_policy_refusal": {
        "en": "Decline the request under the {policy} policy.",
        "ru": "Отклонить запрос по политике «{policy}».",
        "hi": "«{policy}» नीति के तहत अनुरोध अस्वीकार करें।",
        "zh": "依据 {policy} 政策拒绝该请求。",
        "es": "Rechazar la solicitud según la política «{policy}».",
    },
    "thinking_step_policy_refusal_plain": {
        "en": "Decline the request under the safety policy.",
        "ru": "Отклонить запрос по политике безопасности.",
        "hi": "सुरक्षा नीति के तहत अनुरोध अस्वीकार करें।",
        "zh": "依据安全政策拒绝该请求。",
        "es": "Rechazar la solicitud según la política de seguridad.",
    },
    "thinking_step_rule_construction": {
        "en": "Build a local behavior rule.",
        "ru": "Построить локальное правило поведения.",
        "hi": "स्थानीय व्यवहार नियम बनाएँ।",
        "zh": "构建本地行为规则。",
        "es": "Construir una regla de comportamiento local.",
    },
    "thinking_step_coreference_binding": {
        "en": "Resolve what the follow-up refers to.",
        "ru": "Определить, к чему относится уточняющий вопрос.",
        "hi": "समझें कि आगे का प्रश्न किसकी बात कर रहा है।",
        "zh": "确定追问指向的对象。",
        "es": "Determinar a qué se refiere la pregunta de seguimiento.",
    },
    "thinking_step_modifier_detection": {
        "en": "Detect modifiers in the request.",
        "ru": "Обнаружить модификаторы в запросе.",
        "hi": "अनुरोध में संशोधक पहचानें।",
        "zh": "检测请求中的修饰条件。",
        "es": "Detectar los modificadores de la solicitud.",
    },
    "thinking_step_program_plan": {
        "en": "Plan the program: {plan}.",
        "ru": "Спланировать программу: {plan}.",
        "hi": "कार्यक्रम की योजना बनाएँ: {plan}।",
        "zh": "规划程序：{plan}。",
        "es": "Planificar el programa: {plan}.",
    },
    "thinking_step_program_plan_plain": {
        "en": "Plan the requested program.",
        "ru": "Спланировать запрошенную программу.",
        "hi": "माँगे गए कार्यक्रम की योजना बनाएँ।",
        "zh": "规划请求的程序。",
        "es": "Planificar el programa solicitado.",
    },
    "thinking_step_scan_memory": {
        "en": "Search memory for {term}.",
        "ru": "Искать в памяти: {term}.",
        "hi": "स्मृति में खोजें: {term}।",
        "zh": "在记忆中搜索：{term}。",
        "es": "Buscar en la memoria: {term}.",
    },
    "thinking_step_scan_memory_plain": {
        "en": "Search memory for relevant facts.",
        "ru": "Искать в памяти подходящие факты.",
        "hi": "स्मृति में संबंधित तथ्य खोजें।",
        "zh": "在记忆中搜索相关事实。",
        "es": "Buscar en la memoria los datos pertinentes.",
    },
    "thinking_step_user_context": {
        "en": "Apply available context: {context}.",
        "ru": "Применить доступный контекст: {context}.",
        "hi": "उपलब्ध संदर्भ लागू करें: {context}।",
        "zh": "应用可用上下文：{context}。",
        "es": "Aplicar el contexto disponible: {context}.",
    },
    "thinking_step_user_context_plain": {
        "en": "Apply the available context.",
        "ru": "Применить доступный контекст.",
        "hi": "उपलब्ध संदर्भ लागू करें।",
        "zh": "应用可用的上下文。",
        "es": "Aplicar el contexto disponible.",
    },
    "thinking_step_deformalize": {
        "en": 'Compose the answer: \\"{answer}\\".',
        "ru": "Составить ответ: «{answer}».",
        "hi": "उत्तर लिखें: “{answer}”।",
        "zh": "撰写答案：“{answer}”。",
        "es": "Redactar la respuesta: «{answer}».",
    },
    "thinking_step_deformalize_plain": {
        "en": "Compose the answer in natural language.",
        "ru": "Составить ответ на естественном языке.",
        "hi": "उत्तर सहज भाषा में लिखें।",
        "zh": "用自然语言撰写答案。",
        "es": "Redactar la respuesta en lenguaje natural.",
    },
    "thinking_step_http_chat": {
        "en": "Exchange a request with the configured endpoint.",
        "ru": "Обменяться запросом с настроенной конечной точкой.",
        "hi": "निर्धारित सिरे के साथ अनुरोध का आदान-प्रदान करें।",
        "zh": "与配置的端点交换一次请求。",
        "es": "Intercambiar una solicitud con el endpoint configurado.",
    },
    "thinking_step_agent_plan": {
        "en": "Add an agent task: {task}.",
        "ru": "Добавить задачу агента: {task}.",
        "hi": "एजेंट कार्य जोड़ें: {task}।",
        "zh": "添加代理任务：{task}。",
        "es": "Añadir una tarea del agente: {task}.",
    },
    "thinking_step_agent_plan_plain": {
        "en": "Extend the agent plan.",
        "ru": "Дополнить план агента.",
        "hi": "एजेंट की योजना आगे बढ़ाएँ।",
        "zh": "扩展代理计划。",
        "es": "Ampliar el plan del agente.",
    },
    "thinking_step_memory": {
        "en": "Update the local memory bundle.",
        "ru": "Обновить локальный набор памяти.",
        "hi": "स्थानीय स्मृति बंडल अद्यतन करें।",
        "zh": "更新本地记忆包。",
        "es": "Actualizar el paquete de memoria local.",
    },
    "thinking_step_extract_term": {
        "en": "Extract the search term.",
        "ru": "Извлечь поисковый термин.",
        "hi": "खोज शब्द निकालें।",
        "zh": "提取搜索词。",
        "es": "Extraer el término de búsqueda.",
    },
    "thinking_step_group_by_conversation": {
        "en": "Group matching memories by conversation.",
        "ru": "Сгруппировать найденные воспоминания по разговорам.",
        "hi": "मिलती हुई स्मृतियों को बातचीत के हिसाब से समूहित करें।",
        "zh": "按对话分组匹配的记忆。",
        "es": "Agrupar los recuerdos coincidentes por conversación.",
    },
    "thinking_step_fallback": {
        "en": "Fall back to the general unknown-request strategy.",
        "ru": "Перейти к общей стратегии для неизвестного запроса.",
        "hi": "सामान्य अज्ञात-अनुरोध रणनीति पर लौटें।",
        "zh": "回退到通用的未知请求策略。",
        "es": "Recurrir a la estrategia general para solicitudes desconocidas.",
    },
    "thinking_step_unnamed": {
        "en": "step",
        "ru": "шаг",
        "hi": "चरण",
        "zh": "步骤",
        "es": "paso",
    },
    "thinking_step_generic": {
        "en": "{label}: {detail}.",
        "ru": "{label}: {detail}.",
        "hi": "{label}: {detail}.",
        "zh": "{label}：{detail}。",
        "es": "{label}: {detail}.",
    },
    "thinking_step_generic_plain": {
        "en": "{label}.",
        "ru": "{label}.",
        "hi": "{label}।",
        "zh": "{label}。",
        "es": "{label}.",
    },
}

NARRATIVES: dict[str, dict[str, str]] = {
    "thinking_trace_heading": {
        "en": "Thinking",
        "ru": "Размышления",
        "hi": "सोच-विचार",
        "zh": "思考",
        "es": "Razonamiento",
    },
    "thinking_narrative_greeting": {
        "en": "You said hello, so I greeted you back.",
        "ru": "Вы поздоровались — и я поздоровался в ответ.",
        "hi": "आपने नमस्ते कहा, तो मैंने भी नमस्ते कहा।",
        "zh": "你打了招呼，所以我也回了招呼。",
        "es": "Me saludaste, así que te devolví el saludo.",
    },
    "thinking_narrative_wellbeing": {
        "en": "You asked how I'm doing, so I told you and offered to help.",
        "ru": "Вы спросили, как у меня дела, — я ответил и предложил помощь.",
        "hi": "आपने पूछा कि मैं कैसा हूँ, तो मैंने जवाब दिया और मदद की पेशकश की।",
        "zh": "你问我过得怎么样，所以我作了回答并主动提供帮助。",
        "es": "Me preguntaste cómo estoy, así que te respondí y me ofrecí a ayudar.",
    },
    "thinking_narrative_assistant_free_time": {
        "en": "You asked what I get up to, so I answered in a friendly way and offered to help.",
        "ru": "Вы спросили, чем я занимаюсь, — я дружелюбно ответил и предложил помощь.",
        "hi": "आपने पूछा कि मैं क्या करता हूँ, तो मैंने दोस्ताना जवाब दिया और मदद की पेशकश की।",
        "zh": "你问我平时做些什么，所以我友好地回答并主动提供帮助。",
        "es": "Me preguntaste a qué me dedico, así que te respondí con amabilidad y me ofrecí a ayudar.",
    },
    "thinking_narrative_farewell": {
        "en": "You said goodbye, so I wished you well in return.",
        "ru": "Вы попрощались — и я пожелал вам всего доброго в ответ.",
        "hi": "आपने विदा ली, तो मैंने भी शुभकामनाएँ दीं।",
        "zh": "你道了别，所以我也回以祝福。",
        "es": "Te despediste, así que te deseé lo mejor en respuesta.",
    },
    "thinking_narrative_gratitude": {
        "en": "You thanked me, so I acknowledged it warmly.",
        "ru": "Вы поблагодарили меня — и я тепло на это откликнулся.",
        "hi": "आपने धन्यवाद कहा, तो मैंने गर्मजोशी से जवाब दिया।",
        "zh": "你向我道谢，所以我热情地作了回应。",
        "es": "Me diste las gracias, así que respondí con cordialidad.",
    },
    "thinking_narrative_identity": {
        "en": "You asked about my name or who I am, so I answered from what I remember of our chat.",
        "ru": "Вы спросили про моё имя или кто я, — я ответил, опираясь на то, что помню из нашего разговора.",
        "hi": "आपने मेरे नाम या पहचान के बारे में पूछा, तो मैंने बातचीत से जो याद है उसके आधार पर जवाब दिया।",
        "zh": "你问起我的名字或身份，所以我根据这次对话中记得的内容作了回答。",
        "es": "Preguntaste por mi nombre o por quién soy, así que respondí con lo que recuerdo de nuestra conversación.",
    },
    "thinking_narrative_calculation": {
        "en": "This was a calculation, so I worked it out step by step and checked the result.",
        "ru": "Это было вычисление — я решил его шаг за шагом и проверил результат.",
        "hi": "यह एक गणना थी, तो मैंने इसे चरण दर चरण हल किया और नतीजा जाँचा।",
        "zh": "这是一道计算题，所以我一步步算出并核对了结果。",
        "es": "Era un cálculo, así que lo resolví paso a paso y comprobé el resultado.",
    },
    "thinking_narrative_fact_lookup": {
        "en": "You asked for a fact, so I looked it up and reported what I found.",
        "ru": "Вы попросили факт — я нашёл его и сообщил, что выяснил.",
        "hi": "आपने एक तथ्य पूछा, तो मैंने उसे ढूँढकर बताया।",
        "zh": "你想了解一个事实，所以我查找后把结果告诉了你。",
        "es": "Pediste un dato, así que lo busqué y te conté lo que encontré.",
    },
    "thinking_narrative_translation": {
        "en": "You asked for a translation, so I converted the text and returned it.",
        "ru": "Вы попросили перевод — я преобразовал текст и вернул его.",
        "hi": "आपने अनुवाद माँगा, तो मैंने पाठ बदलकर लौटाया।",
        "zh": "你需要翻译，所以我转换了文本并返回给你。",
        "es": "Pediste una traducción, así que convertí el texto y te lo devolví.",
    },
    "thinking_narrative_web": {
        "en": "You pointed me at the web, so I fetched what you needed and summarized it.",
        "ru": "Вы указали мне на веб — я получил нужное и кратко изложил.",
        "hi": "आपने मुझे वेब पर भेजा, तो मैंने ज़रूरी जानकारी लाकर सारांश दिया।",
        "zh": "你让我去查网络，所以我获取了所需内容并作了概括。",
        "es": "Me remitiste a la web, así que obtuve lo que necesitabas y lo resumí.",
    },
    "thinking_narrative_code": {
        "en": "You asked for code, so I planned it and wrote the program.",
        "ru": "Вы попросили код — я спланировал и написал программу.",
        "hi": "आपने कोड माँगा, तो मैंने उसकी योजना बनाई और कार्यक्रम लिखा।",
        "zh": "你需要代码，所以我做了规划并编写了程序。",
        "es": "Pediste código, así que lo planifiqué y escribí el programa.",
    },
    "thinking_narrative_test_status": {
        "en": "You asked about the tests, so I checked their status and reported it.",
        "ru": "Вы спросили про тесты — я проверил их статус и сообщил.",
        "hi": "आपने परीक्षणों के बारे में पूछा, तो मैंने उनकी स्थिति जाँचकर बताई।",
        "zh": "你询问测试情况，所以我检查了它们的状态并作了汇报。",
        "es": "Preguntaste por las pruebas, así que comprobé su estado y te lo informé.",
    },
    "thinking_narrative_self_healing": {
        "en": "You asked me to fix myself, so I diagnosed the failure and repaired it.",
        "ru": "Вы попросили меня починить себя — я нашёл сбой и исправил его.",
        "hi": "आपने मुझे खुद को ठीक करने को कहा, तो मैंने खराबी पहचानकर उसे ठीक किया।",
        "zh": "你让我修复自己，所以我诊断出故障并进行了修复。",
        "es": "Me pediste que me reparara, así que diagnostiqué el fallo y lo corregí.",
    },
    "thinking_narrative_meta_explanation": {
        "en": "You asked how I work, so I walked through my reasoning.",
        "ru": "Вы спросили, как я работаю, — я разобрал ход своих рассуждений.",
        "hi": "आपने पूछा कि मैं कैसे काम करता हूँ, तो मैंने अपनी तर्क-प्रक्रिया समझाई।",
        "zh": "你问我如何工作，所以我梳理了自己的推理过程。",
        "es": "Preguntaste cómo funciono, así que repasé mi razonamiento.",
    },
    "thinking_narrative_learn_from_source": {
        "en": "You gave me something to learn from, so I read it and updated what I know.",
        "ru": "Вы дали мне материал для изучения — я прочитал его и обновил свои знания.",
        "hi": "आपने मुझे सीखने के लिए कुछ दिया, तो मैंने उसे पढ़कर अपना ज्ञान अपडेट किया।",
        "zh": "你给了我学习材料，所以我读完后更新了自己的知识。",
        "es": "Me diste material para aprender, así que lo leí y actualicé lo que sé.",
    },
    "thinking_narrative_clarification": {
        "en": "The request could mean more than one thing, so I asked you to clarify.",
        "ru": "Запрос можно было понять по-разному — и я попросил уточнить.",
        "hi": "अनुरोध के कई मतलब हो सकते थे, तो मैंने स्पष्ट करने को कहा।",
        "zh": "这个请求可能有多种含义，所以我请你进一步说明。",
        "es": "La solicitud podía significar más de una cosa, así que te pedí una aclaración.",
    },
    "thinking_narrative_unknown": {
        "en": "I wasn't sure how to handle this one yet, so I explained what I can do.",
        "ru": "Я пока не знал, как с этим справиться, — и объяснил, что умею.",
        "hi": "मुझे अभी तक नहीं पता था कि इसे कैसे संभालूँ, तो मैंने बताया कि मैं क्या कर सकता हूँ।",
        "zh": "我暂时还不知道该如何处理，所以我说明了自己能做些什么。",
        "es": "Todavía no sabía cómo tratar esto, así que expliqué qué puedo hacer.",
    },
    "thinking_narrative_generic": {
        "en": "I read this as {article} {task} request, worked out the answer, and replied.",
        "ru": "Я распознал запрос «{task}», нашёл ответ и ответил.",
        "hi": "मैंने इसे “{task}” अनुरोध के रूप में पहचाना, जवाब निकाला और उत्तर दिया।",
        "zh": "我把它识别为“{task}”请求，算出了答案并作了回复。",
        "es": "Interpreté esto como una solicitud de tipo «{task}», resolví la respuesta y contesté.",
    },
}

# The name each registered language is called by, in each answer language.
# Issue #706 keeps the English name in `data/seed/languages.lino`; these records
# add the remaining cells of the same matrix so a Russian trace says «русский»
# rather than "Russian". A newly registered language adds one block here and
# needs no Rust edit — `thinking_language_label` composes the intent from the
# slug and falls back to the ledger's English name.
LANGUAGE_NAMES: dict[str, dict[str, str]] = {
    "thinking_language_name_en": {
        "en": "English",
        "ru": "английский",
        "hi": "अंग्रेज़ी",
        "zh": "英语",
        "es": "inglés",
    },
    "thinking_language_name_ru": {
        "en": "Russian",
        "ru": "русский",
        "hi": "रूसी",
        "zh": "俄语",
        "es": "ruso",
    },
    "thinking_language_name_hi": {
        "en": "Hindi",
        "ru": "хинди",
        "hi": "हिंदी",
        "zh": "印地语",
        "es": "hindi",
    },
    "thinking_language_name_zh": {
        "en": "Chinese",
        "ru": "китайский",
        "hi": "चीनी",
        "zh": "中文",
        "es": "chino",
    },
    "thinking_language_name_es": {
        "en": "Spanish",
        "ru": "испанский",
        "hi": "स्पेनिश",
        "zh": "西班牙语",
        "es": "español",
    },
    "thinking_language_name_unknown": {
        "en": "an unrecognized language",
        "ru": "нераспознанный язык",
        "hi": "एक अपरिचित भाषा",
        "zh": "无法识别的语言",
        "es": "un idioma no reconocido",
    },
}

STEP_HEADER = """\
# Localized thinking-step prose (issue #889, parent #710).
#
# `src/thinking.rs` renders one sentence per structured thinking step for every
# non-UI surface -- the CLI `--thinking` trace, the OpenAI/Anthropic/Responses API
# `reasoning` fields, and the Telegram expandable blockquote. Those sentences
# used to be English literals inside the Rust module, so a Russian, Hindi,
# Chinese or Spanish answer still narrated its reasoning in English.
#
# The sentences now live here, one record per (intent, language), and the module
# renders the record for the answer language. The `{...}` fields are filled from
# the step's own `detail`; the trace keys (`step` and `detail`) stay
# language-neutral, so nothing downstream has to parse prose.
#
# Generated by experiments/issue-889/generate_thinking_seed.py -- edit the
# translation table there and re-run rather than editing this file by hand.
# The narrative headlines and the localized language names live in
# data/seed/multilingual-responses-thinking-narrative.lino.
"""

NARRATIVE_HEADER = """\
# Localized thinking narrative headlines and language names (issue #889).
#
# The headline is the human sentence `thinking_narrative` puts above the
# per-step detail (issue #676, R8); the `thinking_language_name_*` records name
# each registered language in each answer language, so a Russian trace reads
# «русский» rather than "Russian" when it names the language of a request.
# A comment here cannot contain a colon, which canonical Links Notation reads as
# structure even inside a `#` line.
#
# Generated by experiments/issue-889/generate_thinking_seed.py -- edit the
# translation table there and re-run rather than editing this file by hand.
# The per-step sentences live in data/seed/multilingual-responses-thinking.lino.
"""


def render(header: str, tables: list[dict[str, dict[str, str]]]) -> str:
    # Canonical Links Notation reads a blank line as a document break, so the
    # header comment has to sit directly above the root, and no comment may
    # contain a colon.
    lines = [header.rstrip("\n"), "multilingual_responses"]
    for table in tables:
        for intent, translations in table.items():
            for language in LANGUAGES:
                text = translations[language]
                lines.append(f"  response response_{intent}_{language}")
                lines.append(f"    intent {intent}")
                lines.append(f"    language {language}")
                lines.append(f'    text "{text}"')
    return "\n".join(lines) + "\n"


def main() -> None:
    root = pathlib.Path(__file__).resolve().parents[2]
    seed = root / "data" / "seed"
    (seed / "multilingual-responses-thinking.lino").write_text(
        render(STEP_HEADER, [STEPS]), encoding="utf-8"
    )
    (seed / "multilingual-responses-thinking-narrative.lino").write_text(
        render(NARRATIVE_HEADER, [NARRATIVES, LANGUAGE_NAMES]), encoding="utf-8"
    )
    print(
        f"wrote {len(STEPS) * len(LANGUAGES)} step records and "
        f"{(len(NARRATIVES) + len(LANGUAGE_NAMES)) * len(LANGUAGES)} narrative records"
    )


if __name__ == "__main__":
    main()
