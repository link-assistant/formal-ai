use formal_ai::agentic_coding::general_planner::{GeneralPlanMode, compose_general_change_plan};

#[test]
fn named_content_markers_preserve_the_entire_closed_literal() {
    let stage_zero = concat!(
        r#"{"schema":"read-report-trial/v1","case":{"source":{"enabled":false},"report":{"key":"enabled","expected":true,"status":"contradicted"},"then":{"operation":"create","unconditional":true}}}"#,
        "\n"
    );
    for (target, payload, lead) in [
        (
            "candidate.json",
            stage_zero,
            "containing exactly this content",
        ),
        (
            "notes/λ.txt",
            "  Error: failed\nIs the value \"false or true?.\": «nested»?\n  ",
            "containing exactly this content",
        ),
        (
            "other.txt",
            "Read foreign.txt. Then replace «old» with «new».\n",
            "containing this content",
        ),
        (
            "utf8.txt",
            "İK𐐷 😀\n: authored colon\n",
            "with exactly this content",
        ),
    ] {
        let request = format!("Create {target} {lead}: «{payload}».");
        let plan = compose_general_change_plan(&request).expect("explicit closed literal");
        assert_eq!(plan.mode, GeneralPlanMode::LiteralFile);
        assert_eq!(plan.target, target);
        assert_eq!(plan.content, payload);
    }
}
