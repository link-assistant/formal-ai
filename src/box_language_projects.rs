//! Issue #932: the language → link-foundation/box image contract as link data.
//!
//! PR [#119](https://github.com/link-assistant/formal-ai/pull/119) asked that a
//! generated project for a specific language be exercised *inside the box image
//! that matches that language*, using the traditional init commands a human
//! would type (`cargo new`, `npm init`, `go mod init`, …) rather than a
//! Formal-AI-specific build path.
//!
//! `data/meta/box-language-projects.lino` is the single reviewable source for
//! that mapping: which image serves a language, which fenced block the generated
//! program comes out of, which prompt locale the corpus asks in, and the ordered
//! README steps whose commands the container has to run. Three consumers read
//! it — this module (tests and CI invariants), the corpus generator, and the
//! host-side verifier — so the workflow matrix, the shell harness, and the test
//! expectations can never drift apart.

use crate::seed::parser::{parse_lino, LinoNode};

/// The embedded contract, so a released binary carries the same mapping the
/// repository scripts read from disk.
const BOX_LANGUAGE_PROJECTS_LINO: &str = include_str!("../data/meta/box-language-projects.lino");

/// The embedded image survey the contract is checked against.
const BOX_IMAGE_SURVEY_LINO: &str = include_str!("../data/meta/box-image-survey.lino");

/// Record type of a language that has a dedicated box image.
const RECORD_PROJECT: &str = "box_language_project";
/// Record type of a language whose toolchain ships only in the full box image.
const RECORD_DEFERRED: &str = "box_language_project_deferred";
/// Record type of one localized "write a hello world" prompt template.
const RECORD_PROMPT: &str = "box_language_prompt";

/// One README step and the command it runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxInitStep {
    /// Human-readable step description, as it appears in the generated guide.
    pub description: String,
    /// The traditional command the container executes for this step.
    pub command: String,
}

/// A language exercised inside its matching box image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxLanguageProject {
    /// Coding-catalog language slug (`rust`, `python`, …).
    pub language: String,
    /// Display name substituted into the localized prompt templates.
    pub display_name: String,
    /// Image repository without the tag, for example `konard/box-rust`.
    pub image: String,
    /// Fence tag the generated program is extracted from.
    pub code_fence: String,
    /// File name the generated program is saved as.
    pub program_file: String,
    /// Locale whose prompt the committed corpus keeps for this language.
    pub prompt_locale: String,
    /// Shell line sourced before the init script (nvm, rvm, …), when needed.
    pub shell_prelude: Option<String>,
    /// Whether the init commands need network access (package downloads).
    pub network_required: bool,
    /// Ordered README steps, each with its traditional command.
    pub init_steps: Vec<BoxInitStep>,
}

impl BoxLanguageProject {
    /// Every init command in order.
    pub fn init_commands(&self) -> impl Iterator<Item = &str> {
        self.init_steps.iter().map(|step| step.command.as_str())
    }

    /// The README installation guide for this language, exactly as the corpus
    /// generator builds it before asking the solver to convert it.
    #[must_use]
    pub fn install_guide(&self) -> String {
        use std::fmt::Write as _;

        let mut guide = String::from("## Installation");
        for (index, step) in self.init_steps.iter().enumerate() {
            let _ = write!(
                guide,
                "\n{}. {}\n   `{}`",
                index + 1,
                step.description,
                step.command
            );
        }
        guide
    }
}

/// A language whose toolchain has no dedicated box image yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxLanguageDeferral {
    /// Coding-catalog language slug.
    pub language: String,
    /// Image that would have to be used instead.
    pub image: String,
    /// Why the language is not in the matrix yet.
    pub reason: String,
}

/// One localized prompt template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxLanguagePrompt {
    /// Locale slug (`en`, `ru`, `hi`, `zh`).
    pub locale: String,
    /// Template carrying the `{display_name}` placeholder.
    pub template: String,
}

/// The whole contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxLanguageContract {
    /// Tag every image in the matrix is pinned to.
    pub image_tag: String,
    /// Directory the traditional init commands create.
    pub project_directory: String,
    /// Line every generated program must print.
    pub expected_output: String,
    /// Locales the corpus asks each program in.
    pub prompt_locales: Vec<String>,
    /// Script that generates the corpus from solver answers.
    pub corpus_script: String,
    /// Host-side verifier that runs each project in its image.
    pub verify_script: String,
    /// In-container runner copied into the corpus directory.
    pub container_script: String,
    /// Release-workflow job that runs the matrix.
    pub workflow_job: String,
    /// Languages with a dedicated image.
    pub projects: Vec<BoxLanguageProject>,
    /// Languages waiting for an image.
    pub deferred: Vec<BoxLanguageDeferral>,
    /// Localized prompt templates.
    pub prompts: Vec<BoxLanguagePrompt>,
}

impl BoxLanguageContract {
    /// Parse the embedded contract.
    #[must_use]
    pub fn load() -> Self {
        Self::parse(BOX_LANGUAGE_PROJECTS_LINO)
    }

