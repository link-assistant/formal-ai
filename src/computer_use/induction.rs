//! Auto-learning: induce generalized computer-use schemas from the recorded
//! benchmark corpus (issue #707).
//!
//! The first milestone of issue #707 could only execute a prompt it had already
//! been given: [`super::seed::plan_for_prompt`] is an exact-phrase lookup over
//! ten recorded tasks. That is memorisation, not capability. This module closes
//! the loop the issue actually asks for — *"the universal solver decomposes a
//! computer-use request into a verified plan of primitive calls"* — by learning
//! the request → plan mapping from the corpus instead of hardcoding it:
//!
//! 1. **Align.** For each recorded task, split its plan into the resource
//!    *materialisation* prefix, the *body* (the steps that realise operations),
//!    and the *verification* suffix; recognise which operation and resource
//!    meanings its prompts evidence, in every language.
//! 2. **Associate.** Count, over the whole corpus, which primitive signature
//!    each operation co-occurs with, which materialisation each resource
//!    co-occurs with, and which verification steps follow each final operation.
//! 3. **Gate.** Adopt an operation schema only when the signature is present in
//!    *every* task whose prompt names that operation (necessity), and adopt a
//!    resource binding only when the corpus never contradicts it. Argument
//!    fields that are unanimous across supporting examples become constants;
//!    fields that differ become resource-parameterised slots.
//! 4. **Record.** Everything adopted, rejected, or left unexplained is written
//!    to Links Notation, so the learned knowledge is reviewable data rather
//!    than opaque state.
//!
//! Learning is deterministic and offline: the same committed corpus always
//! induces the same schemas, which is what makes the synthesized plans
//! replayable in CI.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::OnceLock;

use serde_json::Value;

use super::lexicon::{normalize, operation_cues, resource_cue};
use super::seed::{BenchmarkTask, benchmark_tasks};
use super::{ComputerPlanStep, ComputerUsePrimitive};

/// Head of the Links Notation record [`LearnedSchemas::links_notation`] emits.
const RECORD_TYPE: &str = "computer_use_learned_schemas";

/// The corpus the schemas are induced from, cited in that record so a reader can
/// re-derive them from the same evidence.
const CORPUS_PATH: &str = "data/seed/computer-use-tasks.lino";

/// The primitive-level identity of a step, ignoring its concrete paths: the
/// primitive plus, for `shell.run`, its structured operation. Two steps with
/// the same signature do the same *kind* of work.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StepSignature {
    pub primitive: ComputerUsePrimitive,
    pub operation: Option<String>,
}

impl StepSignature {
    #[must_use]
    pub fn of(step: &ComputerPlanStep) -> Self {
        Self {
            primitive: step.primitive,
            operation: step
                .arguments
                .get("operation")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        }
    }

    #[must_use]
    pub fn label(&self) -> String {
        self.operation.as_ref().map_or_else(
            || self.primitive.name().to_owned(),
            |operation| format!("{}:{operation}", self.primitive.name()),
        )
    }
}

/// A learned step template: the signature to emit plus the argument fields that
/// were unanimous across every supporting example. Fields that varied are
/// omitted here and bound per request by the synthesizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepSchema {
    pub signature: StepSignature,
    pub constants: BTreeMap<String, Value>,
    pub varying: BTreeSet<String>,
    pub support: Vec<String>,
}

/// A learned operation schema: which step an operation slug denotes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationSchema {
    pub operation: String,
    pub step: StepSchema,
    /// Verification steps observed after this operation when it ended a plan.
    pub verification: Vec<StepSchema>,
}

/// A learned resource binding: how to materialise a named resource before the
/// operations run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBinding {
    pub resource: String,
    pub steps: Vec<ComputerPlanStep>,
    /// Per-resource values for argument fields that vary across operations
    /// (for example the DOM selector used with this resource).
    pub parameters: BTreeMap<String, Value>,
    pub support: Vec<String>,
    pub alternatives: Vec<String>,
}

