//! Issue #425: "Сделай мне пдф файл со списком стран …" is a document-generation
//! task. The solver should recognize the request and return the universal
//! algorithm's formal plan instead of falling through to the unknown response.

use formal_ai::{
    convert_document_format, document_format_capabilities, document_package_is_recognized,
    document_profile_is_recognized, supported_document_formats, FormalAiEngine,
    DOCUMENT_FORMAT_ENGINE,
};

#[test]
fn latest_meta_language_document_formats_are_available() {
    let formats = supported_document_formats();
    for expected in ["txt", "Markdown", "HTML", "PDF", "DOCX"] {
        assert!(
            formats.contains(&expected),
            "meta-language document formats should include {expected}: {formats:?}"
        );
    }

    let html = document_format_capabilities("HTML").expect("HTML profile");
    assert!(html.native_concepts.contains(&"strong".to_owned()));
    assert!(html.native_concepts.contains(&"hyperlink".to_owned()));

    let pdf = document_format_capabilities("PDF").expect("PDF profile");
    assert!(pdf.native_concepts.contains(&"strong".to_owned()));
    assert_eq!(
        pdf.fallbacks
            .iter()
            .find(|(concept, _)| concept == "hyperlink")
            .map(|(_, fallback)| fallback.as_str()),
        Some("rendered as its visible text, unstyled (URL dropped)")
    );

    let docx = document_format_capabilities("DOCX").expect("DOCX profile");
    assert!(docx.native_concepts.contains(&"emphasis".to_owned()));
    assert_eq!(
        docx.fallbacks
            .iter()
            .find(|(concept, _)| concept == "hyperlink")
            .map(|(_, fallback)| fallback.as_str()),
        Some("rendered as its visible text, unstyled (URL dropped)")
    );

    let txt = document_format_capabilities("txt").expect("txt profile");
    assert_eq!(
        txt.fallbacks
            .iter()
            .find(|(concept, _)| concept == "strong")
            .map(|(_, fallback)| fallback.as_str()),
        Some("rendered as unstyled plain text")
    );
}

#[test]
fn meta_language_converts_markdown_to_pdf_and_docx_representations() {
    let source = "# Status\n\nThe **system** is ready.";

    let pdf = convert_document_format("Markdown", "PDF", source).expect("Markdown to PDF");
    assert_eq!(pdf.source_format, "Markdown");
    assert_eq!(pdf.target_format, "PDF");
    assert!(
        pdf.output.starts_with("%PDF-"),
        "PDF conversion should render a PDF document, got: {}",
        pdf.output
    );
    assert!(
        document_profile_is_recognized("PDF", &pdf.output),
        "PDF conversion should be recognized by the meta-language PDF profile"
    );

    let docx = convert_document_format("Markdown", "DOCX", source).expect("Markdown to DOCX");
    assert_eq!(docx.target_format, "DOCX");
    assert!(
        docx.output.contains("<w:document"),
        "DOCX conversion should render WordprocessingML, got: {}",
        docx.output
    );
    assert!(
        docx.output.contains("<w:b/>"),
        "DOCX conversion should preserve strong/bold runs, got: {}",
        docx.output
    );
    assert!(
        document_profile_is_recognized("DOCX", &docx.output),
        "DOCX conversion should be recognized by the meta-language DOCX profile"
    );
    let package_bytes = docx
        .package_bytes
        .as_deref()
        .expect("DOCX conversion should include package bytes");
    assert!(
        package_bytes.starts_with(b"PK"),
        "DOCX package should be an OPC ZIP archive"
    );
    assert!(
        document_package_is_recognized("DOCX", package_bytes),
        "DOCX package should be recognized by the meta-language package profile"
    );
}

#[test]
fn russian_pdf_document_request_returns_plan() {
    let response = FormalAiEngine.answer(
        "Сделай мне пдф файл со списком стран, где есть пособия/скидки на еду для \
         малоимущих, как в виде прямых денежных дотаций, так и в косвенной форме, \
         например, талоны.",
    );

    assert_eq!(response.intent, "document_generation_plan");
    assert!(
        !response.answer.contains("Я не смог определить"),
        "document request should not return the Russian unknown fallback, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("PDF"),
        "plan should name the requested PDF format, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("план") || response.answer.contains("План"),
        "answer should present a formal plan, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Экспортировать"),
        "plan should end with an export step, got: {}",
        response.answer
    );
}

#[test]
fn english_document_request_returns_plan() {
    let response = FormalAiEngine.answer(
        "Make me a PDF document listing countries with food subsidies for low-income people.",
    );

    assert_eq!(response.intent, "document_generation_plan");
    assert!(
        response.answer.contains("document-generation request"),
        "english plan should name the document task, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("PDF"),
        "english plan should name the PDF format, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("link-foundation/meta-language"),
        "english plan should expose the meta-language document workflow, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("txt, Markdown, HTML, PDF, and DOCX"),
        "english plan should name the supported document formats, got: {}",
        response.answer
    );
}

