//! Verified, non-visual computer-use plans (issue #707).
//!
//! The planner validates caller-supplied primitive steps. The permission gate,
//! isolated executor and replay verifier share the advertised primitive schemas. Structured HTML/JSON is supported; pixels and rendering are reported
//! honestly as a named `capability_gap`.

mod executor;
mod http;
mod induction;
mod lexicon;
mod planner;
mod seed;
mod synthesis;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use executor::{ComputerUseError, ComputerUsePolicy, ComputerUseSession};
pub use induction::{LearnedSchemas, OperationSchema, ResourceBinding, StepSignature, induce};
pub use lexicon::{
    capability_gap_cue, normalize as normalize_request, operation_cues, resource_cue,
};
pub use planner::{plan_agentic_step, tool_for_primitive};
pub use seed::{BenchmarkTask, CapabilityGap, parse_benchmark_tasks, plan_for_prompt};
pub use synthesis::{Synthesis, synthesize_with};

/// The complete issue-#707 primitive taxonomy, in seed and MCP advertisement
/// order.
pub const COMPUTER_USE_PRIMITIVES: [ComputerUsePrimitive; 12] = [
    ComputerUsePrimitive::FsRead,
    ComputerUsePrimitive::FsWrite,
    ComputerUsePrimitive::FsList,
    ComputerUsePrimitive::FsMove,
    ComputerUsePrimitive::ShellRun,
    ComputerUsePrimitive::HttpFetch,
    ComputerUsePrimitive::HttpPost,
    ComputerUsePrimitive::DomQuery,
    ComputerUsePrimitive::DomExtract,
    ComputerUsePrimitive::ArchivePack,
    ComputerUsePrimitive::ArchiveUnpack,
    ComputerUsePrimitive::ProcessStatus,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ComputerUsePrimitive {
    FsRead,
    FsWrite,
    FsList,
    FsMove,
    ShellRun,
    HttpFetch,
    HttpPost,
    DomQuery,
    DomExtract,
    ArchivePack,
    ArchiveUnpack,
    ProcessStatus,
}

impl ComputerUsePrimitive {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FsRead => "fs.read",
            Self::FsWrite => "fs.write",
            Self::FsList => "fs.list",
            Self::FsMove => "fs.move",
            Self::ShellRun => "shell.run",
            Self::HttpFetch => "http.fetch",
            Self::HttpPost => "http.post",
            Self::DomQuery => "dom.query",
            Self::DomExtract => "dom.extract",
            Self::ArchivePack => "archive.pack",
            Self::ArchiveUnpack => "archive.unpack",
            Self::ProcessStatus => "process.status",
        }
    }

    #[must_use]
    pub fn from_tool_name(name: &str) -> Option<Self> {
        let lower = name.trim().to_ascii_lowercase();
        COMPUTER_USE_PRIMITIVES.into_iter().find(|primitive| {
            let dotted = primitive.name();
            let underscored = dotted.replace('.', "_");
            lower == dotted
                || lower == underscored
                || lower.ends_with(&format!("__{dotted}"))
                || lower.ends_with(&format!("__{underscored}"))
                || lower.ends_with(&format!("_{underscored}"))
        })
    }

    #[must_use]
    pub fn permission_key(self) -> String {
        format!("tool:computer:{}", self.name())
    }

    #[must_use]
    pub const fn changes_state(self) -> bool {
        matches!(
            self,
            Self::FsWrite
                | Self::FsMove
                | Self::ShellRun
                | Self::HttpPost
                | Self::ArchivePack
                | Self::ArchiveUnpack
        )
    }

    #[must_use]
    pub fn input_schema(self) -> Value {
        let properties = match self {
            Self::FsRead | Self::FsList => json!({"path":{"type":"string"}}),
            Self::FsWrite => {
                json!({"path":{"type":"string"},"content":{"type":"string"},"confirmed":{"type":"boolean"}})
            }
            Self::FsMove => {
                json!({"from":{"type":"string"},"to":{"type":"string"},"confirmed":{"type":"boolean"}})
            }
            Self::ShellRun => json!({
                "operation":{"type":"string","enum":["count_lines","filter_csv","unique_csv"]},
                "input":{"type":"string"},
                "output":{"type":"string"},
                "column":{"type":"string"},
                "equals":{"type":"string"},
                "confirmed":{"type":"boolean"}
            }),
            Self::HttpFetch => {
                json!({"url":{"type":"string"},"save_as":{"type":"string"}})
            }
            Self::HttpPost => json!({
                "url":{"type":"string"},
                "body":{"type":"string"},
                "save_as":{"type":"string"},
                "confirmed":{"type":"boolean"}
            }),
            Self::DomQuery => json!({
                "source":{"type":"string"},
                "selector":{"type":"string"},
                "save_as":{"type":"string"}
            }),
            Self::DomExtract => json!({
                "source":{"type":"string"},
                "pointer":{"type":"string"},
                "save_as":{"type":"string"}
            }),
            Self::ArchivePack => json!({
                "paths":{"type":"array","items":{"type":"string"}},
                "archive":{"type":"string"},
                "confirmed":{"type":"boolean"}
            }),
            Self::ArchiveUnpack => json!({
                "archive":{"type":"string"},
                "destination":{"type":"string"},
                "confirmed":{"type":"boolean"}
            }),
            Self::ProcessStatus => json!({"save_as":{"type":"string"}}),
        };
        json!({
            "type":"object",
            "properties": merge_plan_properties(properties),
            "required": required_arguments(self)
                .iter()
                .copied()
                .chain(PLAN_FIELDS)
                .collect::<Vec<_>>(),
            "additionalProperties": false
        })
    }
}

