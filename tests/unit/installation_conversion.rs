use std::fmt::Write as _;

use formal_ai::FormalAiEngine;

#[derive(Clone, Copy)]
pub(super) enum DocumentedFormat {
    Markdown,
    Shell,
    PowerShell,
}

impl DocumentedFormat {
    const fn label(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Shell => "shell_script",
            Self::PowerShell => "powershell_script",
        }
    }
}

/// Construct the complete documented answer for one conversion example.
///
/// The constant prose and every emitted byte live in this test fixture; only
/// the explicitly listed project, formats, descriptions, and commands vary.
/// This keeps the large cross-language/corpus loops readable while still
/// pinning a complete public answer rather than a handful of substrings.
pub(super) fn documented_conversion_answer(
    source: DocumentedFormat,
    targets: &[DocumentedFormat],
    project: &str,
    steps: &[(&str, &str)],
) -> String {
    let mut answer = format!("Converted installation instructions for {project}.\n\n");
    answer.push_str("Formalized meaning:\n```lino\ninstallation_conversion_request\n");
    let _ = writeln!(answer, "  source_format {}", source.label());
    for target in targets {
        let _ = writeln!(answer, "  target_format {}", target.label());
    }
    let _ = writeln!(answer, "  project \"{project}\"");
    answer.push_str(
        "  validation \"ordered_commands_preserved\"\n  validation \"single_ir_renders_markdown_shell_powershell\"\n",
    );
    // Keep the meta-algorithm projection visible in the expected answer. These
    // are the exact public lines, not a call back into the production renderer.
    answer.push_str(
        concat!(
            "  meta_algorithm \"problem_class_to_shared_ir_to_renderers_to_verification\"\n",
            "  active_coding_surface \"installation_conversion\"\n",
            "  construction_stage \"collect_corpus\"\n",
            "  stage_output \"representative problem-class examples\"\n",
            "  stage_verifier \"case-study corpus preserved\"\n",
            "  construction_stage \"derive_surfaces\"\n",
            "  stage_output \"input and output surface ontology\"\n",
            "  stage_verifier \"handler-specific surface recognition fixture\"\n",
            "  construction_stage \"extract_ir\"\n",
            "  stage_output \"shared intermediate representation\"\n",
            "  stage_verifier \"domain invariants preserved\"\n",
            "  construction_stage \"synthesize_operations\"\n",
            "  stage_output \"recognizers, extractors, renderers, and validators\"\n",
            "  stage_verifier \"operation composition invariants\"\n",
            "  construction_stage \"project_targets\"\n",
            "  stage_output \"target-specific code, document, or rule projections\"\n",
            "  stage_verifier \"per-target rendering or execution fixture\"\n",
            "  construction_stage \"mirror_runtimes\"\n",
            "  stage_output \"Rust and browser-worker projections of the same algorithm\"\n",
            "  stage_verifier \"cross-runtime parity checks\"\n",
            "  construction_stage \"promote_capability\"\n",
            "  stage_output \"reusable coding-task construction pattern\"\n",
            "  stage_verifier \"shared-symbol and trace-shape compatibility\"\n",
            "  coding_surface \"coding_catalog\"\n",
            "  surface_projection \"task spec -> parameterized template -> CST/compile check\"\n",
            "  coding_surface \"program_synthesis\"\n",
            "  surface_projection \"semantic function tree -> source program -> sandbox tests\"\n",
            "  coding_surface \"program_blueprint\"\n",
            "  surface_projection \"capability set -> blueprint recipe -> honest code projection\"\n",
            "  coding_surface \"numeric_list\"\n",
            "  surface_projection \"operation/data/language IR -> generated code plus evaluated result\"\n",
            "  coding_surface \"rule_synthesis\"\n",
            "  surface_projection \"operation/target binding -> candidate rule -> verification fixture\"\n",
            "  coding_surface \"installation_conversion\"\n",
            "  surface_projection \"installation surfaces -> install-step IR -> target renderers\"\n",
        ),
    );
    for (index, (description, command)) in steps.iter().enumerate() {
        let _ = writeln!(answer, "  step \"S{}\"", index + 1);
        let _ = writeln!(answer, "  description \"{description}\"");
        let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
        let _ = writeln!(answer, "  command \"{escaped}\"");
    }
    answer.push_str(
        "```\n\nConversion algorithm:\n\
         1. Detect the source surface and requested target surface(s).\n\
         2. Extract command-like install/deploy steps in original order.\n\
         3. Render every target from the same install-step IR.\n\
         4. Preserve commands verbatim so the conversion can round-trip.\n\n\
         Meta algorithm for constructing conversion algorithms:\n\
         1. collect_corpus -> representative problem-class examples; verification fixture: case-study corpus preserved.\n\
         2. derive_surfaces -> input and output surface ontology; verification fixture: handler-specific surface recognition fixture.\n\
         3. extract_ir -> shared intermediate representation; verification fixture: domain invariants preserved.\n\
         4. synthesize_operations -> recognizers, extractors, renderers, and validators; verification fixture: operation composition invariants.\n\
         5. project_targets -> target-specific code, document, or rule projections; verification fixture: per-target rendering or execution fixture.\n\
         6. mirror_runtimes -> Rust and browser-worker projections of the same algorithm; verification fixture: cross-runtime parity checks.\n\
         7. promote_capability -> reusable coding-task construction pattern; verification fixture: shared-symbol and trace-shape compatibility.\n\n\
         Coding solutions using the same meta algorithm:\n\
         - coding_catalog: task spec -> parameterized template -> CST/compile check.\n\
         - program_synthesis: semantic function tree -> source program -> sandbox tests.\n\
         - program_blueprint: capability set -> blueprint recipe -> honest code projection.\n\
         - numeric_list: operation/data/language IR -> generated code plus evaluated result.\n\
         - rule_synthesis: operation/target binding -> candidate rule -> verification fixture.\n\
         - installation_conversion [active]: installation surfaces -> install-step IR -> target renderers.\n",
    );
    for target in targets {
        answer.push('\n');
        match target {
            DocumentedFormat::Markdown => {
                answer.push_str("README.md installation guide:\n\n## Installation\n\n");
                for (index, (description, command)) in steps.iter().enumerate() {
                    let _ = write!(
                        answer,
                        "{}. {}.\n\n   ```sh\n   {}\n   ```\n",
                        index + 1,
                        description,
                        command
                    );
                }
            }
            DocumentedFormat::Shell => {
                answer
                    .push_str("Bash script:\n```bash\n#!/usr/bin/env bash\nset -euo pipefail\n\n");
                for (description, command) in steps {
                    let _ = writeln!(answer, "# {description}\n{command}");
                }
                answer.push_str("```\n");
            }
            DocumentedFormat::PowerShell => {
                answer.push_str(
                    "PowerShell script:\n```powershell\n$ErrorActionPreference = 'Stop'\n\n",
                );
                for (description, command) in steps {
                    let _ = writeln!(answer, "# {description}\n{command}");
                }
                answer.push_str("```\n");
            }
        }
    }
    answer.trim_end().to_owned()
}

