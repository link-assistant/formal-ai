//! Issue #745: intent routing is semantic, object-typed, multilingual, and variation-complete.
//! Registered in the shared unit-test binary so language-coverage CI sees every locale.
//! Coverage matrix: English, Russian, Hindi, and Chinese.
use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::ChatMessage;
use formal_ai::FormalAiEngine;

fn call(prompt: &str) -> (String, serde_json::Value) {
    let tools = [
        "web_fetch",
        "web_search",
        "read_file",
        "write_file",
        "exec_command",
    ];
    call_with_tools(prompt, &tools)
}

fn call_with_tools(prompt: &str, tools: &[&str]) -> (String, serde_json::Value) {
    let messages = vec![ChatMessage::user(prompt)];
    match plan_chat_step(&messages, tools) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1, "expected one call for {prompt:?}");
            let call = &calls[0];
            (
                call.tool.clone(),
                serde_json::from_str(&call.arguments).expect("valid arguments"),
            )
        }
        other => panic!("expected a tool call for {prompt:?}, got {other:?}"),
    }
}

#[test]
fn code_search_prefers_an_advertised_grep_capability_over_shell_lowering() {
    let (tool, arguments) = call_with_tools("search the code for RouteIntent", &["grep_search"]);
    assert_eq!(tool, "grep_search");
    assert_eq!(arguments["pattern"], "RouteIntent");
}

fn assert_routes(prompts: &[&str], expected: &str) {
    assert!(
        prompts.len() >= 15,
        "variation matrix must contain at least 15 rows"
    );
    for prompt in prompts {
        assert_eq!(call(prompt).0, expected, "{prompt}");
    }
}

#[test]
fn url_object_routes_fetch_variations_without_cross_tool_misroutes() {
    for actions in [
        &[
            "fetch",
            "get",
            "download",
            "open",
            "load",
            "retrieve",
            "show me",
            "visit",
            "browse to",
            "read",
            "summarize",
            "check",
            "grab",
            "pull the page",
            "tell me about",
            "what does",
            "what is on",
        ][..],
        &[
            "получи",
            "загрузи",
            "скачай",
            "открой",
            "прочитай",
            "покажи",
            "посети",
            "перейди на",
            "просмотри",
            "проверь",
            "возьми",
            "извлеки",
            "подведи итог",
            "расскажи о",
            "что на",
        ][..],
        &[
            "लाएँ",
            "प्राप्त करें",
            "डाउनलोड करें",
            "खोलें",
            "लोड करें",
            "पढ़ें",
            "दिखाएँ",
            "देखें",
            "जाँचें",
            "सारांश दें",
            "पृष्ठ लें",
            "वेबसाइट पर जाएँ",
            "इसके बारे में बताएँ",
            "क्या लिखा है",
            "क्या है",
        ][..],
        &[
            "获取",
            "下载",
            "打开",
            "加载",
            "读取",
            "显示",
            "访问",
            "前往",
            "查看",
            "检查",
            "抓取",
            "拉取页面",
            "总结",
            "告诉我关于",
            "上面有什么",
        ][..],
    ] {
        let prompts: Vec<String> = actions
            .iter()
            .map(|action| format!("{action} https://example.com"))
            .collect();
        let borrowed: Vec<&str> = prompts.iter().map(String::as_str).collect();
        assert_routes(&borrowed, "web_fetch");
    }
}