fn merge_plan_properties(mut properties: Value) -> Value {
    let common = [
        ("plan_id", json!({"type":"string"})),
        ("step_id", json!({"type":"string"})),
        ("precondition", json!({"type":"string"})),
        ("postcondition", json!({"type":"string"})),
    ];
    if let Some(map) = properties.as_object_mut() {
        map.extend(
            common
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value)),
        );
    }
    properties
}

const fn required_arguments(primitive: ComputerUsePrimitive) -> &'static [&'static str] {
    match primitive {
        ComputerUsePrimitive::FsRead | ComputerUsePrimitive::FsList => &["path"],
        ComputerUsePrimitive::FsWrite => &["path", "content", "confirmed"],
        ComputerUsePrimitive::FsMove => &["from", "to", "confirmed"],
        ComputerUsePrimitive::ShellRun => &["operation", "input", "output", "confirmed"],
        ComputerUsePrimitive::HttpFetch => &["url", "save_as"],
        ComputerUsePrimitive::HttpPost => &["url", "body", "save_as", "confirmed"],
        ComputerUsePrimitive::DomQuery => &["source", "selector", "save_as"],
        ComputerUsePrimitive::DomExtract => &["source", "pointer", "save_as"],
        ComputerUsePrimitive::ArchivePack => &["paths", "archive", "confirmed"],
        ComputerUsePrimitive::ArchiveUnpack => &["archive", "destination", "confirmed"],
        ComputerUsePrimitive::ProcessStatus => &["save_as"],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputerPlanStep {
    pub id: String,
    pub primitive: ComputerUsePrimitive,
    pub arguments: Value,
    pub precondition: String,
    pub postcondition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputerUsePlan {
    pub id: String,
    pub locale: String,
    pub prompt: String,
    pub steps: Vec<ComputerPlanStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationPhase {
    Precondition,
    Effect,
    Postcondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationEvent {
    pub id: String,
    pub step_id: String,
    pub primitive: String,
    pub phase: VerificationPhase,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputerStepRecord {
    pub plan_id: String,
    pub step_id: String,
    pub primitive: String,
    pub arguments: Value,
    pub output: Value,
    pub events: Vec<VerificationEvent>,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputerUseOutcome {
    pub plan: ComputerUsePlan,
    pub workspace: String,
    pub steps: Vec<ComputerStepRecord>,
    pub verified: bool,
}

const PLAN_FENCE: &str = "```computer-use";
const PLAN_FIELDS: [&str; 4] = ["plan_id", "step_id", "precondition", "postcondition"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestedStep {
    primitive: String,
    arguments: serde_json::Map<String, Value>,
}

/// Collect and validate every step before handing any tool call to the client.
#[must_use]
pub fn plan_request(prompt: &str) -> Option<ComputerUsePlan> {
    let (request, block) = requested_block(prompt)?;
    let mut lines = block.lines().peekable();
    let mut body = String::new();
    while let Some(line) = lines.next_if(|line| line.trim() != "```") {
        if body.len().saturating_add(line.len()) > 256 * 1024 {
            return None;
        }
        body.push_str(line);
        body.push('\n');
    }
    if lines.next().is_none() || lines.any(|line| !line.trim().is_empty()) {
        return None;
    }
    body.pop();
    let entries = serde_json::from_str::<Vec<RequestedStep>>(&body).ok()?;
    if !(1..=64).contains(&entries.len()) {
        return None;
    }
    let digest = crate::source_fetch::sha256_hex(body.as_bytes());
    let id = format!("computer-use-{}", &digest[..24]);
    let steps = entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let primitive = COMPUTER_USE_PRIMITIVES
                .into_iter()
                .find(|item| item.name() == entry.primitive)?;
            validate_plan_arguments(primitive, &entry.arguments)?;
            let (precondition, postcondition) = seed::step_conditions(primitive);
            let number = index + 1;
            Some(ComputerPlanStep {
                id: format!("{id}-{number:02}"),
                primitive,
                arguments: Value::Object(entry.arguments),
                precondition,
                postcondition,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ComputerUsePlan {
        id,
        locale: crate::language::detect(request).slug().to_owned(),
        prompt: prompt.to_owned(),
        steps,
    })
}

fn requested_block(prompt: &str) -> Option<(&str, &str)> {
    let mut offset = 0;
    for line in prompt.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            return (line.trim() == PLAN_FENCE)
                .then(|| (&prompt[..offset], &prompt[offset + line.len()..]));
        }
        offset += line.len();
    }
    None
}

fn validate_plan_arguments(
    primitive: ComputerUsePrimitive,
    arguments: &serde_json::Map<String, Value>,
) -> Option<()> {
    if !required_arguments(primitive)
        .iter()
        .all(|key| arguments.contains_key(*key))
    {
        return None;
    }
    let schema = primitive.input_schema();
    for (key, value) in arguments {
        if PLAN_FIELDS.contains(&key.as_str()) {
            return None;
        }
        let property = schema["properties"].get(key)?;
        let valid = match property["type"].as_str()? {
            "string" => {
                value.is_string()
                    && property["enum"]
                        .as_array()
                        .is_none_or(|options| options.contains(value))
            }
            "boolean" => value.is_boolean(),
            "array" => value
                .as_array()
                .is_some_and(|entries| !entries.is_empty() && entries.iter().all(Value::is_string)),
            _ => false,
        };
        if !valid {
            return None;
        }
    }
    if primitive == ComputerUsePrimitive::ShellRun {
        let operation = arguments.get("operation")?.as_str()?;
        if (matches!(operation, "filter_csv" | "unique_csv") && !arguments.contains_key("column"))
            || (operation == "filter_csv" && !arguments.contains_key("equals"))
        {
            return None;
        }
    }
    Some(())
}

/// A malformed explicit plan or a request requiring an unavailable capability.
#[must_use]
pub fn capability_gap_for_request(prompt: &str) -> Option<CapabilityGap> {
    if let Some((request, _)) = requested_block(prompt) {
        return if plan_request(prompt).is_none() {
            seed::capability_gap_response("plan-input", crate::language::detect(request).slug())
        } else {
            None
        };
    }
    synthesis::capability_gap_for_request(prompt)
}

/// Execute a computer-use request with all issue-#707 permissions granted.
///
/// This is the native orchestration surface. Protocol and MCP clients use the
/// same plan and [`ComputerUseSession::execute_step`] one primitive at a time.
pub fn run_verified_plan(prompt: &str) -> Result<ComputerUseOutcome, ComputerUseError> {
    let plan =
        plan_request(prompt).ok_or_else(|| ComputerUseError::UnknownPlan(prompt.to_owned()))?;
    execute_verified_plan(plan, ComputerUsePolicy::agent_mode_all())
}

/// Apply a plan derived from an explicit request or caller-supplied schemas.
pub fn execute_verified_plan(
    plan: ComputerUsePlan,
    policy: ComputerUsePolicy,
) -> Result<ComputerUseOutcome, ComputerUseError> {
    let mut session = ComputerUseSession::new(&plan.id, policy)?;
    let mut records = Vec::with_capacity(plan.steps.len());
    for step in &plan.steps {
        let record = session.execute_step(step);
        let verified = record.verified;
        records.push(record);
        if !verified {
            break;
        }
    }
    let verified = records.iter().all(|record| record.verified);
    Ok(ComputerUseOutcome {
        plan,
        workspace: session.root().display().to_string(),
        steps: records,
        verified,
    })
}

/// Replay a recorded outcome and verify its stable plan, primitive order, and
/// per-step verification events against a fresh isolated workspace.
pub fn replay_verified_plan(recorded: &ComputerUseOutcome) -> Result<bool, ComputerUseError> {
    replay_verified_plan_with_policy(recorded, ComputerUsePolicy::agent_mode_all())
}

/// Replay with the same operator-supplied policy used for the original plan.
pub fn replay_verified_plan_with_policy(
    recorded: &ComputerUseOutcome,
    policy: ComputerUsePolicy,
) -> Result<bool, ComputerUseError> {
    let replay = execute_verified_plan(recorded.plan.clone(), policy)?;
    Ok(replay.verified
        && replay.plan == recorded.plan
        && replay
            .steps
            .iter()
            .map(|step| (&step.primitive, &step.output, &step.events))
            .eq(recorded
                .steps
                .iter()
                .map(|step| (&step.primitive, &step.output, &step.events))))
}

#[must_use]
pub fn mcp_tool_definitions() -> Vec<Value> {
    COMPUTER_USE_PRIMITIVES
        .into_iter()
        .map(|primitive| {
            json!({
                "name": primitive.name(),
                "description": seed::tool_description(primitive),
                "inputSchema": primitive.input_schema()
            })
        })
        .collect()
}