#[test]
fn readme_install_guide_converts_to_bash_and_powershell() {
    let prompt = r"Convert this README.md installation guide into both sh and PowerShell scripts:

```markdown
## Installation

1. Clone the project.
   `git clone https://github.com/example/widget.git`
2. Enter the directory.
   `cd widget`
3. Install dependencies.
   `npm install`
4. Build the project.
   `npm run build`
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.answer,
        documented_conversion_answer(
            DocumentedFormat::Markdown,
            &[DocumentedFormat::Shell, DocumentedFormat::PowerShell],
            "the project",
            &[
                (
                    "Clone the repository",
                    "git clone https://github.com/example/widget.git",
                ),
                ("Enter the project directory", "cd widget"),
                ("Install dependencies", "npm install"),
                ("Build the project", "npm run build"),
            ],
        )
    );
    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("installation_conversion_request"));
    assert!(response.answer.contains("source_format markdown"));
    assert!(response.answer.contains("target_format shell_script"));
    assert!(response.answer.contains("target_format powershell_script"));
    assert!(response.answer.contains("```bash"));
    assert!(response.answer.contains("```powershell"));
    assert!(
        response
            .answer
            .contains("git clone https://github.com/example/widget.git")
    );
    assert!(response.answer.contains("npm run build"));
}

