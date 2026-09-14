use formal_ai::research_learning::{
    AutonomyMode, CycleConfig, KnowledgeKind, RESEARCH_LEARNING_RECIPE, ResearchLearningCycle,
    VerificationGate,
};

fn main() {
    let mut cycle = ResearchLearningCycle::new(
        RESEARCH_LEARNING_RECIPE,
        ["baseline"],
        CycleConfig {
            autonomy: AutonomyMode::FullTrust,
            ..Default::default()
        },
    );
    cycle.begin_unknown("how to calibrate an unfamiliar instrument");
    cycle.record_source(
        "https://example.invalid/calibration-manual",
        "captured calibration procedure",
        true,
    );
    let candidate = cycle.propose_version(
        KnowledgeKind::Procedure,
        "measure reference; adjust offset; verify tolerance",
    );
    cycle.verify_candidate(
        &candidate,
        vec![
            VerificationGate::immutable("baseline", true),
            VerificationGate::immutable("procedure-safety", true),
            VerificationGate::adaptive("new-instrument", true),
        ],
    );

    println!("{}", cycle.links_notation());
}
