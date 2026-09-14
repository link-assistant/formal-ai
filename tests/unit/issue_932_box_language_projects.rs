//! Issue #932: a generated language project must build inside the
//! link-foundation/box image that matches its language, driven by the
//! traditional init commands.
//!
//! These regressions cover the two halves that do not need Docker: the contract
//! in `data/meta/box-language-projects.lino` (which the CI matrix and the shell
//! harness both read) and the solver output the corpus is built from — the
//! program itself, asked in English, Russian, Hindi, and Chinese, and the
//! install guide converted into the runnable init script. The container half is
//! `tests/integration/issue_932_box_language_projects.rs`.

use formal_ai::FormalAiEngine;
use formal_ai::box_language_projects::{
    BoxLanguageProject, box_image_survey, box_language_contract,
};

/// The command a developer of each language would actually type to start a
/// project. PR #119 asked for exactly these, not a Formal-AI-specific build.
const TRADITIONAL_INIT_COMMANDS: &[(&str, &str)] = &[
    ("rust", "cargo new"),
    ("python", "python3 -m venv"),
    ("javascript", "npm init"),
    ("typescript", "npm init"),
    ("go", "go mod init"),
    ("java", "javac"),
    ("ruby", "bundle init"),
];

fn expected_program(language: &str) -> &'static str {
    match language {
        "rust" => {
            r#"fn main() {
    println!("Hello, world!");
}"#
        }
        "python" => r#"print("Hello, world!")"#,
        "javascript" | "typescript" => r#"console.log("Hello, world!");"#,
        "go" => {
            r#"package main

import "fmt"

func main() {
    fmt.Println("Hello, world!")
}"#
        }
        "java" => {
            r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, world!");
    }
}"#
        }
        "ruby" => r#"puts "Hello, world!""#,
        other => panic!("no documented program for {other}"),
    }
}

#[test]
fn contract_maps_every_language_to_a_matching_box_image() {
    let contract = box_language_contract();
    assert!(
        contract.projects.len() >= 7,
        "the matrix should cover every language with a dedicated box image"
    );
    assert_eq!(contract.image_tag, "2.4.0", "images must be pinned");

    for project in &contract.projects {
        assert!(
            project.image.starts_with("konard/box"),
            "{} must run in a link-foundation/box image, got {}",
            project.language,
            project.image
        );
        assert_eq!(
            contract.image_reference(project),
            format!("{}:{}", project.image, contract.image_tag)
        );
        for field in [
            &project.display_name,
            &project.code_fence,
            &project.program_file,
            &project.prompt_locale,
        ] {
            assert!(!field.is_empty(), "{} has an empty field", project.language);
        }
        assert!(
            project.init_steps.len() >= 4,
            "{} needs create/enter/build/run steps, got {:?}",
            project.language,
            project.init_steps
        );
        assert!(
            project
                .init_commands()
                .any(|command| command.contains(&contract.project_directory)),
            "{} never creates {}",
            project.language,
            contract.project_directory
        );
    }

    for (language, expected) in TRADITIONAL_INIT_COMMANDS {
        let project = contract
            .project(language)
            .unwrap_or_else(|| panic!("{language} should be in the matrix"));
        assert!(
            project
                .init_commands()
                .any(|command| command.contains(expected)),
            "{language} must use its traditional init command `{expected}`, got {:?}",
            project.init_steps
        );
    }
}

#[test]
fn contract_asks_for_the_program_in_every_supported_locale() {
    let contract = box_language_contract();
    assert_eq!(contract.prompt_locales, ["en", "ru", "hi", "zh"]);

    for locale in &contract.prompt_locales {
        let prompt = contract
            .prompts
            .iter()
            .find(|prompt| &prompt.locale == locale)
            .unwrap_or_else(|| panic!("no prompt template for {locale}"));
        assert!(
            prompt.template.contains("{display_name}"),
            "the {locale} template must be language-independent: {}",
            prompt.template
        );
        assert!(
            contract
                .projects
                .iter()
                .any(|project| &project.prompt_locale == locale),
            "no language keeps the {locale} prompt in the committed corpus"
        );
    }
}