#[test]
fn wrapped_readme_with_nested_shell_fences_converts_to_scripts() {
    let prompt = r"Convert this README.md installation guide for react/react into both sh and PowerShell scripts:

```markdown
## Installation

1. Clone the repository.

   ```sh
   git clone https://github.com/react/react.git
   cd react
   ```

2. Install and verify.

   ```sh
   yarn install
   yarn test
   ```
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.answer,
        documented_conversion_answer(
            DocumentedFormat::Markdown,
            &[DocumentedFormat::Shell, DocumentedFormat::PowerShell],
            "react/react",
            &[
                (
                    "Clone the repository",
                    "git clone https://github.com/react/react.git",
                ),
                ("Enter the project directory", "cd react"),
                ("Install dependencies", "yarn install"),
                ("Run the verification command", "yarn test"),
            ],
        )
    );
    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("target_format shell_script"));
    assert!(response.answer.contains("target_format powershell_script"));
    assert!(
        response
            .answer
            .contains("git clone https://github.com/react/react.git")
    );
    assert!(response.answer.contains("cd react"));
    assert!(response.answer.contains("yarn install"));
    assert!(response.answer.contains("yarn test"));
}

#[test]
fn unwrapped_readme_with_shell_fences_stays_markdown_source() {
    let prompt = r"Convert this README.md installation guide for example/widget into a sh script:

## Installation

Clone the repository:

```sh
git clone https://github.com/example/widget.git
cd widget
```

Install and verify:

```sh
npm install
npm test
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.answer,
        documented_conversion_answer(
            DocumentedFormat::Markdown,
            &[DocumentedFormat::Shell],
            "example/widget",
            &[
                (
                    "Clone the repository",
                    "git clone https://github.com/example/widget.git",
                ),
                ("Enter the project directory", "cd widget"),
                ("Install dependencies", "npm install"),
                ("Run the verification command", "npm test"),
            ],
        )
    );
    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("source_format markdown"));
    assert!(response.answer.contains("target_format shell_script"));
    assert!(
        response
            .answer
            .contains("git clone https://github.com/example/widget.git")
    );
    assert!(response.answer.contains("cd widget"));
    assert!(response.answer.contains("npm install"));
    assert!(response.answer.contains("npm test"));
}

