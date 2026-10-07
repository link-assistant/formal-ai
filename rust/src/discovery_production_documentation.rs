// Rediscovery from documentation on the `write_program` miss path (issue
// #1165 R1165-1/R1165-2), included into `discovery_production`.
//
// A cache miss whose `(language, task)` pair has documentation captures in
// `data/seed/coding-documentation-captures.lino` rediscovers its program at
// answer time: the captures are pre-cached source data (the code blocks the
// page formalizer reads from a page fetched byte for byte), and the program
// is derived from them by the #1164 decomposer, never stored. The policy
// seed's `documentation_route` record says whether the route is active and
// how a rediscovered program is verified. The JavaScript twin is
// `rediscoverDocumentedProgram` in js/worker/formal_ai_worker_code_examples.js.

/// The documentation captures seed, mirrored into the embedded bundle.
const DOCUMENTATION_CAPTURES: &str =
    include_str!("../embedded/data/seed/coding-documentation-captures.lino");

/// The environment variable naming a writable runtime procedure cache.
const RUNTIME_CACHE_ENV: &str = "FORMAL_AI_PROCEDURE_CACHE";

/// Why no program was rediscovered: the pair has no captured page (or the
/// route is retired in the policy seed), so the miss is the research miss.
pub const NO_DOCUMENTATION_CAPTURE: &str = "no_documentation_capture";

/// One documentation page captured for a `(language, task)` pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationCapture {
    /// The language the page documents.
    pub language: String,
    /// The task it was captured for.
    pub task: String,
    /// The page's URL, the row's `rediscovery_source`.
    pub url: String,
    /// The mime type the page was served as.
    pub mime: String,
    /// The repository path of the byte-for-byte capture.
    pub fixture: String,
    /// The SHA-256 of the captured bytes.
    pub sha256: String,
    /// The query that finds the page again, the row's `rediscovery_query`.
    pub rediscovery_query: String,
    /// `(language, text)` of every code block on the page, in page order.
    pub blocks: Vec<(String, String)>,
}

/// What a language's catalog row asks of a program it runs.
#[derive(Debug, Clone, Copy)]
pub struct RunContract<'a> {
    /// The file the program is saved as (`Main.scala`).
    pub save_as: &'a str,
    /// The commands that check and run the saved file.
    pub commands: &'a [&'a str],
}

impl RunContract<'_> {
    /// The names a program must declare: the stem of `save_as`, when a
    /// command names it bare (`scala Main` runs the object `Main`;
    /// `rustc main.rs -o main` names the binary `fn main` builds).
    #[must_use]
    pub fn invoked_names(&self) -> Vec<String> {
        let file = self.save_as.rsplit('/').next().unwrap_or_default();
        let stem = file.rfind('.').map_or(file, |dot| &file[..dot]);
        let named = self
            .commands
            .iter()
            .flat_map(|command| command.split_whitespace())
            .any(|word| word == stem);
        if stem.is_empty() || !named {
            return Vec::new();
        }
        vec![stem.to_owned()]
    }
}

/// Whether the policy seed keeps the documentation route active.
#[must_use]
pub fn documentation_route_active() -> bool {
    let policy = parse_lino(POLICY);
    policy
        .children
        .first()
        .and_then(|root| {
            root.children
                .iter()
                .find(|node| node.name == "documentation_route")
        })
        .is_some_and(|route| route.find_child_value("active") == "true")
}

/// The documentation captures for `language` and `task`, in seed order.
#[must_use]
pub fn documentation_captures(language: &str, task: &str) -> Vec<DocumentationCapture> {
    all_documentation_captures()
        .into_iter()
        .filter(|capture| capture.language == language && capture.task == task)
        .collect()
}

/// Every documentation capture the seed records, in seed order.
#[must_use]
pub fn all_documentation_captures() -> Vec<DocumentationCapture> {
    let seed = parse_lino(DOCUMENTATION_CAPTURES);
    let Some(root) = seed
        .children
        .iter()
        .find(|node| node.name == "coding_documentation_captures")
    else {
        return Vec::new();
    };
    root.children
        .iter()
        .filter(|node| node.name == "capture")
        .map(|node| DocumentationCapture {
            language: node.find_child_value("language").to_owned(),
            task: node.find_child_value("task").to_owned(),
            url: node.find_child_value("url").to_owned(),
            mime: node.find_child_value("mime").to_owned(),
            fixture: node.find_child_value("fixture").to_owned(),
            sha256: node.find_child_value("sha256").to_owned(),
            rediscovery_query: node.find_child_value("rediscovery_query").to_owned(),
            blocks: node
                .children
                .iter()
                .filter(|child| child.name == "block")
                .map(|block| {
                    (
                        block.find_child_value("language").to_owned(),
                        block.find_child_value("text").to_owned(),
                    )
                })
                .collect(),
        })
        .collect()
}

