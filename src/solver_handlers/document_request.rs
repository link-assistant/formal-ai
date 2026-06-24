//! Document-generation request handler (issue #425).
//!
//! Open-ended "make me a PDF / document / report with <subject>" prompts ask the
//! deterministic solver to research arbitrary data and render a binary file —
//! two capabilities a symbolic engine does not have. Rather than fall through to
//! the unknown opener, this handler recognizes the request as a *document task*
//! and returns the formal decomposition the universal algorithm produces: scope
//! the deliverable, enumerate the items, classify them by the stated criteria,
//! assemble the structure, then export it to the requested format. The plan is
//! localized to the prompt language so a Russian PDF request gets a Russian plan.

use std::fmt::Write as _;

use crate::document_formats::{
    convert_document_format, cross_format_document_concepts, supported_document_formats,
    DocumentConversion, DOCUMENT_FORMAT_ENGINE,
};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::is_agent_request;

/// The document container a prompt asked for. The variants double as the
/// language-neutral format label shown in the plan; [`DocFormat::Generic`] is the
/// fallback for a bare "document"/"report" with no explicit container format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocFormat {
    Pdf,
    Word,
    Spreadsheet,
    Presentation,
    Ebook,
    Generic,
}

impl DocFormat {
    /// The language-neutral format token appended to the plan's opening line, or
    /// `None` for [`DocFormat::Generic`] (which reads as a bare "document").
    const fn label(self) -> Option<&'static str> {
        match self {
            Self::Pdf => Some("PDF"),
            Self::Word => Some("DOCX"),
            Self::Spreadsheet => Some("CSV/XLSX"),
            Self::Presentation => Some("PPTX"),
            Self::Ebook => Some("EPUB"),
            Self::Generic => None,
        }
    }
}

/// Recognize a document-generation request and answer with a formal plan.
///
/// The gate is the conjunction of an authoring action ("make"/"сделай"/…) and a
/// document artifact (an explicit format such as "PDF" or a generic document
/// noun). Prompts that also name a software artifact ("app", "bot", "script", …)
/// defer to the software-project handler, which runs later in the dispatch table.
pub fn try_document_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let lowercased = normalized.to_lowercase();
    // Agent-mode prompts ("[agent] … create file report.txt …") are file
    // operations owned by the agent-workspace handler, not document tasks.
    if is_agent_request(&lowercased) {
        return None;
    }
    if mentions_software_artifact(&lowercased) {
        return None;
    }
    if let Some(answer) = try_document_conversion_request(prompt, &lowercased, log) {
        return Some(answer);
    }
    if !mentions_authoring_action(&lowercased) {
        return None;
    }
    let format = detect_document_format(&lowercased)?;

    if let Some(label) = format.label() {
        log.append("document_request:format", label.to_owned());
    } else {
        log.append("document_request:format", "document".to_owned());
    }

    let language = detect_language(prompt).slug();
    let body = render_document_plan(language, format);
    Some(finalize_simple(
        prompt,
        log,
        "document_generation_plan",
        "response:document_generation_plan",
        &body,
        0.6,
    ))
}

fn try_document_conversion_request(
    prompt: &str,
    lowercased: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !mentions_conversion_action(lowercased) {
        return None;
    }

    let mentions = format_mentions(lowercased);
    let (source_format, target_format) = conversion_formats(lowercased, &mentions)?;
    let source_text = extract_document_source_text(prompt)?;
    let conversion = convert_document_format(source_format, target_format, &source_text)?;

    log.append(
        "document_conversion_engine",
        DOCUMENT_FORMAT_ENGINE.to_owned(),
    );
    log.append(
        "document_conversion_source_format",
        conversion.source_format.clone(),
    );
    log.append(
        "document_conversion_target_format",
        conversion.target_format.clone(),
    );
    log.append(
        "document_conversion_concepts",
        cross_format_document_concepts().join(","),
    );
    log.append(
        "document_conversion_output_bytes",
        conversion.output.len().to_string(),
    );
    if let Some(package_bytes) = conversion.package_bytes.as_ref() {
        log.append(
            "document_conversion_package_bytes",
            package_bytes.len().to_string(),
        );
    }

    let body = render_conversion_answer(&conversion);
    Some(finalize_simple(
        prompt,
        log,
        "document_format_conversion",
        "response:document_format_conversion",
        &body,
        0.85,
    ))
}