#[test]
fn install_script_converts_back_to_readme_guide() {
    let prompt = r"Convert this shell installation script back to a README.md installation guide:

```bash
#!/usr/bin/env bash
set -euo pipefail
git clone https://github.com/ollama/ollama.git
cd ollama
curl -fsSL https://ollama.com/install.sh | sh
ollama serve
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.answer,
        documented_conversion_answer(
            DocumentedFormat::Shell,
            &[DocumentedFormat::Markdown],
            "the project",
            &[
                (
                    "Clone the repository",
                    "git clone https://github.com/ollama/ollama.git",
                ),
                ("Enter the project directory", "cd ollama"),
                (
                    "Run the curl https://ollama.com/install.sh step",
                    "curl -fsSL https://ollama.com/install.sh | sh",
                ),
                ("Start the application", "ollama serve"),
            ],
        )
    );
    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("installation_conversion_request"));
    assert!(response.answer.contains("source_format shell_script"));
    assert!(response.answer.contains("target_format markdown"));
    assert!(response.answer.contains("README.md installation guide"));
    assert!(
        response
            .answer
            .contains("git clone https://github.com/ollama/ollama.git")
    );
    assert!(response.answer.contains("ollama serve"));
}

#[test]
fn powershell_install_script_converts_back_to_readme_guide() {
    let prompt = r#"Convert this PowerShell installation script back to a README.md installation guide:

```powershell
$ErrorActionPreference = 'Stop'
irm https://get.activated.win | iex
powershell -NoProfile -Command "$PSVersionTable.PSVersion"
```
"#;

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.answer,
        documented_conversion_answer(
            DocumentedFormat::PowerShell,
            &[DocumentedFormat::Markdown],
            "the project",
            &[
                (
                    "Run the irm https://get.activated.win step",
                    "irm https://get.activated.win | iex",
                ),
                (
                    "Run the powershell $psversiontable.psversion step",
                    "powershell -NoProfile -Command \"$PSVersionTable.PSVersion\"",
                ),
            ],
        )
    );
    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("source_format powershell_script"));
    assert!(response.answer.contains("target_format markdown"));
    assert!(response.answer.contains("README.md installation guide"));
    assert!(
        response
            .answer
            .contains("irm https://get.activated.win | iex")
    );
    assert!(
        response
            .answer
            .contains("powershell -NoProfile -Command \"$PSVersionTable.PSVersion\"")
    );
}

#[test]
fn conversion_answer_exposes_algorithm_construction_trace() {
    let prompt = r"Convert this README.md installation guide into a sh script and show the meta algorithm:

```markdown
## Installation

1. Install dependencies.
   `python -m pip install -r requirements.txt`
2. Verify the package.
   `python -m pytest`
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.answer,
        documented_conversion_answer(
            DocumentedFormat::Markdown,
            &[DocumentedFormat::Shell],
            "the project",
            &[
                (
                    "Install dependencies",
                    "python -m pip install -r requirements.txt",
                ),
                ("Run the verification command", "python -m pytest"),
            ],
        )
    );
    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("Meta algorithm for constructing conversion algorithms"),
        "answer should expose the algorithm-construction layer, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("shared intermediate representation")
            && response.answer.contains("verification fixture"),
        "meta algorithm should name IR construction and verification, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("program_blueprint")
            && response.answer.contains("rule_synthesis")
            && response.answer.contains("numeric_list"),
        "meta algorithm should connect existing coding surfaces, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("algorithm_construction:stage"),
        "trace should record algorithm construction stages, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("coding_surface")
            && response.links_notation.contains("program_blueprint"),
        "formal meaning should record producible coding surfaces, got: {}",
        response.links_notation
    );
}

#[test]
fn install_conversion_prompts_route_across_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
        command: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            prompt: "Convert this README.md installation guide into a sh script:\n\
                     ## Installation\n\
                     1. Run `npm install`.\n",
            command: "npm install",
        },
        Case {
            language: "ru",
            prompt: "Преобразуй это README.md руководство по установке в sh скрипт:\n\
                     ## Установка\n\
                     1. Выполни `npm install`.\n",
            command: "npm install",
        },
        Case {
            language: "hi",
            prompt: "इस README.md स्थापना guide को sh script में बदलें:\n\
                     ## स्थापना\n\
                     1. चलाएं `npm install`.\n",
            command: "npm install",
        },
        Case {
            language: "zh",
            prompt: "请把这个 README.md 安装指南转换为 sh 脚本:\n\
                     ## 安装\n\
                     1. 运行 `npm install`.\n",
            command: "npm install",
        },
    ];

    for case in cases {
        let response = FormalAiEngine.answer(case.prompt);

        if case.language == "en" {
            assert_eq!(
                response.answer,
                documented_conversion_answer(
                    DocumentedFormat::Markdown,
                    &[DocumentedFormat::Shell],
                    "the project",
                    &[("Install dependencies", "npm install")],
                )
            );
        }
        assert_eq!(
            response.intent, "installation_conversion",
            "language: {}, answer was: {}",
            case.language, response.answer
        );
        assert!(response.answer.contains("source_format markdown"));
        assert!(response.answer.contains("target_format shell_script"));
        assert!(response.answer.contains(case.command));
    }
}

mod extended;