/// Everything the corpus taught us, plus the honest residue.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LearnedSchemas {
    pub operations: BTreeMap<String, OperationSchema>,
    pub resources: BTreeMap<String, ResourceBinding>,
    /// Operations recognised in a prompt whose signature was not present in
    /// every supporting task — rejected rather than guessed.
    pub rejected: Vec<String>,
    /// Recorded steps no prompt cue accounts for, named per task.
    pub unexplained: Vec<String>,
}

/// The schemas induced from the committed benchmark corpus. Cached: the corpus
/// is immutable at runtime, so induction runs at most once per process.
#[must_use]
pub fn learned() -> &'static LearnedSchemas {
    static CACHE: OnceLock<LearnedSchemas> = OnceLock::new();
    CACHE.get_or_init(|| induce(benchmark_tasks()))
}

/// Split a recorded plan into materialisation, body, and verification.
#[must_use]
pub fn partition(steps: &[ComputerPlanStep]) -> (usize, usize) {
    let materialisation = steps
        .iter()
        .take_while(|step| {
            matches!(
                step.primitive,
                ComputerUsePrimitive::FsWrite | ComputerUsePrimitive::HttpFetch
            )
        })
        .count();
    let verification = steps[materialisation..]
        .iter()
        .rev()
        .take_while(|step| {
            matches!(
                step.primitive,
                ComputerUsePrimitive::FsRead | ComputerUsePrimitive::FsList
            )
        })
        .count();
    (materialisation, steps.len() - verification)
}

/// Induce schemas from `tasks`. Public so tests can induce from a synthetic
/// corpus and assert the gates reject contradictory evidence.
#[must_use]
pub fn induce(tasks: &[BenchmarkTask]) -> LearnedSchemas {
    let mut observations: BTreeMap<String, Vec<(String, ComputerPlanStep)>> = BTreeMap::new();
    let mut verifications: BTreeMap<String, Vec<Vec<ComputerPlanStep>>> = BTreeMap::new();
    let mut resource_steps: BTreeMap<String, Vec<(String, Vec<ComputerPlanStep>)>> =
        BTreeMap::new();
    let mut operations_per_task: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut schemas = LearnedSchemas::default();

    for task in tasks {
        let operations = recognized_operations(task);
        let resource = recognized_resource(task);
        operations_per_task.insert(task.id.clone(), operations.clone());
        let (materialised, body_end) = partition(&task.steps);
        let body = &task.steps[materialised..body_end];
        let verification = &task.steps[body_end..];

        if let Some(resource) = resource.clone() {
            let mut steps = task.steps[..materialised].to_vec();
            // A fetch is both the materialisation and the fetch operation; a
            // task with no leading write or fetch materialises nothing.
            steps.retain(|step| !step.arguments.is_null());
            resource_steps
                .entry(resource)
                .or_default()
                .push((task.id.clone(), steps));
        }

        // Attribute body steps to the operations named by the prompt, in order.
        let body_operations = operations
            .iter()
            .filter(|slug| slug.as_str() != FETCH_OPERATION || materialised == 0)
            .cloned()
            .collect::<Vec<_>>();
        let mut attributed = vec![false; body.len()];
        for (index, operation) in body_operations.iter().enumerate() {
            if let Some(step) = body.get(index) {
                observations
                    .entry(operation.clone())
                    .or_default()
                    .push((task.id.clone(), step.clone()));
                attributed[index] = true;
            }
        }
        if let (Some(last), false) = (body_operations.last(), verification.is_empty())
            && body_operations.len() >= body.len()
        {
            verifications
                .entry(last.clone())
                .or_default()
                .push(verification.to_vec());
        }
        // The fetch operation is realised by the materialisation prefix.
        if operations.iter().any(|slug| slug == FETCH_OPERATION)
            && materialised > 0
            && let Some(step) = task.steps[..materialised]
                .iter()
                .find(|step| step.primitive == ComputerUsePrimitive::HttpFetch)
        {
            observations
                .entry(FETCH_OPERATION.to_owned())
                .or_default()
                .push((task.id.clone(), step.clone()));
        }
        for (index, step) in body.iter().enumerate() {
            if !attributed[index] {
                schemas
                    .unexplained
                    .push(format!("{}:{}", task.id, step.primitive.name()));
            }
        }
    }

    for (operation, examples) in observations {
        let signatures = examples
            .iter()
            .map(|(_, step)| StepSignature::of(step))
            .collect::<BTreeSet<_>>();
        let Some(signature) = single(&signatures) else {
            schemas
                .rejected
                .push(format!("{operation}:ambiguous_signature"));
            continue;
        };
        // Necessity gate: the signature must be present in every task whose
        // prompt names this operation.
        let named_in = operations_per_task
            .iter()
            .filter(|(_, operations)| operations.contains(&operation))
            .map(|(task, _)| task.clone())
            .collect::<BTreeSet<_>>();
        let observed_in = examples
            .iter()
            .map(|(task, _)| task.clone())
            .collect::<BTreeSet<_>>();
        if !named_in.is_subset(&observed_in) {
            schemas
                .rejected
                .push(format!("{operation}:missing_in_supporting_task"));
            continue;
        }
        let step = step_schema(&signature, &examples);
        let verification = verifications
            .get(&operation)
            .map(|observed| verification_schema(observed))
            .unwrap_or_default();
        schemas.operations.insert(
            operation.clone(),
            OperationSchema {
                operation,
                step,
                verification,
            },
        );
    }

    for (resource, observed) in resource_steps {
        let Some((task, steps)) = observed.first().cloned() else {
            continue;
        };
        let alternatives = observed
            .iter()
            .skip(1)
            .map(|(task, _)| task.clone())
            .collect::<Vec<_>>();
        let parameters = resource_parameters(&resource, tasks);
        schemas.resources.insert(
            resource.clone(),
            ResourceBinding {
                resource,
                steps,
                parameters,
                support: vec![task],
                alternatives,
            },
        );
    }
    schemas.rejected.sort();
    schemas.rejected.dedup();
    schemas.unexplained.sort();
    schemas.unexplained.dedup();
    schemas
}