pub(super) fn looks_like_document_conversion_request(prompt: &str, lowercased: &str) -> bool {
    mentions_conversion_action(lowercased)
        && conversion_formats(lowercased, &format_mentions(lowercased)).is_some()
        && extract_document_source_text(prompt).is_some()
}

fn mentions_conversion_action(lowercased: &str) -> bool {
    const ACTIONS: &[&str] = &[
        "convert",
        "translate",
        "transform",
        "render",
        "export",
        "turn into",
        "turn this into",
        "change into",
        "конверт",
        "преобраз",
        "переведи",
        "перевести",
        "экспорт",
        "बदल",
        "रूपांतर",
        "转换",
        "转成",
        "变成",
        "导出",
    ];
    ACTIONS.iter().any(|needle| lowercased.contains(needle))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FormatMention {
    index: usize,
    format: &'static str,
}

fn format_mentions(lowercased: &str) -> Vec<FormatMention> {
    const ALIASES: &[(&str, &str)] = &[
        ("plain text", "txt"),
        ("plaintext", "txt"),
        ("txt", "txt"),
        ("markdown", "Markdown"),
        ("md", "Markdown"),
        ("html", "HTML"),
        ("htm", "HTML"),
        ("pdf", "PDF"),
        ("docx", "DOCX"),
        ("word document", "DOCX"),
        ("wordprocessingml", "DOCX"),
    ];

    let mut mentions = Vec::new();
    for (alias, format) in ALIASES {
        mentions.extend(
            find_alias_positions(lowercased, alias)
                .into_iter()
                .map(|index| FormatMention { index, format }),
        );
    }
    mentions.sort_by_key(|mention| mention.index);
    mentions.dedup_by(|left, right| left.index == right.index && left.format == right.format);
    mentions
}

fn conversion_formats<'a>(
    lowercased: &str,
    mentions: &'a [FormatMention],
) -> Option<(&'a str, &'a str)> {
    let explicit_source = mentions
        .iter()
        .find(|mention| has_direction_marker_before(lowercased, mention.index, SOURCE_MARKERS))
        .map(|mention| mention.format);
    let explicit_target = mentions
        .iter()
        .find(|mention| has_direction_marker_before(lowercased, mention.index, TARGET_MARKERS))
        .map(|mention| mention.format);

    match (explicit_source, explicit_target) {
        (Some(source), Some(target)) if source != target => Some((source, target)),
        (Some(source), _) => mentions
            .iter()
            .rev()
            .find(|mention| mention.format != source)
            .map(|mention| (source, mention.format)),
        (_, Some(target)) => mentions
            .iter()
            .find(|mention| mention.format != target)
            .map(|mention| (mention.format, target)),
        _ => {
            let source = mentions.first()?.format;
            mentions
                .iter()
                .rev()
                .find(|mention| mention.format != source)
                .map(|mention| (source, mention.format))
        }
    }
}

const SOURCE_MARKERS: &[&str] = &["from ", "source ", "из ", "с ", "от ", "से ", "从"];

const TARGET_MARKERS: &[&str] = &[
    "to ",
    "into ",
    "as ",
    "target ",
    "в ",
    "на ",
    "में ",
    "को ",
    "为",
    "成",
    "到",
];

fn has_direction_marker_before(haystack: &str, index: usize, markers: &[&str]) -> bool {
    let before = haystack[..index].trim_end();
    markers
        .iter()
        .any(|marker| before.ends_with(marker.trim_end()))
}

fn find_alias_positions(haystack: &str, alias: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut offset = 0usize;
    while let Some(relative) = haystack[offset..].find(alias) {
        let index = offset + relative;
        let end = index + alias.len();
        if is_token_boundary(haystack, index, end) {
            positions.push(index);
        }
        offset = end;
    }
    positions
}

fn is_token_boundary(haystack: &str, start: usize, end: usize) -> bool {
    let before = haystack[..start].chars().next_back();
    let after = haystack[end..].chars().next();
    !before.is_some_and(is_ascii_word_char) && !after.is_some_and(is_ascii_word_char)
}

const fn is_ascii_word_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}