#[test]
fn natural_language_markdown_to_html_conversion_uses_meta_language() {
    let response = FormalAiEngine.answer(
        "Convert this Markdown document to HTML: \"# Status\n\nThe system is **ready** for *launch*.\"",
    );

    assert_eq!(response.intent, "document_format_conversion");
    assert!(
        response.answer.contains(DOCUMENT_FORMAT_ENGINE),
        "conversion answer should name the meta-language engine, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("<h1>Status</h1>"),
        "conversion should preserve the heading as HTML, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("<strong>ready</strong>"),
        "conversion should preserve strong/bold text, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("document_conversion_engine meta_language"),
        "conversion should record the engine in trace links, got: {}",
        response.links_notation
    );
}

#[test]
fn natural_language_to_from_conversion_selects_document_formats() {
    let response = FormalAiEngine
        .answer("Convert to HTML from Markdown: \"# Status\n\nThe system is ready.\"");

    assert_eq!(response.intent, "document_format_conversion");
    assert!(
        response.answer.contains("Source: Markdown; target: HTML."),
        "conversion should read source/target markers, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("<h1>Status</h1>"),
        "conversion should render HTML from the Markdown source, got: {}",
        response.answer
    );
}

#[test]
fn translate_markdown_to_html_routes_to_document_conversion() {
    let response = FormalAiEngine.answer(
        "Translate Markdown to HTML: \"# Status\n\nVisit [the site](https://example.com).\"",
    );

    assert_eq!(response.intent, "document_format_conversion");
    assert!(
        response
            .answer
            .contains("<a href=\"https://example.com\">the site</a>"),
        "document translation should use the meta-language format layer, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("document_conversion_engine meta_language"),
        "conversion should record the meta-language document workflow, got: {}",
        response.links_notation
    );
}

#[test]
fn natural_language_markdown_to_txt_conversion_reports_lossy_fallbacks() {
    let response =
        FormalAiEngine.answer("Convert Markdown to txt: \"# Status\n\nThe **system** is ready.\"");

    assert_eq!(response.intent, "document_format_conversion");
    assert!(
        response.answer.contains("The system is ready."),
        "txt conversion should keep the prose, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("strong -> rendered as unstyled plain text"),
        "txt conversion should report the strong/bold fallback, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("**system**"),
        "txt conversion should drop Markdown styling, got: {}",
        response.answer
    );
}

#[test]
fn natural_language_markdown_to_docx_conversion_reports_package_layer() {
    let response =
        FormalAiEngine.answer("Convert Markdown to DOCX: \"# Status\n\nThe **system** is ready.\"");

    assert_eq!(response.intent, "document_format_conversion");
    assert!(
        response.answer.contains("Package layer:"),
        "DOCX conversion should expose the package layer, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("<w:document"),
        "DOCX conversion should show the WordprocessingML representation, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("document_conversion_package_bytes"),
        "conversion trace should include DOCX package size evidence, got: {}",
        response.links_notation
    );
}

#[test]
fn hindi_pdf_document_request_returns_plan() {
    let response = FormalAiEngine.answer(
        "गरीब लोगों के लिए खाद्य सब्सिडी वाले देशों की सूची के साथ एक PDF \
         दस्तावेज़ बनाओ।",
    );

    assert_eq!(response.intent, "document_generation_plan");
    assert!(
        response.answer.contains("PDF"),
        "hindi plan should name the PDF format, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("दस्तावेज़"),
        "answer should be localized to Hindi, got: {}",
        response.answer
    );
}

#[test]
fn chinese_pdf_document_request_returns_plan() {
    let response =
        FormalAiEngine.answer("给我做一个包含为低收入者提供食品补贴的国家列表的PDF文档。");

    assert_eq!(response.intent, "document_generation_plan");
    assert!(
        response.answer.contains("PDF"),
        "chinese plan should name the PDF format, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("文档"),
        "answer should be localized to Chinese, got: {}",
        response.answer
    );
}

#[test]
fn software_build_request_is_not_a_document_plan() {
    // A build task that merely mentions PDF export must still route to the
    // software-project handler, not the document plan.
    let response = FormalAiEngine.answer("Build me a CLI tool that generates PDF invoices");
    assert_ne!(response.intent, "document_generation_plan");
}

#[test]
fn agent_file_operation_is_not_a_document_plan() {
    // "[agent] … create file report.txt …" is a workspace file op owned by the
    // agent-workspace handler; the document handler must not claim it even
    // though it names "create" and a "report".
    let response = FormalAiEngine
        .answer("[agent] In the isolated workspace, create file report.txt with `alpha`");
    assert_ne!(response.intent, "document_generation_plan");
}

#[test]
fn request_without_document_artifact_is_not_a_document_plan() {
    // "create a list of prime numbers" names no document container, so the
    // handler stays out of the way of the numeric/reasoning handlers.
    let response = FormalAiEngine.answer("Create a list of prime numbers under 100");
    assert_ne!(response.intent, "document_generation_plan");
}
