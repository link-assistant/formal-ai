#!/usr/bin/env rust-script
//! Materialize issue #1 dataset source records as human-readable Links Notation.
//!
//! Usage:
//!   rust-script scripts/download-datasets.rs
//!   rust-script scripts/download-datasets.rs --output data
//!
//! ```cargo
//! [dependencies]
//! lino-objects-codec = "0.2.1"
//! ```

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use lino_objects_codec::format::format_indented_ordered;

const MAX_LINO_LINES: usize = 1_500;

struct DatasetSource {
    id: &'static str,
    title: &'static str,
    kind: &'static str,
    source_url: &'static str,
    download_url: &'static str,
    license_notes: &'static str,
    conversion_notes: &'static str,
}

struct HelloWorldSeed {
    id: &'static str,
    language: &'static str,
    aliases: &'static [&'static str],
    code_fence: &'static str,
    code: &'static str,
}

struct DemoDialogSeed {
    id: &'static str,
    greeting: &'static str,
    request: &'static str,
    language: &'static str,
    expected_intent: &'static str,
}

const DATASET_SOURCES: &[DatasetSource] = &[
    DatasetSource {
        id: "issue_1",
        title: "formal-ai issue 1",
        kind: "requirements",
        source_url: "https://github.com/link-assistant/formal-ai/issues/1",
        download_url: "gh issue view https://github.com/link-assistant/formal-ai/issues/1 --json number,title,body,comments",
        license_notes: "repository-local issue text",
        conversion_notes: "store requirement statements and PR feedback as Links Notation records",
    },
    DatasetSource {
        id: "openai_chat_completions",
        title: "OpenAI Chat Completions API shape",
        kind: "api-reference",
        source_url: "https://platform.openai.com/docs/api-reference/chat/create",
        download_url: "official documentation page",
        license_notes: "reference only; do not vendor documentation text",
        conversion_notes: "convert field names and compatibility notes into schema records",
    },
    DatasetSource {
        id: "openai_responses",
        title: "OpenAI Responses API shape",
        kind: "api-reference",
        source_url: "https://platform.openai.com/docs/api-reference/responses/create",
        download_url: "official documentation page",
        license_notes: "reference only; do not vendor documentation text",
        conversion_notes: "convert field names and compatibility notes into schema records",
    },
    DatasetSource {
        id: "agent",
        title: "link-assistant agent",
        kind: "agent-integration",
        source_url: "https://github.com/link-assistant/agent",
        download_url: "gh repo view link-assistant/agent --json name,description,licenseInfo,url",
        license_notes: "own project; verify repository license before copying source text",
        conversion_notes: "index integration expectations and HTTP client assumptions",
    },
    DatasetSource {
        id: "wikidata",
        title: "Wikidata entity dumps",
        kind: "knowledge-graph",
        source_url: "https://www.wikidata.org/wiki/Wikidata:Database_download",
        download_url: "https://dumps.wikimedia.org/wikidatawiki/entities/latest-all.json.bz2",
        license_notes: "external data; preserve attribution and license metadata",
        conversion_notes: "stream entities into chunked entity records with stable Wikidata IDs",
    },
    DatasetSource {
        id: "wikipedia",
        title: "Wikipedia dumps",
        kind: "corpus",
        source_url: "https://www.wikipedia.org/",
        download_url: "https://dumps.wikimedia.org/enwiki/latest/",
        license_notes: "external data; preserve attribution and license metadata",
        conversion_notes: "chunk pages into document, paragraph, sentence, and phrase links",
    },
    DatasetSource {
        id: "rosetta_code",
        title: "Rosetta Code tasks",
        kind: "code-corpus",
        source_url: "https://rosettacode.org/wiki/Rosetta_Code",
        download_url: "https://rosettacode.org/wiki/Hello_world/Text",
        license_notes: "external data; transform and retain source/license fields",
        conversion_notes: "convert task, language, solution, and test intent records",
    },
    DatasetSource {
        id: "wikifunctions",
        title: "Wikifunctions",
        kind: "function-corpus",
        source_url: "https://www.wikifunctions.org/wiki/Wikifunctions:Main_Page",
        download_url: "https://www.wikifunctions.org/wiki/Special:AllPages",
        license_notes: "external data; preserve attribution and license metadata",
        conversion_notes: "convert functions, implementations, tests, and natural-language labels",
    },
    DatasetSource {
        id: "hello_world_collection",
        title: "Hello World Collection",
        kind: "hello-world-corpus",
        source_url: "http://helloworldcollection.de",
        download_url: "http://helloworldcollection.de",
        license_notes: "external data; use transformed Links Notation records, not verbatim pages",
        conversion_notes: "seed language aliases and self-authored hello-world templates",
    },
    DatasetSource {
        id: "hive_mind_issues",
        title: "link-assistant hive-mind issues",
        kind: "task-corpus",
        source_url: "https://github.com/link-assistant/hive-mind/issues",
        download_url: "gh api repos/link-assistant/hive-mind/issues --paginate",
        license_notes: "own project issue data",
        conversion_notes: "convert task descriptions and labels into problem-solving traces",
    },
    DatasetSource {
        id: "hive_mind_pulls",
        title: "link-assistant hive-mind pull requests",
        kind: "review-corpus",
        source_url: "https://github.com/link-assistant/hive-mind/pulls",
        download_url: "gh api repos/link-assistant/hive-mind/pulls --paginate",
        license_notes: "own project PR data",
        conversion_notes: "convert diffs, comments, logs, and outcomes into reasoning traces",
    },
    DatasetSource {
        id: "calculator",
        title: "link-assistant calculator",
        kind: "related-project",
        source_url: "https://github.com/link-assistant/calculator",
        download_url: "gh repo view link-assistant/calculator --json name,description,licenseInfo,url",
        license_notes: "own project; verify repository license before copying source text",
        conversion_notes: "index symbolic arithmetic patterns and tests",
    },
    DatasetSource {
        id: "human_language",
        title: "link-assistant human-language",
        kind: "related-project",
        source_url: "https://github.com/link-assistant/human-language",
        download_url: "gh repo view link-assistant/human-language --json name,description,licenseInfo,url",
        license_notes: "own project; verify repository license before copying source text",
        conversion_notes: "index language parsing concepts and examples",
    },
    DatasetSource {
        id: "meta_expression",
        title: "link-assistant meta-expression",
        kind: "related-project",
        source_url: "https://github.com/link-assistant/meta-expression",
        download_url: "gh repo view link-assistant/meta-expression --json name,description,licenseInfo,url",
        license_notes: "own project; verify repository license before copying source text",
        conversion_notes: "index expression representation and transformation ideas",
    },
    DatasetSource {
        id: "relative_meta_logic",
        title: "relative-meta-logic",
        kind: "prover",
        source_url: "https://github.com/link-foundation/relative-meta-logic",
        download_url: "gh repo view link-foundation/relative-meta-logic --json name,description,licenseInfo,url",
        license_notes: "related project; verify repository license before copying source text",
        conversion_notes: "index formal statement and prover integration boundaries",
    },
    DatasetSource {
        id: "command_stream",
        title: "command-stream",
        kind: "dependency",
        source_url: "https://github.com/link-foundation/command-stream",
        download_url: "gh repo view link-foundation/command-stream --json name,description,licenseInfo,url",
        license_notes: "related project; verify repository license before copying source text",
        conversion_notes: "index command protocol concepts for agent tool use",
    },
    DatasetSource {
        id: "link_cli",
        title: "link-cli",
        kind: "dependency",
        source_url: "https://github.com/link-foundation/link-cli",
        download_url: "gh repo view link-foundation/link-cli --json name,description,licenseInfo,url",
        license_notes: "related project; verify repository license before copying source text",
        conversion_notes: "index import/export CLI expectations for link stores",
    },
    DatasetSource {
        id: "lino_objects_codec",
        title: "lino-objects-codec",
        kind: "dependency",
        source_url: "https://github.com/link-foundation/lino-objects-codec",
        download_url: "https://crates.io/api/v1/crates/lino-objects-codec",
        license_notes: "dependency metadata only",
        conversion_notes: "use indented untyped formatting helpers for reviewable records",
    },
    DatasetSource {
        id: "links_notation",
        title: "links-notation",
        kind: "dependency",
        source_url: "https://github.com/link-foundation/links-notation",
        download_url: "https://crates.io/api/v1/crates/links-notation",
        license_notes: "dependency metadata only",
        conversion_notes: "parse and validate Links Notation records",
    },
    DatasetSource {
        id: "doublets_rs",
        title: "doublets-rs",
        kind: "dependency",
        source_url: "https://github.com/linksplatform/doublets-rs",
        download_url: "gh repo view linksplatform/doublets-rs --json name,description,licenseInfo,url",
        license_notes: "related project; verify repository license before copying source text",
        conversion_notes: "index doublet storage concepts for later native stores",
    },
    DatasetSource {
        id: "doublets_web",
        title: "doublets-web",
        kind: "dependency",
        source_url: "https://github.com/linksplatform/doublets-web",
        download_url: "gh repo view linksplatform/doublets-web --json name,description,licenseInfo,url",
        license_notes: "related project; verify repository license before copying source text",
        conversion_notes: "index browser-side doublet storage concepts",
    },
    DatasetSource {
        id: "react_chat_ui",
        title: "link-assistant react-chat-ui",
        kind: "ui-reference",
        source_url: "https://github.com/link-assistant/react-chat-ui",
        download_url: "gh repo view link-assistant/react-chat-ui --json name,description,licenseInfo,url",
        license_notes: "own Unlicense project",
        conversion_notes: "reuse markdown message and markdown composer UX ideas",
    },
    DatasetSource {
        id: "vk_bot_desktop",
        title: "vk-bot-desktop",
        kind: "desktop-reference",
        source_url: "https://github.com/konard/vk-bot-desktop",
        download_url: "gh repo view konard/vk-bot-desktop --json name,description,licenseInfo,url",
        license_notes: "related project; verify repository license before copying source text",
        conversion_notes: "index desktop wrapper requirements for a later shell",
    },
];