fn extract_document_source_text(prompt: &str) -> Option<String> {
    first_fenced_code_block(prompt)
        .or_else(|| quoted_segments(prompt).last().cloned())
        .or_else(|| text_after_colon(prompt))
        .and_then(non_empty)
}

fn first_fenced_code_block(text: &str) -> Option<String> {
    let fence_start = text.find("```")?;
    let after_fence = &text[fence_start + 3..];
    let content_start = after_fence
        .find('\n')
        .map_or(fence_start + 3, |newline| fence_start + 3 + newline + 1);
    let content = &text[content_start..];
    let fence_end = content.find("```")?;
    Some(content[..fence_end].trim_matches('\n').to_owned())
}

fn text_after_colon(text: &str) -> Option<String> {
    text.find(':')
        .or_else(|| text.find('：'))
        .map(|index| text[index + 1..].trim().to_owned())
}

fn quoted_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        let Some((relative_start, open, close)) =
            text[cursor..]
                .char_indices()
                .find_map(|(index, character)| {
                    quote_close_for(character).map(|close| (index, character, close))
                })
        else {
            break;
        };
        let content_start = cursor + relative_start + open.len_utf8();
        let Some(relative_end) = text[content_start..].find(close) else {
            break;
        };
        let content_end = content_start + relative_end;
        segments.push(text[content_start..content_end].to_owned());
        cursor = content_end + close.len_utf8();
    }
    segments
}

const fn quote_close_for(open: char) -> Option<char> {
    match open {
        '\'' => Some('\''),
        '"' => Some('"'),
        '`' => Some('`'),
        '«' => Some('»'),
        '“' => Some('”'),
        '‘' => Some('’'),
        '「' => Some('」'),
        '『' => Some('』'),
        _ => None,
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn render_conversion_answer(conversion: &DocumentConversion) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Document format conversion via link-foundation/meta-language ({DOCUMENT_FORMAT_ENGINE})."
    );
    let _ = writeln!(
        body,
        "Source: {}; target: {}.",
        conversion.source_format, conversion.target_format
    );
    let _ = writeln!(
        body,
        "Supported document formats: {}.",
        supported_document_formats().join(", ")
    );
    if !conversion.target_capabilities.native_concepts.is_empty() {
        let _ = writeln!(
            body,
            "Native target concepts: {}.",
            conversion.target_capabilities.native_concepts.join(", ")
        );
    }
    if !conversion.target_capabilities.fallbacks.is_empty() {
        let _ = writeln!(body, "Documented target fallbacks:");
        for (concept, fallback) in &conversion.target_capabilities.fallbacks {
            let _ = writeln!(body, "- {concept} -> {fallback}");
        }
    }
    if let Some(package_bytes) = conversion.package_bytes.as_ref() {
        let _ = writeln!(
            body,
            "Package layer: {} bytes in the DOCX OPC stored-entry profile.",
            package_bytes.len()
        );
    }

    let fence = output_fence(&conversion.target_format);
    let _ = write!(body, "\n```{fence}\n{}\n```", conversion.output.trim_end());
    body
}

fn output_fence(format: &str) -> &'static str {
    match format {
        "HTML" => "html",
        "Markdown" => "markdown",
        "PDF" => "pdf",
        "DOCX" => "xml",
        "txt" => "text",
        _ => "",
    }
}

/// True when the prompt names a software artifact, in which case the request is a
/// build task for the software-project handler rather than a document task.
fn mentions_software_artifact(lowercased: &str) -> bool {
    const SOFTWARE: &[&str] = &[
        // English
        "app",
        "application",
        "extension",
        "plugin",
        "addon",
        "add-on",
        "bot",
        "program",
        "script",
        "website",
        "web site",
        "web app",
        "webapp",
        "game",
        "api",
        "library",
        "framework",
        "tool",
        "cli",
        "daemon",
        "server",
        // Russian (stems tolerate inflection)
        "приложени",
        "программ",
        "скрипт",
        "расширени",
        "плагин",
        "бот",
        "сайт",
        "веб-сайт",
        "игр",
        "утилит",
        "сервер",
        "библиотек",
    ];
    SOFTWARE.iter().any(|needle| lowercased.contains(needle))
}