#[test]
fn local_path_object_routes_read_variations_without_web_misroutes() {
    for actions in [
        &[
            "read",
            "read the file",
            "show me the contents of",
            "open",
            "print",
            "show",
            "get the contents of",
            "display",
            "view the file",
            "load",
            "what is in",
            "tell me what",
            "cat",
            "inspect",
            "preview",
        ][..],
        &[
            "прочитай",
            "прочитай файл",
            "покажи содержимое",
            "открой",
            "выведи",
            "покажи",
            "получи содержимое",
            "отобрази",
            "просмотри файл",
            "загрузи",
            "что в",
            "расскажи содержимое",
            "посмотри",
            "проверь файл",
            "предпросмотр",
        ][..],
        &[
            "पढ़ें",
            "फ़ाइल पढ़ें",
            "सामग्री दिखाएँ",
            "खोलें",
            "प्रिंट करें",
            "दिखाएँ",
            "सामग्री प्राप्त करें",
            "प्रदर्शित करें",
            "फ़ाइल देखें",
            "लोड करें",
            "में क्या है",
            "सामग्री बताएँ",
            "देखें",
            "फ़ाइल जाँचें",
            "पूर्वावलोकन करें",
        ][..],
        &[
            "读取",
            "读取文件",
            "显示内容",
            "打开",
            "打印",
            "显示",
            "获取内容",
            "展示",
            "查看文件",
            "加载",
            "里面有什么",
            "告诉我内容",
            "查看",
            "检查文件",
            "预览",
        ][..],
    ] {
        let prompts: Vec<String> = actions
            .iter()
            .map(|action| format!("{action} sample.txt"))
            .collect();
        let borrowed: Vec<&str> = prompts.iter().map(String::as_str).collect();
        assert_routes(&borrowed, "read_file");
    }
}

#[test]
fn explicit_content_and_file_object_route_write_variations() {
    const ACTION_SLOT: &str = "{action}";
    let matrices = [
        (
            &[
                "create", "write", "save", "make", "generate", "append", "add", "put", "set",
                "store", "output", "echo", "create a", "new", "produce",
            ][..],
            "{action} file note.txt containing hello",
        ),
        (
            &[
                "создай",
                "напиши",
                "сохрани",
                "сделай",
                "сгенерируй",
                "добавь",
                "помести",
                "установи",
                "запиши",
                "выведи",
                "сохрани в",
                "создать",
                "новый",
                "произведи",
                "сформируй",
            ][..],
            "{action} файл note.txt с текстом hello",
        ),
        (
            &[
                "बनाओ",
                "लिखो",
                "सहेजो",
                "तैयार करो",
                "उत्पन्न करो",
                "जोड़ो",
                "रखो",
                "सेट करो",
                "संग्रहित करो",
                "आउटपुट करो",
                "लिख दें",
                "बनाएँ",
                "नई",
                "उत्पादित करो",
                "दर्ज करो",
            ][..],
            "{action} फ़ाइल note.txt सामग्री के साथ hello",
        ),
        (
            &[
                "创建", "写", "保存", "制作", "生成", "追加", "添加", "放入", "设置", "存储",
                "输出", "回显", "新建", "产生", "记录",
            ][..],
            "{action} 文件 note.txt 内容为 hello",
        ),
    ];
    for (actions, template) in matrices {
        let prompts: Vec<String> = actions
            .iter()
            .map(|action| template.replace(ACTION_SLOT, action))
            .collect();
        let borrowed: Vec<&str> = prompts.iter().map(String::as_str).collect();
        assert_routes(&borrowed, "write_file");
    }
}

#[test]
fn directory_listing_routes_shell_variations_in_every_supported_language() {
    for prompts in [
        &[
            "list files in this folder",
            "show files in this folder",
            "what files are in this folder",
            "display the files here",
            "give me a directory listing",
            "enumerate files in the current directory",
            "show directory contents",
            "which files are here",
            "list this directory",
            "print the file list",
            "inspect this folder",
            "show me local files",
            "what is in the current folder",
            "reveal folder contents",
            "scan the current directory",
        ][..],
        &[
            "покажи файлы в этой папке",
            "список файлов",
            "какие файлы в этой папке",
            "перечисли файлы здесь",
            "покажи содержимое каталога",
            "выведи список файлов",
            "что находится в текущей папке",
            "отобрази файлы каталога",
            "перечисли текущий каталог",
            "покажи локальные файлы",
            "просмотри эту папку",
            "какие файлы здесь",
            "дай список каталога",
            "покажи содержимое текущего каталога",
            "просканируй текущую папку",
        ][..],
        &[
            "इस फ़ोल्डर में फ़ाइलें दिखाएँ",
            "फ़ाइलों की सूची",
            "इस फ़ोल्डर में कौन सी फ़ाइलें हैं",
            "यहाँ फ़ाइलें सूचीबद्ध करें",
            "निर्देशिका की सामग्री दिखाएँ",
            "फ़ाइल सूची प्रिंट करें",
            "वर्तमान फ़ोल्डर में क्या है",
            "निर्देशिका फ़ाइलें प्रदर्शित करें",
            "वर्तमान निर्देशिका सूचीबद्ध करें",
            "स्थानीय फ़ाइलें दिखाएँ",
            "इस फ़ोल्डर को देखें",
            "यहाँ कौन सी फ़ाइलें हैं",
            "निर्देशिका सूची दें",
            "वर्तमान निर्देशिका की सामग्री दिखाएँ",
            "वर्तमान फ़ोल्डर स्कैन करें",
        ][..],
        &[
            "显示这个文件夹里的文件",
            "文件列表",
            "这个文件夹里有哪些文件",
            "列出这里的文件",
            "显示目录内容",
            "打印文件列表",
            "当前文件夹里有什么",
            "展示目录文件",
            "列出当前目录",
            "显示本地文件",
            "查看这个文件夹",
            "这里有哪些文件",
            "给出目录列表",
            "显示当前目录的内容",
            "扫描当前文件夹",
        ][..],
    ] {
        assert_routes(prompts, "exec_command");
        for prompt in prompts {
            assert_eq!(call(prompt).1["command"], "ls", "{prompt}");
        }
    }
}