const HELLO_WORLD_SEEDS: &[HelloWorldSeed] = &[
    HelloWorldSeed {
        id: "hello_world_rust",
        language: "Rust",
        aliases: &["rust", "rs"],
        code_fence: "rust",
        code: r#"fn main() {\n    println!("Hello, world!");\n}"#,
    },
    HelloWorldSeed {
        id: "hello_world_python",
        language: "Python",
        aliases: &["python", "py"],
        code_fence: "python",
        code: r#"print("Hello, world!")"#,
    },
    HelloWorldSeed {
        id: "hello_world_javascript",
        language: "JavaScript",
        aliases: &["javascript", "js", "node"],
        code_fence: "javascript",
        code: r#"console.log("Hello, world!");"#,
    },
    HelloWorldSeed {
        id: "hello_world_typescript",
        language: "TypeScript",
        aliases: &["typescript", "ts"],
        code_fence: "typescript",
        code: r#"console.log("Hello, world!");"#,
    },
    HelloWorldSeed {
        id: "hello_world_go",
        language: "Go",
        aliases: &["go", "golang"],
        code_fence: "go",
        code: r#"package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello, world!")\n}"#,
    },
    HelloWorldSeed {
        id: "hello_world_c",
        language: "C",
        aliases: &["c"],
        code_fence: "c",
        code: r#"#include <stdio.h>\n\nint main(void) {\n    puts("Hello, world!");\n    return 0;\n}"#,
    },
];