/// True when the prompt carries an explicit authoring verb. Russian entries are
/// stems so they survive inflection (`сделай`, `сделать`, `сделаешь`, …).
fn mentions_authoring_action(lowercased: &str) -> bool {
    const ACTIONS: &[&str] = &[
        // English
        "make me",
        "make a",
        "make an",
        "create",
        "generate",
        "produce",
        "compile",
        "prepare",
        "assemble",
        "draft",
        "build me",
        "export",
        "give me",
        "i need",
        "i want",
        "put together",
        // Russian (verb stems)
        "сдела",
        "созда",
        "сгенерир",
        "подготов",
        "сформир",
        "состав",
        "собери",
        "собрать",
        "оформ",
        "выгрузи",
        "экспортир",
        "сверста",
        // Hindi (verb stems; `बना` covers बनाओ/बनाएं/बनाना)
        "बना",
        "तैयार",
        "उत्पन्न",
        "चाहिए",
        // Chinese (authoring verbs)
        "做",
        "制作",
        "生成",
        "创建",
        "编写",
        "整理",
        "给我",
        "帮我",
    ];
    ACTIONS.iter().any(|needle| lowercased.contains(needle))
}

/// Classify the requested document container, preferring an explicit format token
/// over a generic document noun. Returns `None` when no document artifact is
/// named (so a plain "create X" prompt is left for other handlers).
fn detect_document_format(lowercased: &str) -> Option<DocFormat> {
    let has_any = |needles: &[&str]| needles.iter().any(|needle| lowercased.contains(needle));

    if has_any(&["pdf", "пдф"]) {
        return Some(DocFormat::Pdf);
    }
    if has_any(&["docx", ".doc", "word document", "ворд", "документ word"]) {
        return Some(DocFormat::Word);
    }
    if has_any(&[
        "xlsx",
        "csv",
        "spreadsheet",
        "excel",
        "эксель",
        "табличк",
        "电子表格",
        "表格",
        "स्प्रेडशीट",
    ]) {
        return Some(DocFormat::Spreadsheet);
    }
    if has_any(&[
        "pptx",
        "powerpoint",
        "slide deck",
        "презентаци",
        "слайд",
        "演示",
        "幻灯片",
        "प्रस्तुति",
    ]) {
        return Some(DocFormat::Presentation);
    }
    if has_any(&["epub", "e-book", "ebook", "электронн"]) && has_any(&["book", "книг"])
    {
        return Some(DocFormat::Ebook);
    }
    // Generic document noun: distinctive stems unlikely to fire spuriously. A
    // bare "file"/"файл" is intentionally excluded — it is too weak on its own.
    if has_any(&[
        "document",
        "документ",
        "report",
        "отчет",
        "отчёт",
        "dossier",
        "досье",
        "brochure",
        "брошюр",
        "booklet",
        "буклет",
        "memo",
        "записку",
        "записка",
        // Hindi
        "दस्तावेज़",
        "दस्तावेज",
        "रिपोर्ट",
        // Chinese
        "文档",
        "文件",
        "报告",
    ]) {
        return Some(DocFormat::Generic);
    }
    None
}

/// Render the localized document-generation plan. The plan is translated for
/// every supported language (Russian, Hindi, Chinese); any other detected
/// language falls back to English, matching the localization scope of the
/// sibling reasoning handlers.
fn render_document_plan(language: &str, format: DocFormat) -> String {
    match language {
        "ru" => render_plan_ru(format),
        "hi" => render_plan_hi(format),
        "zh" => render_plan_zh(format),
        _ => render_plan_en(format),
    }
}

fn render_plan_ru(format: DocFormat) -> String {
    let format_suffix = format
        .label()
        .map_or_else(String::new, |label| format!(" в формате {label}"));
    format!(
        "Это запрос на создание документа{format_suffix}. Я детерминированный \
         символьный решатель: у меня нет доступа к произвольным актуальным данным \
         в вебе и я не рендерю бинарные файлы напрямую, поэтому я раскладываю \
         задачу на формальный план по универсальному алгоритму (декомпозиция → \
         проверки → черновики → композиция):\n\n\
         1. Уточнить объём и критерии документа: какие элементы включать и по \
         каким признакам их различать.\n\
         2. Собрать список элементов из проверяемых источников и зафиксировать \
         ссылки на эти источники.\n\
         3. Классифицировать каждый элемент по заявленным критериям.\n\
         4. Собрать структуру документа: заголовок, разделы и таблицу или список.\n\
         5. Экспортировать готовую структуру в запрошенный формат.\n\n\
         Подтвердите план или уточните критерии и источники — и я продолжу с \
         конкретными шагами. Если нужны фактические данные, которых нет в \
         локальной памяти Links Notation, укажите источник, и я добавлю его как \
         правило связей."
    )
}

