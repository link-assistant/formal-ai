//! Issue #425: "Сделай мне пдф файл со списком стран …" is a document-generation
//! task. The solver should recognize the request and return the universal
//! algorithm's formal plan instead of falling through to the unknown response.

use formal_ai::FormalAiEngine;

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