/// The page example a rediscovery recomposes.
///
/// In each capture it is the first block in `language` (or untagged) that
/// decomposes into an output call printing a literal; across captures, the
/// shortest program body wins (the smallest sufficient candidate), the
/// earlier capture on a tie.
#[cfg(feature = "meta-language")]
fn documented_example<'a>(
    captures: &'a [DocumentationCapture],
    language: &str,
) -> Option<(
    &'a DocumentationCapture,
    crate::code_example_knowledge::DecomposedCodeNode,
)> {
    use crate::code_example_knowledge::{CodePartKind, decompose_code_node_from};
    let mut best: Option<(
        &DocumentationCapture,
        crate::code_example_knowledge::DecomposedCodeNode,
    )> = None;
    for capture in captures {
        for (block_language, text) in &capture.blocks {
            if block_language != language && block_language != "unknown" {
                continue;
            }
            let Ok(node) = decompose_code_node_from(text, language, &[], &capture.url) else {
                continue;
            };
            let prints = node
                .parts
                .iter()
                .any(|part| part.kind == CodePartKind::OutputOperation);
            if node.program_body.is_empty() || !prints {
                continue;
            }
            let shorter = best.as_ref().is_none_or(|(_, kept)| {
                node.program_body.chars().count() < kept.program_body.chars().count()
            });
            if shorter {
                best = Some((capture, node));
            }
            break;
        }
    }
    best
}

/// Rediscover a `write_program` procedure from the documentation captures.
///
/// The page example (each capture's first block that prints a literal,
/// the shortest across captures) is recomposed with
/// `expected_output` bound into its literal slot, then verified: decomposing
/// the program again must find the same output call printing
/// `expected_output`, and the program must declare every name `contract`
/// invokes. The row names the page as its `rediscovery_source`.
///
/// # Errors
///
/// [`NO_DOCUMENTATION_CAPTURE`] when the pair has no capture (or the route is
/// retired), else why the captured pages yielded no program:
/// `no_single_line_output`, `no_output_example`, `no_program_body`,
/// `verification`, or `run_contract:<names>`.
#[cfg(feature = "meta-language")]
pub fn rediscover_from_documentation(
    language: &str,
    task: &str,
    expected_output: &str,
    contract: RunContract<'_>,
) -> Result<RediscoverableRecipe, String> {
    use crate::code_example_knowledge::{
        CodePartKind, ParameterBindings, decompose_code_node_from, generalize_examples,
        recompose_for_requirement,
    };
    let captures = if documentation_route_active() {
        documentation_captures(language, task)
    } else {
        Vec::new()
    };
    if captures.is_empty() {
        return Err(NO_DOCUMENTATION_CAPTURE.to_owned());
    }
    if expected_output.is_empty() || expected_output.contains('\n') {
        return Err(String::from("no_single_line_output"));
    }
    let (capture, node) =
        documented_example(&captures, language).ok_or_else(|| String::from("no_output_example"))?;
    let call = node
        .parts
        .iter()
        .find(|part| part.kind == CodePartKind::OutputOperation)
        .map(|part| part.source_text.clone())
        .unwrap_or_default();
    let bindings = ParameterBindings(
        [(String::from("output_literal"), expected_output.to_owned())]
            .into_iter()
            .collect(),
    );
    let procedure = generalize_examples(std::slice::from_ref(&node));
    let recomposed = recompose_for_requirement(&procedure, &bindings, language)
        .map_err(|_| String::from("no_program_body"))?;
    let verified = decompose_code_node_from(&recomposed.source, language, &[], &capture.url)
        .is_ok_and(|check| {
            check
                .parts
                .iter()
                .any(|part| part.kind == CodePartKind::OutputOperation && part.source_text == call)
                && check.parts.iter().any(|part| {
                    part.kind == CodePartKind::StringLiteral && part.source_text == expected_output
                })
        });
    if !verified {
        return Err(String::from("verification"));
    }
    let tokens: Vec<&str> = recomposed
        .source
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .collect();
    let undeclared: Vec<String> = contract
        .invoked_names()
        .into_iter()
        .filter(|name| !tokens.contains(&name.as_str()))
        .collect();
    if !undeclared.is_empty() {
        return Err(format!("run_contract:{}", undeclared.join(",")));
    }
    Ok(RediscoverableRecipe {
        language: language.to_owned(),
        task: task.to_owned(),
        rediscovery_query: capture.rediscovery_query.clone(),
        rediscovery_source: capture.url.clone(),
        content_id: RediscoverableRecipe::content_address(&recomposed.source),
        entry: recomposed.source,
        verified_output: expected_output.to_owned(),
    })
}

/// Without the meta-language parse there is no decomposer, so a captured
/// pair is refused as `no_decomposer` and any other pair is
/// [`NO_DOCUMENTATION_CAPTURE`].
///
/// # Errors
///
/// Always: rediscovery needs the decomposer.
#[cfg(not(feature = "meta-language"))]
pub fn rediscover_from_documentation(
    language: &str,
    task: &str,
    expected_output: &str,
    contract: RunContract<'_>,
) -> Result<RediscoverableRecipe, String> {
    let _ = (expected_output, contract);
    if documentation_route_active() && !documentation_captures(language, task).is_empty() {
        return Err(String::from("no_decomposer"));
    }
    Err(NO_DOCUMENTATION_CAPTURE.to_owned())
}

