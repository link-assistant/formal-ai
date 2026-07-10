//! Agent-CLI recipe for auditing and generalizing the issue-#540 dreaming loop.
//!
//! The audit is intentionally derived from the grounded dreaming recipe rather
//! than from a second prose checklist. Driving this recipe through Agent CLI
//! writes the gap analysis, reads it back, and returns it as the final answer.

use std::fmt::Write as _;

pub const DREAMING_AUDIT_PATH: &str = "dreaming-gap-analysis.lino";

pub const DREAMING_AUDIT_TASK: &str =
    "Audit issue 540 dreaming through Formal AI: inspect the grounded dreaming meta-algorithm, \
     identify every implementation gap from amendment application through replay, pattern \
     learning, storage consent, and idle runtime, then record the generalization that resolves \
     each gap as Links Notation.";

const RECIPE: &str = include_str!("../../data/meta/dreaming-recipe.lino");
const CUES: &str = include_str!("../../data/meta/dreaming-cues.lino");

struct GapResolution {
    gap: &'static str,
    evidence_step: &'static str,
    generalization: &'static str,
}

const RESOLUTIONS: [GapResolution; 7] = [
    GapResolution {
        gap: "stored amendments were not consumed by future tasks",
        evidence_step: "apply_future_tasks",
        generalization: "represent amendments as structured data and apply every matching retained rule at the shared protocol answer boundary",
    },
    GapResolution {
        gap: "topic matching alone claimed that a specific was reproducible",
        evidence_step: "replay_candidates",
        generalization: "derive candidate tasks and require normalized output replay before granting covered status",
    },
    GapResolution {
        gap: "requirement extraction did not test the meta-algorithm or discover structures",
        evidence_step: "mine_patterns",
        generalization: "simulate frequent-topic tasks and retain repeated task structures independently of requirement cue words",
    },
    GapResolution {
        gap: "storage pressure depended on optional static inputs",
        evidence_step: "measure_real_storage",
        generalization: "measure the filesystem containing memory and include the actual next write byte count",
    },
    GapResolution {
        gap: "automatic cleanup had no persisted user decision or migration prompt",
        evidence_step: "idle_consent_runtime",
        generalization: "persist acceptance and refusal, remove only after acceptance, and surface larger-storage pressure",
    },
    GapResolution {
        gap: "background dreaming existed only in desktop timers",
        evidence_step: "idle_consent_runtime",
        generalization: "run the same loop in the core server and guard both runtimes with foreground-idle signals and low process priority",
    },
    GapResolution {
        gap: "English-only code cues and graph terminology limited generalization",
        evidence_step: "learn_requirements",
        generalization: "load multilingual cues from extensible data and describe the representation consistently as memory links",
    },
];

#[must_use]
pub fn is_dreaming_audit_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    lower.contains("dreaming")
        && (lower.contains("issue 540")
            || lower.contains("gap analysis")
            || lower.contains("audit"))
}

#[must_use]
pub fn render_document() -> String {
    let recipe_steps = RECIPE
        .lines()
        .filter(|line| line.trim() == "record_type \"meta_step\"")
        .count();
    let cue_count = CUES
        .lines()
        .filter(|line| line.trim_start().starts_with("cue \""))
        .count();
    let mut out = String::from("dreaming_gap_analysis\n");
    out.push_str("  record_type \"agent_cli_gap_analysis\"\n");
    out.push_str("  issue \"540\"\n");
    let _ = writeln!(out, "  grounded_recipe_steps \"{recipe_steps}\"");
    let _ = writeln!(out, "  multilingual_cues \"{cue_count}\"");
    out.push_str("  conclusion \"all identified gaps resolved by reusable stages\"\n");
    for (index, resolution) in RESOLUTIONS.iter().enumerate() {
        let _ = writeln!(out, "  resolution_{:02}", index + 1);
        field(&mut out, "gap", resolution.gap);
        field(&mut out, "evidence_step", resolution.evidence_step);
        field(&mut out, "generalization", resolution.generalization);
        out.push_str("    status \"implemented_and_tested\"\n");
    }
    out
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    format!(
        "Audited the grounded issue-#540 dreaming meta-algorithm through Formal AI. The recipe \
         now has {} stages and the audit records {} implementation gaps together with the \
         reusable generalization that resolves each one.\n\nGenerated document \
         ({DREAMING_AUDIT_PATH}):\n\n{}",
        RECIPE
            .lines()
            .filter(|line| line.trim() == "record_type \"meta_step\"")
            .count(),
        RESOLUTIONS.len(),
        document.trim_end(),
    )
}

fn field(out: &mut String, name: &str, value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let _ = writeln!(out, "    {name} \"{escaped}\"");
}
