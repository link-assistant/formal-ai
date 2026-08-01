//! Inspect issue #705's symbolic next-request plan without network access.
//!
//! Run with: `cargo run --example issue_705_anticipatory_dreaming`

use formal_ai::anticipation::{plan_anticipation, AnticipationConfig, ProbeStatus};
use formal_ai::MemoryEvent;

fn request(id: &str, prompt: &str, intent: &str) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        kind: Some(String::from("message")),
        role: Some(String::from("user")),
        intent: Some(intent.to_owned()),
        content: Some(prompt.to_owned()),
        sent_at: Some(format!("2026-08-01T00:00:{id}Z")),
        write_count: 1,
        ..MemoryEvent::default()
    }
}

fn main() {
    let history = vec![
        request("01", "hello", "greeting"),
        request("02", "2 + 2", "calculation"),
        request("03", "hello again", "greeting"),
        request("04", "reverse the words alpha beta", "text_transformation"),
        request("05", "hello once more", "greeting"),
        request("06", "describe frobulator705 resonance", "unknown"),
        request("07", "hello finally", "greeting"),
    ];
    let plan = plan_anticipation(&history, &AnticipationConfig::default());

    println!("predictions: {}", plan.predictions.len());
    for prediction in &plan.predictions {
        let why = plan
            .why_prediction(&prediction.id)
            .unwrap_or_else(|| String::from("missing transition evidence"));
        println!(
            "{}. {} p={:.3} variants={}\n   why: {}",
            prediction.rank,
            prediction.class.id,
            prediction.probability,
            prediction.variants.len(),
            why,
        );
    }

    let failures = plan
        .probes
        .iter()
        .filter(|probe| probe.status != ProbeStatus::Passed)
        .count();
    println!(
        "offline probes: {}; failures: {}; adoption frontier: {}",
        plan.probes.len(),
        failures,
        plan.frontier.len(),
    );
    println!("\n{}", plan.links_notation());
}