const DEMO_DIALOG_SEEDS: &[DemoDialogSeed] = &[
    DemoDialogSeed {
        id: "dialog_rust",
        greeting: "Hi",
        request: "Write me hello world program in Rust",
        language: "Rust",
        expected_intent: "hello_world_rust",
    },
    DemoDialogSeed {
        id: "dialog_python",
        greeting: "Hello",
        request: "Create a hello world example in Python",
        language: "Python",
        expected_intent: "hello_world_python",
    },
    DemoDialogSeed {
        id: "dialog_javascript",
        greeting: "Hi",
        request: "Write hello world in JavaScript",
        language: "JavaScript",
        expected_intent: "hello_world_javascript",
    },
    DemoDialogSeed {
        id: "dialog_go",
        greeting: "Hello",
        request: "Show hello world in Go",
        language: "Go",
        expected_intent: "hello_world_go",
    },
    DemoDialogSeed {
        id: "dialog_typescript",
        greeting: "Hi",
        request: "Write hello world in TypeScript",
        language: "TypeScript",
        expected_intent: "hello_world_typescript",
    },
    DemoDialogSeed {
        id: "dialog_c",
        greeting: "Hello",
        request: "Show hello world in C",
        language: "C",
        expected_intent: "hello_world_c",
    },
];

