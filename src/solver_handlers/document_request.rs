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
    if has_any(&["xlsx", "csv", "spreadsheet", "excel", "эксель", "табличк"]) {
        return Some(DocFormat::Spreadsheet);
    }
    if has_any(&["pptx", "powerpoint", "slide deck", "презентаци", "слайд"]) {
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
    ]) {
        return Some(DocFormat::Generic);
    }
    None
}

/// Render the localized document-generation plan. Russian prompts get a Russian
/// plan; every other detected language falls back to English, matching the
/// localization scope of the sibling reasoning handlers.
fn render_document_plan(language: &str, format: DocFormat) -> String {
    match language {
        "ru" => render_plan_ru(format),
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

fn render_plan_en(format: DocFormat) -> String {
    let format_suffix = format
        .label()
        .map_or_else(String::new, |label| format!(" in {label} format"));
    format!(
        "This is a document-generation request{format_suffix}. I am a \
         deterministic symbolic solver: I cannot research arbitrary live data on \
         the web and I do not render binary files directly, so I decompose the \
         task into the formal plan the universal algorithm produces (decompose → \
         tests → drafts → composition):\n\n\
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
