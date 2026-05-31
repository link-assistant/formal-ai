//! Unit tests for the composite-program blueprint synthesizer
//! (`src/coding/blueprint.rs`). Extracted into a sibling file and mounted
//! with `#[path]` so the implementation file stays under the 1000-line
//! Rust file-size limit enforced by `scripts/check-file-size.rs`.

use super::*;

#[test]
fn detects_http_json_statistics_capabilities() {
    let prompt = "write a rust program that makes an http get request to a url, parses the \
                  json response, calculates statistics mean median and outputs the results \
                  with error handling and comments";
    let detected: Vec<&str> = detect_capabilities(prompt)
        .iter()
        .map(|capability| capability.slug)
        .collect();
    for expected in [
        "http_request",
        "json_parse",
        "statistics",
        "output_results",
        "error_handling",
        "comments",
    ] {
        assert!(
            detected.contains(&expected),
            "missing {expected} in {detected:?}"
        );
    }
}

#[test]
fn selects_rust_http_json_stats_blueprint() {
    let prompt = "makes an http get request to a url parses the json response calculates \
                  statistics mean median";
    let blueprint = select_blueprint(prompt, "rust").expect("blueprint resolves for rust");
    assert_eq!(blueprint.recipe.slug, "http_json_stats");
    assert_eq!(blueprint.program.language_slug, "rust");
    assert!(blueprint.program.code.contains("fn main()"));
    assert!(blueprint.program.code.contains("reqwest::blocking::get"));
}

#[test]
fn selects_python_personal_budget_report_blueprint() {
    let prompt = "search average living costs in moscow berlin and new york write a python \
                  program with monthly income 50/30/20 budget rule annual return 10 years \
                  comparison table markdown report with sources";
    let blueprint = select_blueprint(prompt, "python").expect("budget blueprint resolves");
    assert_eq!(blueprint.recipe.slug, "personal_budget_report");
    assert_eq!(blueprint.program.language_slug, "python");
    assert!(blueprint.program.code.contains("budget_50_30_20"));
    assert!(blueprint.program.code.contains("budget_report.md"));
}

#[test]
fn missing_statistics_capability_does_not_match_recipe() {
    // http + json but no statistics -> the recipe's required capabilities are
    // not all present, so no blueprint (honest unsupported fallback kept).
    let prompt = "write a program that makes an http get request and parses the json";
    assert!(select_blueprint(prompt, "rust").is_none());
}

#[test]
fn unsupported_language_for_recipe_yields_none() {
    let prompt = "http get request parse json calculate mean median statistics";
    // The recipe only ships rust/python/javascript programs.
    assert!(select_blueprint(prompt, "go").is_none());
}

#[test]
fn every_recipe_program_language_is_a_catalog_language() {
    for recipe in RECIPES {
        for program in recipe.programs {
            assert!(
                crate::coding::program_language_by_slug(program.language_slug).is_some(),
                "recipe {} references unknown language {}",
                recipe.slug,
                program.language_slug
            );
        }
    }
}

#[test]
fn russian_keywords_detect_capabilities() {
    let prompt = "напиши программу которая делает http запрос разбирает json и считает \
                  среднее и медиану";
    let detected: Vec<&str> = detect_capabilities(prompt)
        .iter()
        .map(|capability| capability.slug)
        .collect();
    assert!(detected.contains(&"http_request"), "{detected:?}");
    assert!(detected.contains(&"json_parse"), "{detected:?}");
    assert!(detected.contains(&"statistics"), "{detected:?}");
}