#[test]
fn web_search_routes_action_variations_in_every_supported_language() {
    for actions in [
        &[
            "search the web for",
            "search the internet for",
            "search online for",
            "web search for",
            "find online",
            "look up online",
            "google search for",
            "research online",
            "investigate on the web",
            "discover online",
            "query the web for",
            "browse the web for",
            "seek online",
            "find on the internet",
            "check the web for",
        ][..],
        &[
            "найди в интернете",
            "поищи в интернете",
            "поиск в сети",
            "найди онлайн",
            "посмотри в интернете",
            "загугли",
            "исследуй онлайн",
            "проверь в сети",
            "отыщи в интернете",
            "выполни веб поиск",
            "запроси сеть о",
            "поищи онлайн",
            "найди в сети",
            "изучи в интернете",
            "разыщи онлайн",
        ][..],
        &[
            "वेब पर खोजें",
            "इंटरनेट पर खोजें",
            "ऑनलाइन खोजें",
            "वेब खोज",
            "ऑनलाइन ढूँढें",
            "इंटरनेट पर देखें",
            "गूगल करें",
            "ऑनलाइन शोध करें",
            "वेब पर जाँचें",
            "ऑनलाइन पता लगाएँ",
            "वेब से पूछें",
            "इंटरनेट खंगालें",
            "ऑनलाइन तलाशें",
            "वेब में खोजें",
            "नेट पर खोजें",
        ][..],
        &[
            "搜索网络",
            "在互联网上搜索",
            "在线搜索",
            "网页搜索",
            "在线查找",
            "上网查找",
            "谷歌搜索",
            "在线研究",
            "在网络上调查",
            "在线发现",
            "查询网络",
            "浏览网络查找",
            "在线寻找",
            "在互联网上查找",
            "检查网络上的",
        ][..],
    ] {
        let prompts: Vec<String> = actions
            .iter()
            .map(|action| format!("{action} rust ownership"))
            .collect();
        let borrowed: Vec<&str> = prompts.iter().map(String::as_str).collect();
        assert_routes(&borrowed, "web_search");
    }
}

#[test]
fn reported_object_type_collisions_choose_the_right_capability() {
    for (prompt, tool) in [
        ("display sample.txt", "read_file"),
        ("cat sample.txt", "read_file"),
        ("load sample.txt", "read_file"),
        ("read https://example.com", "web_fetch"),
        ("summarize https://example.com", "web_fetch"),
        ("set the contents of note.txt to hello", "write_file"),
        ("search the code for RouteIntent", "exec_command"),
    ] {
        assert_eq!(call(prompt).0, tool, "{prompt}");
    }
}

#[test]
fn attachment_filenames_are_not_reinterpreted_as_bare_web_hosts() {
    for prompt in [
        "Check this attached text for uniqueness and plagiarism\n\nAttached files:\n1. article.txt (text/plain, 12.0 KB)",
        "Проверь приложенный текст на достоверность\n\nAttached files:\n1. novost.txt (text/plain, 4.0 KB)",
    ] {
        assert_eq!(
            FormalAiEngine.answer(prompt).intent,
            "document_originality_check",
            "{prompt}"
        );
    }
}