/// The operation slug for fetching a remote resource — the one operation that
/// is realised by a task's materialisation prefix rather than by its body.
pub const FETCH_OPERATION: &str = "computer_use_fetch";

fn recognized_operations(task: &BenchmarkTask) -> Vec<String> {
    // An operation counts as named by the task when every localized prompt
    // evidences it: a cue only one translation happens to contain is noise.
    let mut per_locale = task.prompts.values().map(|prompt| {
        operation_cues(&normalize(prompt))
            .into_iter()
            .map(|cue| cue.slug)
            .collect::<Vec<_>>()
    });
    let Some(first) = per_locale.next() else {
        return Vec::new();
    };
    let shared = per_locale.fold(
        first.iter().cloned().collect::<BTreeSet<_>>(),
        |shared, locale| {
            shared
                .intersection(&locale.into_iter().collect())
                .cloned()
                .collect()
        },
    );
    first
        .into_iter()
        .filter(|slug| shared.contains(slug))
        .collect()
}

fn recognized_resource(task: &BenchmarkTask) -> Option<String> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for prompt in task.prompts.values() {
        if let Some(cue) = resource_cue(&normalize(prompt)) {
            *counts.entry(cue.slug).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(slug, _)| slug)
}

fn single(signatures: &BTreeSet<StepSignature>) -> Option<StepSignature> {
    (signatures.len() == 1).then(|| signatures.iter().next().cloned())?
}