fn render_plan_hi(format: DocFormat) -> String {
    let format_suffix = format
        .label()
        .map_or_else(String::new, |label| format!(" ({label} प्रारूप में)"));
    format!(
        "यह एक दस्तावेज़ बनाने का अनुरोध है{format_suffix}. मैं एक नियतात्मक \
         प्रतीकात्मक हल करने वाला हूँ: मैं वेब पर मनमाना सजीव डेटा नहीं खोज सकता और \
         बाइनरी फ़ाइलें सीधे नहीं बनाता, इसलिए मैं इस कार्य को सार्वभौमिक एल्गोरिदम \
         की औपचारिक योजना में विभाजित करता हूँ (विभाजन → जाँच → मसौदे → रचना):\n\n\
         1. दस्तावेज़ और उसके मानदंड का दायरा तय करें: कौन-सी वस्तुएँ शामिल करनी हैं \
         और कौन-से गुण उन्हें अलग करते हैं।\n\
         2. सत्यापन योग्य स्रोतों से वस्तुओं की सूची एकत्र करें और उन स्रोतों के लिंक \
         दर्ज करें।\n\
         3. प्रत्येक वस्तु को बताए गए मानदंड के अनुसार वर्गीकृत करें।\n\
         4. दस्तावेज़ की संरचना बनाएँ: शीर्षक, अनुभाग और एक तालिका या सूची।\n\
         5. तैयार संरचना को अनुरोधित प्रारूप में निर्यात करें।\n\n\
         योजना की पुष्टि करें या मानदंड और स्रोत स्पष्ट करें, और मैं ठोस चरणों के \
         साथ आगे बढ़ूँगा।"
    )
}

fn render_plan_zh(format: DocFormat) -> String {
    let format_suffix = format
        .label()
        .map_or_else(String::new, |label| format!("（{label} 格式）"));
    format!(
        "这是一个生成文档的请求{format_suffix}。我是一个确定性的符号求解器：我无法在\
         网络上检索任意实时数据，也不会直接渲染二进制文件，因此我把任务分解为通用\
         算法生成的形式化计划（分解 → 校验 → 草稿 → 组合）：\n\n\
         1. 界定文档及其标准：包含哪些条目以及用哪些属性区分它们。\n\
         2. 从可验证的来源收集条目清单，并记录这些来源的链接。\n\
         3. 根据所述标准对每个条目进行分类。\n\
         4. 组装文档结构：标题、章节以及表格或列表。\n\
         5. 将完成的结构导出为所请求的格式。\n\n\
         请确认计划或细化标准与来源，我将继续给出具体步骤。"
    )
}

fn render_plan_en(format: DocFormat) -> String {
    let format_suffix = format
        .label()
        .map_or_else(String::new, |label| format!(" in {label} format"));
    format!(
        "This is a document-generation request{format_suffix}. I am a \
         deterministic symbolic solver: I cannot research arbitrary live data on \
         the web and I do not render binary files directly, so I decompose the \
         task into the formal plan the universal algorithm produces (decompose → \
         tests → drafts → composition). The document workflow uses \
         link-foundation/meta-language for txt, Markdown, HTML, PDF, and DOCX \
         representation/conversion, with concept profiles for headings, \
         paragraphs, lists, strong/bold text, emphasis, and hyperlinks:\n\n\
         1. Scope the document and its criteria: which items to include and which \
         attributes distinguish them.\n\
         2. Collect the list of items from verifiable sources and record links to \
         those sources.\n\
         3. Classify each item against the stated criteria.\n\
         4. Assemble the document structure: title, sections, and a table or list.\n\
         5. Export the finished structure to the requested format.\n\n\
         Confirm the plan or refine the criteria and sources and I will continue \
         with concrete steps. If you need facts that are not in the local Links \
         Notation memory, name a source and I will add it as a links rule."
    )
}