#[test]
fn deferred_languages_name_the_image_they_are_waiting_for() {
    let contract = box_language_contract();
    for language in ["c", "cpp", "csharp"] {
        let deferral = contract
            .deferred
            .iter()
            .find(|deferral| deferral.language == language)
            .unwrap_or_else(|| panic!("{language} should be recorded as deferred"));
        assert_eq!(deferral.image, "konard/box");
        assert!(
            deferral.reason.contains("box image"),
            "{language} must say why it has no dedicated image: {}",
            deferral.reason
        );
        assert!(
            contract.project(language).is_none(),
            "{language} cannot be both deferred and in the matrix"
        );
    }
}

#[test]
fn every_matrix_image_was_actually_found_on_the_registry() {
    let survey = box_image_survey();
    let contract = box_language_contract();
    assert_eq!(
        contract.image_tag, survey.pinned_tag,
        "the matrix must pin the tag the survey found on every variant"
    );
    assert!(
        std::path::Path::new(&survey.evidence).exists(),
        "the survey must cite a committed log, got {}",
        survey.evidence
    );

    for project in &contract.projects {
        assert!(
            survey.publishes(&project.image),
            "{} is pointed at {}, which the survey did not find published",
            project.language,
            project.image
        );
    }
    for deferral in &contract.deferred {
        assert!(
            survey.publishes(&deferral.image),
            "{} falls back to {}, which is not published either",
            deferral.language,
            deferral.image
        );
    }
    for missing in &survey.missing {
        let image = format!("{}/{missing}", survey.namespace);
        assert!(
            !contract
                .projects
                .iter()
                .any(|project| project.image == image),
            "the matrix names {image}, which the survey recorded as missing"
        );
    }
}

#[test]
fn generated_programs_are_identical_across_prompt_locales() {
    let contract = box_language_contract();
    for project in &contract.projects {
        let mut reference: Option<(String, String)> = None;
        for locale in &contract.prompt_locales {
            let prompt = contract
                .prompt_for(locale, project)
                .unwrap_or_else(|| panic!("no {locale} prompt"));
            let response = FormalAiEngine.answer(&prompt);
            let program =
                fenced_block(&response.answer, &project.code_fence).unwrap_or_else(|| {
                    panic!(
                        "{}/{locale} carried no ```{} block: {}",
                        project.language, project.code_fence, response.answer
                    )
                });
            assert_eq!(
                fenced_block(&response.answer, &project.code_fence).as_deref(),
                Some(expected_program(&project.language)),
                "{}/{locale} must show the canonical program exactly",
                project.language
            );
            assert!(
                program.contains(&contract.expected_output),
                "{}/{locale} program never prints {}: {program}",
                project.language,
                contract.expected_output
            );
            match &reference {
                None => reference = Some((locale.clone(), program)),
                Some((first_locale, first_program)) => assert_eq!(
                    first_program, &program,
                    "{} differs between {first_locale} and {locale}",
                    project.language
                ),
            }
        }
    }
}

#[test]
fn install_guides_convert_into_scripts_that_keep_every_traditional_command() {
    let contract = box_language_contract();
    for project in &contract.projects {
        let response = FormalAiEngine.answer(&install_guide_prompt(project));
        assert_eq!(
            response.intent, "installation_conversion",
            "{} guide routed to {}: {}",
            project.language, response.intent, response.answer
        );
        let script = fenced_block(&response.answer, "bash").unwrap_or_else(|| {
            panic!(
                "{} conversion carried no ```bash block: {}",
                project.language, response.answer
            )
        });
        for command in project.init_commands() {
            assert!(
                script.lines().any(|line| line == command),
                "{} init script dropped `{command}`: {script}",
                project.language
            );
        }
    }
}

/// The exact prompt `scripts/generate-box-language-corpus.sh` sends.
fn install_guide_prompt(project: &BoxLanguageProject) -> String {
    format!(
        "Convert this README.md installation guide into a sh script:\n\n```markdown\n{}\n```",
        project.install_guide()
    )
}

/// First fenced block opened with exactly ```` ```<fence> ````, mirroring the
/// extraction the corpus generator performs.
fn fenced_block(answer: &str, fence: &str) -> Option<String> {
    let opening = format!("```{fence}");
    let mut lines = answer.lines().skip_while(|line| line.trim_end() != opening);
    lines.next()?;
    let block: Vec<&str> = lines.take_while(|line| line.trim_end() != "```").collect();
    (!block.is_empty()).then(|| block.join("\n"))
}