/// Store a row rediscovered on the solve path in the runtime cache.
///
/// Only an explicitly configured cache (`FORMAL_AI_PROCEDURE_CACHE`) is
/// written: the committed `data/cache/coding-procedure-cache.lino` is
/// shipped data, and a solve that rewrote it would change the repository it
/// runs in. The next request for the pair reads the stored row as a hit.
///
/// # Errors
///
/// `no_runtime_cache` when no runtime cache is configured, or the store's
/// own refusal.
pub fn record_discovered(recipe: &RediscoverableRecipe) -> Result<(), String> {
    let explicit = std::env::var(RUNTIME_CACHE_ENV).unwrap_or_default();
    if explicit.trim().is_empty() {
        return Err(String::from("no_runtime_cache"));
    }
    ProcedureCache::load_at(Path::new(&explicit)).store(recipe.clone())
}

/// The `write_program` cache miss of the solver (R1165-1/R1165-2).
///
/// An unmodified request the cache has no row for rediscovers its program
/// from the documentation captures: on success the `procedure_cache` event
/// is `outcome=discovered` with the page and the program's `content_id`, the
/// row is recorded in the runtime cache, and the program is returned for the
/// answer and the execution recipe. Otherwise the event is the research miss
/// naming the `miss_route` inputs a solve lacks, plus why captured pages
/// yielded nothing when there were some, and `None` keeps the template.
pub fn answer_cache_miss(
    log: &mut crate::event_log::EventLog,
    language: &str,
    task: &str,
    expected_output: &str,
    contract: RunContract<'_>,
) -> Option<String> {
    match rediscover_from_documentation(language, task, expected_output, contract) {
        Ok(recipe) => {
            let content_id = format!("0x{:016x}", recipe.content_id);
            log.append_fields(
                "procedure_cache",
                &[
                    ("outcome", "discovered"),
                    ("language", language),
                    ("task", task),
                    ("rediscovery_source", &recipe.rediscovery_source),
                    ("content_id", &content_id),
                ],
            );
            let _recorded = record_discovered(&recipe);
            Some(recipe.entry)
        }
        Err(reason) => {
            let missing = miss_research_missing().join(",");
            let mut fields = vec![
                ("outcome", "miss"),
                ("language", language),
                ("task", task),
                ("research_missing", missing.as_str()),
            ];
            if reason != NO_DOCUMENTATION_CAPTURE {
                fields.push(("documentation_rejected", reason.as_str()));
            }
            log.append_fields("procedure_cache", &fields);
            None
        }
    }
}

/// The `program_source` of a seed template row whose program is retired to
/// the documentation route.
pub const DOCUMENTATION_ROUTE_SOURCE: &str = "documentation_route";

/// One stored catalog program the documentation route reproduces (R1165-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentedCatalogProgram {
    /// The catalog language slug.
    pub language: String,
    /// The catalog task slug.
    pub task: String,
    /// The program the Rust catalog still compiles for the pair.
    pub compiled: String,
    /// What the documentation captures yield for the task's expected output.
    pub rediscovered: Result<RediscoverableRecipe, String>,
}

/// The catalog pairs whose seed program is retired to the documentation route.
///
/// A row of `data/seed/hello-world-programs.lino` whose `program_source` is
/// `documentation_route` stores no program: the solver answers it from the
/// documentation captures. The Rust catalog still compiles a template for the
/// pair (the answer a retired route falls back to), so each such pair is
/// listed with both programs and a test holds them equal.
#[must_use]
pub fn documented_catalog_programs() -> Vec<DocumentedCatalogProgram> {
    let seed = parse_lino(crate::seed::HELLO_WORLD_PROGRAMS_LINO);
    seed.children
        .iter()
        .filter(|node| node.find_child_value("program_source") == DOCUMENTATION_ROUTE_SOURCE)
        .filter_map(|node| {
            let spec = crate::coding::program_spec(
                node.find_child_value("task"),
                node.find_child_value("language"),
            )?;
            let execution = &spec.language.execution;
            let commands: Vec<&str> = execution
                .check_command
                .into_iter()
                .chain(std::iter::once(execution.run_command))
                .collect();
            Some(DocumentedCatalogProgram {
                language: spec.language.slug.to_owned(),
                task: spec.task.slug.to_owned(),
                compiled: spec.template.code.to_owned(),
                rediscovered: rediscover_from_documentation(
                    spec.language.slug,
                    spec.task.slug,
                    &spec.task.output_for_language(spec.language),
                    RunContract {
                        save_as: spec.language.save_as,
                        commands: &commands,
                    },
                ),
            })
        })
        .collect()
}
