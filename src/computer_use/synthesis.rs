//! Plan synthesis for unseen computer-use requests (issue #707).
//!
//! [`super::induction`] learns *what* each operation and resource means from the
//! recorded corpus. This module answers the request: given a prompt nobody ever
//! wrote before, in any of the four supported languages, it recognises the
//! resource and the operations the speaker named, materialises the resource with
//! the learned binding, chains the learned operation steps in the order the
//! speaker used, and appends the learned verification.
//!
//! Two rules keep synthesis honest rather than merely plausible:
//!
//! * **Every step is data-flow bound.** Each operation consumes the artifact the
//!   previous step produced and declares the artifact it produces. Path fields
//!   are never copied from the example that taught the operation — copying the
//!   example's `input/customers.csv` into a plan about notes is exactly the
//!   memorisation this module exists to remove. Only non-path fields (the shell
//!   operation name, the confirmation flag) are inherited as learned constants.
//! * **Unsupported requests get no plan.** If the prompt names no known
//!   resource, names no known operation, or names an operation whose schema the
//!   corpus refused to adopt, synthesis returns `None` and the caller answers
//!   with a named `capability_gap` instead of inventing steps.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Map, Value};

use super::induction::{learned, LearnedSchemas, FETCH_OPERATION};
use super::lexicon::{capability_gap_cue, normalize, operation_cues, resource_cue};
use super::seed::{capability_gap_response, step_conditions, CapabilityGap};
use super::{ComputerPlanStep, ComputerUsePlan, ComputerUsePrimitive};

/// Argument fields that name a location in the workspace. Their values are
/// bound per request from the data flow, never inherited from a learned
/// example.
const PATH_FIELDS: [&str; 10] = [
    "path",
    "paths",
    "input",
    "output",
    "source",
    "save_as",
    "archive",
    "destination",
    "from",
    "to",
];

/// What the synthesizer produced for a request, including why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Synthesis {
    pub plan: ComputerUsePlan,
    /// Language-independent slugs of the operations that were realised, in the
    /// order the prompt named them.
    pub operations: Vec<String>,
    /// The resource slug the plan materialised.
    pub resource: String,
}

/// An artifact flowing between steps: a workspace path and whether it is a
/// directory (which decides whether verification reads or lists it).
#[derive(Debug, Clone)]
struct Artifact {
    path: String,
    directory: bool,
}

/// The honest answer for a request we cannot plan.
#[must_use]
pub fn capability_gap_for_request(prompt: &str) -> Option<CapabilityGap> {
    let locale = crate::language::detect(prompt).slug().to_owned();
    let capability = capability_gap_cue(&normalize(prompt))?;
    capability_gap_response(&capability, &locale)
}

/// Synthesize a verified plan for an arbitrary request from the learned
/// schemas, or `None` when the request falls outside what the corpus taught.
#[must_use]
pub fn synthesize(prompt: &str) -> Option<Synthesis> {
    synthesize_with(prompt, learned())
}

/// Synthesize against explicit schemas — used by tests that induce from a
/// synthetic corpus.
#[must_use]
pub fn synthesize_with(prompt: &str, schemas: &LearnedSchemas) -> Option<Synthesis> {
    let normalized = normalize(prompt);
    if capability_gap_cue(&normalized).is_some() {
        return None;
    }
    let locale = crate::language::detect(prompt).slug().to_owned();
    let resource = resource_cue(&normalized)?.slug;
    let binding = schemas.resources.get(&resource)?;

    let mut operations = Vec::new();
    for cue in operation_cues(&normalized) {
        if !operations.contains(&cue.slug) && schemas.operations.contains_key(&cue.slug) {
            operations.push(cue.slug);
        }
    }
    if operations.is_empty() {
        return None;
    }

    let mut steps = binding.steps.clone();
    let mut artifact = last_artifact(&steps)?;
    // A leading `http.fetch` already realises the fetch operation; asking for it
    // again would duplicate a step the materialisation performed.
    let materialised_fetch = steps
        .iter()
        .any(|step| step.primitive == ComputerUsePrimitive::HttpFetch);

    for operation in &operations {
        if operation == FETCH_OPERATION && materialised_fetch {
            continue;
        }
        let schema = schemas.operations.get(operation)?;
        let (step, produced) = realise(
            &schema.step.signature.primitive,
            schema.step.signature.operation.as_deref(),
            &schema.step.constants,
            &schema.step.varying,
            &binding.parameters,
            &resource,
            &artifact,
        )?;
        steps.push(step);
        artifact = produced;
    }

    steps.push(verification_step(&artifact));

    let id = format!("synthesized-{resource}-{}", operations.join("-"));
    let steps = finalize(&id, steps);
    Some(Synthesis {
        plan: ComputerUsePlan {
            id,
            locale,
            prompt: prompt.to_owned(),
            steps,
        },
        operations,
        resource,
    })
}

/// The artifact a materialisation prefix leaves behind.
fn last_artifact(steps: &[ComputerPlanStep]) -> Option<Artifact> {
    let step = steps.last()?;
    let path = step
        .arguments
        .get("save_as")
        .or_else(|| step.arguments.get("path"))?
        .as_str()?
        .to_owned();
    Some(Artifact {
        path,
        directory: false,
    })
}