#[test]
fn render_contains_plan_code_libraries_and_honest_execution() {
    let prompt = "http get request parse json calculate mean median statistics";
    let blueprint = select_blueprint(prompt, "rust").expect("blueprint resolves");
    let rendered = render(
        &blueprint,
        Language::English,
        BlueprintComposition::Composed,
    );
    // Decomposition plan is numbered.
    assert!(rendered.contains("1. Make an HTTP request"), "{rendered}");
    // The real program is embedded in a fenced block.
    assert!(rendered.contains("```rust"), "{rendered}");
    assert!(rendered.contains("reqwest::blocking::get"), "{rendered}");
    // Library prerequisites are listed.
    assert!(rendered.contains("Required libraries:"), "{rendered}");
    assert!(rendered.contains("serde_json"), "{rendered}");
    // The execution report is honest: it never claims the program ran.
    assert!(rendered.contains("not run"), "{rendered}");
    assert!(
        !rendered.to_lowercase().contains("compiled and ran"),
        "{rendered}"
    );
}

#[test]
fn render_localizes_framing_into_russian() {
    let prompt = "http get request parse json calculate mean median statistics";
    let blueprint = select_blueprint(prompt, "python").expect("blueprint resolves");
    let rendered = render(
        &blueprint,
        Language::Russian,
        BlueprintComposition::Composed,
    );
    assert!(rendered.contains("Статус выполнения"), "{rendered}");
    assert!(rendered.contains("```python"), "{rendered}");
}

/// Region directive markers are an internal annotation: they must never reach
/// the user, in either composition strategy.
#[test]
fn region_directives_are_always_stripped_from_output() {
    let prompt = "http get request parse json calculate mean median statistics \
                  with error handling and comments";
    for slug in ["rust", "python", "javascript"] {
        let blueprint = select_blueprint(prompt, slug).expect("blueprint resolves");
        for strategy in [
            BlueprintComposition::Composed,
            BlueprintComposition::Documented,
        ] {
            let program = compose_program(&blueprint, strategy);
            assert!(
                !program.contains("region:"),
                "{slug}/{strategy:?}: leaked region marker: {program}"
            );
            assert!(
                !program.contains("endregion:"),
                "{slug}/{strategy:?}: leaked endregion marker: {program}"
            );
        }
    }
}

/// The composite request that *asks for comments* keeps the documented program
/// — every numbered comment from the curated recipe survives.
#[test]
fn comments_requested_keeps_the_documented_program() {
    let prompt = "http get request parse json calculate mean median statistics \
                  with comments";
    let blueprint = select_blueprint(prompt, "rust").expect("blueprint resolves");
    assert!(wants_comments(&blueprint), "comments capability detected");
    let composed = compose_program(&blueprint, BlueprintComposition::Composed);
    let rendered = render(
        &blueprint,
        Language::English,
        BlueprintComposition::Composed,
    );
    assert!(rendered.contains(&composed), "{rendered}");
    assert!(rendered.contains("// 1. Read the target URL"), "{rendered}");
}

/// A composite request that does *not* ask for comments synthesizes the same
/// logic with the documentation stripped — the program is composed from the
/// decomposition, not served as one frozen blob.
#[test]
fn comments_omitted_strips_documentation_but_keeps_logic() {
    let prompt = "http get request parse json calculate mean median statistics";
    for (slug, fence, logic) in [
        ("rust", "//", "reqwest::blocking::get"),
        ("python", "#", "requests.get"),
        ("javascript", "//", "await fetch(url)"),
    ] {
        let blueprint = select_blueprint(prompt, slug).expect("blueprint resolves");
        assert!(
            !wants_comments(&blueprint),
            "no comments capability for {slug}"
        );
        let stripped = compose_program(&blueprint, BlueprintComposition::Composed);
        let rendered = render(
            &blueprint,
            Language::English,
            BlueprintComposition::Composed,
        );
        assert!(
            rendered.contains(&stripped),
            "{slug}: rendered should embed composed form"
        );
        // Core logic survives the strip.
        assert!(stripped.contains(logic), "{slug}: logic lost: {stripped}");
        assert!(stripped.contains("mean"), "{slug}: stats lost");
        assert!(stripped.contains("median"), "{slug}: stats lost");
        // No whole-line comments remain.
        for line in stripped.lines() {
            assert!(
                !line.trim_start().starts_with(fence),
                "{slug}: residual comment line: {line:?}"
            );
        }
        // No leftover blank-line runs from removed comment blocks.
        assert!(
            !stripped.contains("\n\n\n"),
            "{slug}: blank run left: {stripped:?}"
        );
        // Python docstring is gone.
        if slug == "python" {
            assert!(
                !stripped.contains("\"\"\""),
                "python docstring left: {stripped}"
            );
        }
    }
}