fn main() -> Result<(), Box<dyn Error>> {
    let data_dir = output_dir();
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(data_dir.join("seed"))?;

    write_lino_file(
        &data_dir.join("source-index.lino"),
        source_index_records()?.join("\n\n"),
    )?;
    write_lino_file(
        &data_dir.join("seed/greetings.lino"),
        greeting_records()?.join("\n\n"),
    )?;
    write_lino_file(
        &data_dir.join("seed/hello-world-programs.lino"),
        hello_world_records()?.join("\n\n"),
    )?;
    write_lino_file(
        &data_dir.join("seed/demo-dialogs.lino"),
        demo_dialog_records()?.join("\n\n"),
    )?;

    println!("wrote Links Notation datasets to {}", data_dir.display());
    Ok(())
}

fn output_dir() -> PathBuf {
    let args = env::args().collect::<Vec<_>>();
    args.windows(2)
        .find_map(|window| (window[0] == "--output").then(|| PathBuf::from(&window[1])))
        .unwrap_or_else(|| PathBuf::from("data"))
}

fn source_index_records() -> Result<Vec<String>, Box<dyn Error>> {
    DATASET_SOURCES
        .iter()
        .map(|source| {
            record(
                &format!("source_{}", source.id),
                &[
                    ("title", source.title),
                    ("kind", source.kind),
                    ("source_url", source.source_url),
                    ("download", source.download_url),
                    ("license", source.license_notes),
                    ("conversion", source.conversion_notes),
                ],
            )
        })
        .collect()
}

fn greeting_records() -> Result<Vec<String>, Box<dyn Error>> {
    ["Hi", "Hello", "Hey"]
        .iter()
        .enumerate()
        .map(|(index, greeting)| {
            record(
                &format!("greeting_{}", index + 1),
                &[
                    ("text", *greeting),
                    ("intent", "greeting"),
                    ("response_link", "response:greeting"),
                    ("answer", "Hi, how may I help you?"),
                ],
            )
        })
        .collect()
}

fn hello_world_records() -> Result<Vec<String>, Box<dyn Error>> {
    HELLO_WORLD_SEEDS
        .iter()
        .map(|seed| {
            record(
                seed.id,
                &[
                    ("language", seed.language),
                    ("aliases", &seed.aliases.join(", ")),
                    ("code_fence", seed.code_fence),
                    ("code", seed.code),
                    (
                        "source",
                        "local public-domain seed inspired by hello-world corpora",
                    ),
                ],
            )
        })
        .collect()
}

fn demo_dialog_records() -> Result<Vec<String>, Box<dyn Error>> {
    DEMO_DIALOG_SEEDS
        .iter()
        .map(|seed| {
            record(
                seed.id,
                &[
                    ("greeting", seed.greeting),
                    ("request", seed.request),
                    ("language", seed.language),
                    ("expected_intent", seed.expected_intent),
                    ("cycle_wait_seconds", "10-20"),
                ],
            )
        })
        .collect()
}

fn record(id: &str, pairs: &[(&str, &str)]) -> Result<String, Box<dyn Error>> {
    format_indented_ordered(id, pairs, "  ").map_err(Into::into)
}

fn write_lino_file(path: &Path, content: String) -> Result<(), Box<dyn Error>> {
    let line_count = content.lines().count();
    if line_count > MAX_LINO_LINES {
        return Err(format!(
            "{} has {line_count} lines, exceeding {MAX_LINO_LINES}",
            path.display()
        )
        .into());
    }

    fs::write(path, format!("{content}\n"))?;
    Ok(())
}