/// Build one operation step, binding its path fields from the data flow and
/// inheriting the learned non-path constants.
fn realise(
    primitive: &ComputerUsePrimitive,
    operation: Option<&str>,
    constants: &BTreeMap<String, Value>,
    varying: &BTreeSet<String>,
    parameters: &BTreeMap<String, Value>,
    resource: &str,
    input: &Artifact,
) -> Option<(ComputerPlanStep, Artifact)> {
    // Two independent gates keep a field learned for one operation from leaking
    // into another: the primitive's own advertised schema must declare it, and
    // the learned operation schema must actually use it. `archive.pack` fails
    // the first (an archive has no CSV column); `shell.run:count_lines` fails
    // the second (counting lines never filtered by one).
    let accepted = accepted_fields(*primitive);
    let uses = |key: &str| {
        accepted.iter().any(|field| field == key)
            && (constants.contains_key(key) || varying.contains(key))
    };
    let mut arguments = Map::new();
    for (key, value) in constants {
        if !PATH_FIELDS.contains(&key.as_str()) && uses(key) {
            arguments.insert(key.clone(), value.clone());
        }
    }
    // Selector, pointer, column, and comparison value are properties of the
    // resource, learned from the corpus rather than written here.
    for key in ["selector", "pointer", "column", "equals"] {
        if !arguments.contains_key(key) && uses(key) {
            if let Some(value) = parameters.get(key) {
                arguments.insert(key.to_owned(), value.clone());
            }
        }
    }

    let stem = resource.rsplit('_').next().unwrap_or(resource).to_owned();
    let produced = match primitive {
        ComputerUsePrimitive::ShellRun
        | ComputerUsePrimitive::DomQuery
        | ComputerUsePrimitive::DomExtract => {
            let output = format!("reports/{}-{stem}.txt", operation.unwrap_or("extract"));
            let field = if matches!(primitive, ComputerUsePrimitive::ShellRun) {
                ("input", "output")
            } else {
                ("source", "save_as")
            };
            arguments.insert(field.0.to_owned(), json!(input.path));
            arguments.insert(field.1.to_owned(), json!(output.clone()));
            Artifact {
                path: output,
                directory: false,
            }
        }
        ComputerUsePrimitive::FsList => {
            let directory = parent_of(&input.path);
            arguments.insert("path".to_owned(), json!(directory.clone()));
            // Listing observes; the data flow continues from its input.
            arguments.remove("confirmed");
            return Some((
                step(*primitive, Value::Object(arguments)),
                Artifact {
                    path: input.path.clone(),
                    directory: false,
                },
            ));
        }
        ComputerUsePrimitive::ArchivePack => {
            let archive = format!("out/{stem}.fai");
            arguments.insert("paths".to_owned(), json!([input.path]));
            arguments.insert("archive".to_owned(), json!(archive.clone()));
            Artifact {
                path: archive,
                directory: false,
            }
        }
        ComputerUsePrimitive::ArchiveUnpack => {
            arguments.insert("archive".to_owned(), json!(input.path));
            arguments.insert("destination".to_owned(), json!("restored"));
            Artifact {
                path: format!("restored/{}", parent_of(&input.path)),
                directory: true,
            }
        }
        ComputerUsePrimitive::FsMove => {
            let destination = format!("processed/{}", basename_of(&input.path));
            arguments.insert("from".to_owned(), json!(input.path));
            arguments.insert("to".to_owned(), json!(destination.clone()));
            Artifact {
                path: destination,
                directory: false,
            }
        }
        ComputerUsePrimitive::ProcessStatus => {
            let output = format!("reports/process-{stem}.json");
            arguments.insert("save_as".to_owned(), json!(output.clone()));
            Artifact {
                path: output,
                directory: false,
            }
        }
        ComputerUsePrimitive::HttpPost => {
            let output = format!("reports/submission-{stem}.json");
            arguments.insert("save_as".to_owned(), json!(output.clone()));
            Artifact {
                path: output,
                directory: false,
            }
        }
        ComputerUsePrimitive::HttpFetch
        | ComputerUsePrimitive::FsRead
        | ComputerUsePrimitive::FsWrite => return None,
    };
    Some((step(*primitive, Value::Object(arguments)), produced))
}

/// The argument names a primitive advertises in its own MCP input schema.
///
/// Synthesis asks the primitive rather than assuming, so a field learned as a
/// property of a resource (a CSV `column`) can only ever reach a step that
/// declares it.
fn accepted_fields(primitive: ComputerUsePrimitive) -> Vec<String> {
    primitive.input_schema()["properties"]
        .as_object()
        .map(|properties| properties.keys().cloned().collect())
        .unwrap_or_default()
}

/// Every synthesized plan ends by observing what it produced, so the run emits
/// a postcondition event over real bytes rather than trusting the effect.
fn verification_step(artifact: &Artifact) -> ComputerPlanStep {
    let primitive = if artifact.directory {
        ComputerUsePrimitive::FsList
    } else {
        ComputerUsePrimitive::FsRead
    };
    step(primitive, json!({ "path": artifact.path }))
}

fn step(primitive: ComputerUsePrimitive, mut arguments: Value) -> ComputerPlanStep {
    if primitive.changes_state() {
        if let Some(fields) = arguments.as_object_mut() {
            fields.insert("confirmed".to_owned(), json!(true));
        }
    } else if let Some(fields) = arguments.as_object_mut() {
        fields.remove("confirmed");
    }
    let (precondition, postcondition) = step_conditions(primitive);
    ComputerPlanStep {
        id: String::new(),
        primitive,
        arguments,
        precondition,
        postcondition,
    }
}

fn finalize(plan_id: &str, steps: Vec<ComputerPlanStep>) -> Vec<ComputerPlanStep> {
    steps
        .into_iter()
        .enumerate()
        .map(|(index, mut step)| {
            step.id = format!("{plan_id}-{:02}", index + 1);
            step
        })
        .collect()
}

fn parent_of(path: &str) -> String {
    path.rsplit_once('/')
        .map_or_else(|| String::from("."), |(parent, _)| parent.to_owned())
}

fn basename_of(path: &str) -> String {
    path.rsplit_once('/')
        .map_or_else(|| path.to_owned(), |(_, name)| name.to_owned())
}