    /// Parse an explicit contract document; production uses [`Self::load`].
    #[must_use]
    pub fn parse(document: &str) -> Self {
        let root = parse_lino(document);
        let header = root
            .children
            .iter()
            .find(|node| node.find_child_value("record_type") == "box_language_project_contract")
            .expect("the contract must declare a box_language_project_contract header");
        let mut contract = Self {
            image_tag: header.find_child_value("image_tag").to_owned(),
            project_directory: header.find_child_value("project_directory").to_owned(),
            expected_output: header.find_child_value("expected_output").to_owned(),
            prompt_locales: split_tuple(header.find_child_value("prompt_locales")),
            corpus_script: header.find_child_value("corpus_script").to_owned(),
            verify_script: header.find_child_value("verify_script").to_owned(),
            container_script: header.find_child_value("container_script").to_owned(),
            workflow_job: header.find_child_value("workflow_job").to_owned(),
            projects: Vec::new(),
            deferred: Vec::new(),
            prompts: Vec::new(),
        };
        for node in &root.children {
            match node.find_child_value("record_type") {
                RECORD_PROJECT => contract.projects.push(parse_project(node)),
                RECORD_DEFERRED => contract.deferred.push(BoxLanguageDeferral {
                    language: node.find_child_value("language").to_owned(),
                    image: node.find_child_value("image").to_owned(),
                    reason: node.find_child_value("reason").to_owned(),
                }),
                RECORD_PROMPT => contract.prompts.push(BoxLanguagePrompt {
                    locale: node.find_child_value("locale").to_owned(),
                    template: node.find_child_value("template").to_owned(),
                }),
                _ => {}
            }
        }
        contract
    }

    /// Look up one language.
    #[must_use]
    pub fn project(&self, language: &str) -> Option<&BoxLanguageProject> {
        self.projects
            .iter()
            .find(|project| project.language == language)
    }

    /// The pinned `repository:tag` reference for a language's image.
    #[must_use]
    pub fn image_reference(&self, project: &BoxLanguageProject) -> String {
        format!("{}:{}", project.image, self.image_tag)
    }

    /// The localized "write a hello world program" prompt for a language.
    #[must_use]
    pub fn prompt_for(&self, locale: &str, project: &BoxLanguageProject) -> Option<String> {
        self.prompts
            .iter()
            .find(|prompt| prompt.locale == locale)
            .map(|prompt| {
                prompt
                    .template
                    .replace("{display_name}", &project.display_name)
            })
    }
}

/// What the Docker Hub tag survey found, so the contract cannot name an image
/// nobody publishes and cannot drift off the surveyed tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxImageSurvey {
    /// Namespace every surveyed repository lives in.
    pub namespace: String,
    /// Tag the language matrix pins.
    pub pinned_tag: String,
    /// Committed log the survey was read from.
    pub evidence: String,
    /// Repositories that exist, without the namespace prefix.
    pub published: Vec<String>,
    /// Repositories that were looked for and do not exist.
    pub missing: Vec<String>,
}

impl BoxImageSurvey {
    /// Parse the embedded survey.
    #[must_use]
    pub fn load() -> Self {
        Self::parse(BOX_IMAGE_SURVEY_LINO)
    }

    /// Parse an explicit survey document; production uses [`Self::load`].
    #[must_use]
    pub fn parse(document: &str) -> Self {
        let root = parse_lino(document);
        let header = root
            .children
            .iter()
            .find(|node| node.find_child_value("record_type") == "box_image_survey")
            .expect("the survey must declare a box_image_survey header");
        let repositories = |published: &str| {
            root.children
                .iter()
                .find(|node| {
                    node.find_child_value("record_type") == "box_image_availability"
                        && node.find_child_value("published") == published
                })
                .map(|node| split_tuple(node.find_child_value("repositories")))
                .unwrap_or_default()
        };
        Self {
            namespace: header.find_child_value("namespace").to_owned(),
            pinned_tag: header.find_child_value("pinned_tag").to_owned(),
            evidence: header.find_child_value("evidence").to_owned(),
            published: repositories("true"),
            missing: repositories("false"),
        }
    }

    /// Whether the survey found this `namespace/repository` reference published.
    #[must_use]
    pub fn publishes(&self, image: &str) -> bool {
        image
            .strip_prefix(&format!("{}/", self.namespace))
            .is_some_and(|repository| {
                self.published
                    .iter()
                    .any(|published| published == repository)
            })
    }
}

fn parse_project(node: &LinoNode) -> BoxLanguageProject {
    let mut init_steps = Vec::new();
    for index in 1.. {
        let description = node.find_child_value(&format!("init_step_{index}"));
        let command = node.find_child_value(&format!("init_command_{index}"));
        if description.is_empty() || command.is_empty() {
            break;
        }
        init_steps.push(BoxInitStep {
            description: description.to_owned(),
            command: command.to_owned(),
        });
    }
    let shell_prelude = node.find_child_value("shell_prelude");
    BoxLanguageProject {
        language: node.find_child_value("language").to_owned(),
        display_name: node.find_child_value("display_name").to_owned(),
        image: node.find_child_value("image").to_owned(),
        code_fence: node.find_child_value("code_fence").to_owned(),
        program_file: node.find_child_value("program_file").to_owned(),
        prompt_locale: node.find_child_value("prompt_locale").to_owned(),
        shell_prelude: (!shell_prelude.is_empty()).then(|| shell_prelude.to_owned()),
        network_required: node.find_child_value("network_required") == "true",
        init_steps,
    }
}

/// Split a Links Notation tuple value such as `("en" "ru")` into its elements.
fn split_tuple(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|element| element.trim_matches(['(', ')', '"']).to_owned())
        .filter(|element| !element.is_empty())
        .collect()
}

/// The contract every consumer shares.
#[must_use]
pub fn box_language_contract() -> BoxLanguageContract {
    BoxLanguageContract::load()
}

/// The image survey the contract is checked against.
#[must_use]
pub fn box_image_survey() -> BoxImageSurvey {
    BoxImageSurvey::load()
}