/// The `error_handling` axis is composable independently of comments: when the
/// request omits error handling, the guarded regions disappear from the
/// Composed program but its core logic stays intact, while the Documented
/// strategy keeps the guards regardless.
#[test]
fn error_handling_axis_composes_independently() {
    // Markers of each language's defensive region body.
    let guards = [
        ("rust", "contained no numbers"),
        ("python", "raise_for_status"),
        ("javascript", "response.ok"),
    ];
    for (slug, guard) in guards {
        // Without error handling requested.
        let plain = select_blueprint(
            "http get request parse json calculate mean median statistics",
            slug,
        )
        .expect("blueprint resolves");
        assert!(
            !plain
                .capabilities
                .iter()
                .any(|c| c.slug == "error_handling"),
            "{slug}: error_handling should be absent"
        );
        let composed = compose_program(&plain, BlueprintComposition::Composed);
        assert!(
            !composed.contains(guard),
            "{slug}: composed program should drop guard `{guard}`: {composed}"
        );
        // Core logic is still present after the region drop.
        assert!(
            composed.contains("median"),
            "{slug}: core logic lost: {composed}"
        );

        // Documented strategy keeps the guard even though it was not requested.
        let documented = compose_program(&plain, BlueprintComposition::Documented);
        assert!(
            documented.contains(guard),
            "{slug}: documented program must keep guard `{guard}`: {documented}"
        );

        // With error handling requested, the Composed program keeps the guard.
        let guarded = select_blueprint(
            "http get request parse json calculate mean median statistics \
             with error handling",
            slug,
        )
        .expect("blueprint resolves");
        assert!(
            guarded
                .capabilities
                .iter()
                .any(|c| c.slug == "error_handling"),
            "{slug}: error_handling should be detected"
        );
        let guarded_code = compose_program(&guarded, BlueprintComposition::Composed);
        assert!(
            guarded_code.contains(guard),
            "{slug}: requested guard must survive: {guarded_code}"
        );
    }
}

#[test]
fn stripped_program_is_smaller_than_documented() {
    let prompt = "http get request parse json calculate mean median statistics";
    let blueprint = select_blueprint(prompt, "rust").expect("blueprint resolves");
    let composed = compose_program(&blueprint, BlueprintComposition::Composed);
    let documented = compose_program(&blueprint, BlueprintComposition::Documented);
    assert!(
        composed.lines().count() < documented.lines().count(),
        "composing away comments and unrequested regions should shrink the program"
    );
}

/// Under the `Documented` strategy a *bare* request (no comments, no error
/// handling) still receives the fully annotated program: every optional
/// region body and every whole-line comment is kept — only the region marker
/// lines are stripped.
#[test]
fn documented_strategy_keeps_every_region_and_comment() {
    let prompt = "http get request parse json calculate mean median statistics";
    for (slug, fence, guard) in [
        ("rust", "//", "contained no numbers"),
        ("python", "#", "raise_for_status"),
        ("javascript", "//", "response.ok"),
    ] {
        let blueprint = select_blueprint(prompt, slug).expect("blueprint resolves");
        assert!(!wants_comments(&blueprint), "{slug}: bare request");
        let documented = compose_program(&blueprint, BlueprintComposition::Documented);
        // Optional region bodies survive even though the request omitted them.
        assert!(
            documented.contains(guard),
            "{slug}: documented must keep region body `{guard}`: {documented}"
        );
        // Whole-line comments survive.
        assert!(
            documented
                .lines()
                .any(|line| line.trim_start().starts_with(fence)),
            "{slug}: documented must keep comments: {documented}"
        );
        // Marker lines never reach the user.
        assert!(
            !documented.contains("region:") && !documented.contains("endregion:"),
            "{slug}: marker leaked: {documented}"
        );
    }
}