fn step_schema(signature: &StepSignature, examples: &[(String, ComputerPlanStep)]) -> StepSchema {
    let mut constants: BTreeMap<String, Value> = BTreeMap::new();
    let mut varying = BTreeSet::new();
    for (_, step) in examples {
        let Some(fields) = step.arguments.as_object() else {
            continue;
        };
        for (key, value) in fields {
            match constants.get(key) {
                None if !varying.contains(key) => {
                    constants.insert(key.clone(), value.clone());
                }
                Some(existing) if existing != value => {
                    constants.remove(key);
                    varying.insert(key.clone());
                }
                _ => {}
            }
        }
    }
    StepSchema {
        signature: signature.clone(),
        constants,
        varying,
        support: examples.iter().map(|(task, _)| task.clone()).collect(),
    }
}

fn verification_schema(observed: &[Vec<ComputerPlanStep>]) -> Vec<StepSchema> {
    let Some(first) = observed.first() else {
        return Vec::new();
    };
    if observed
        .iter()
        .any(|steps| steps.len() != first.len() || !same_shape(steps, first))
    {
        return Vec::new();
    }
    first
        .iter()
        .map(|step| StepSchema {
            signature: StepSignature::of(step),
            constants: BTreeMap::new(),
            varying: BTreeSet::new(),
            support: Vec::new(),
        })
        .collect()
}

fn same_shape(left: &[ComputerPlanStep], right: &[ComputerPlanStep]) -> bool {
    left.iter()
        .zip(right)
        .all(|(left, right)| StepSignature::of(left) == StepSignature::of(right))
}

/// Field values that a resource fixes for the operations applied to it — the
/// DOM selector or JSON pointer a task used with this resource, and the CSV
/// column and value a filter used. Learned from the corpus, never hardcoded.
fn resource_parameters(resource: &str, tasks: &[BenchmarkTask]) -> BTreeMap<String, Value> {
    let mut parameters: BTreeMap<String, Value> = BTreeMap::new();
    for task in tasks {
        if recognized_resource(task).as_deref() != Some(resource) {
            continue;
        }
        for step in &task.steps {
            let Some(fields) = step.arguments.as_object() else {
                continue;
            };
            for key in ["selector", "pointer", "column", "equals", "body"] {
                if let Some(value) = fields.get(key) {
                    parameters
                        .entry(key.to_owned())
                        .or_insert_with(|| value.clone());
                }
            }
        }
    }
    parameters
}

impl LearnedSchemas {
    /// A reviewable Links Notation record of everything that was learned,
    /// rejected, and left unexplained.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "{RECORD_TYPE}");
        let _ = writeln!(out, "  source {CORPUS_PATH}");
        for schema in self.operations.values() {
            let _ = writeln!(out, "  operation {}", schema.operation);
            let _ = writeln!(out, "    step {}", schema.step.signature.label());
            for (key, value) in &schema.step.constants {
                let _ = writeln!(out, "    constant {key} {value}");
            }
            for key in &schema.step.varying {
                let _ = writeln!(out, "    slot {key}");
            }
            for task in &schema.step.support {
                let _ = writeln!(out, "    support {task}");
            }
            for step in &schema.verification {
                let _ = writeln!(out, "    verification {}", step.signature.label());
            }
        }
        for binding in self.resources.values() {
            let _ = writeln!(out, "  resource {}", binding.resource);
            for step in &binding.steps {
                let _ = writeln!(
                    out,
                    "    materialize {} {}",
                    step.primitive.name(),
                    step.arguments
                );
            }
            for (key, value) in &binding.parameters {
                let _ = writeln!(out, "    parameter {key} {value}");
            }
            for task in &binding.support {
                let _ = writeln!(out, "    support {task}");
            }
            for task in &binding.alternatives {
                let _ = writeln!(out, "    alternative {task}");
            }
        }
        for rejected in &self.rejected {
            let _ = writeln!(out, "  rejected {rejected}");
        }
        for unexplained in &self.unexplained {
            let _ = writeln!(out, "  unexplained {unexplained}");
        }
        out
    }
}
