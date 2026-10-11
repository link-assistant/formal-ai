use formal_ai::agentic_coding::general_planner::{GeneralPlanMode, compose_general_change_plan};

#[test]
fn closed_punctuation_operands_preserve_exact_bytes() {
    for payload in ["!?", "…。", "  !  "] {
        let request = format!("Write «{payload}» to punctuation.txt");
        let plan = compose_general_change_plan(&request).expect("closed literal");
        assert_eq!(plan.target, "punctuation.txt");
        assert_eq!(plan.content, payload);
    }
}
#[test]
fn explicit_qualifier_licenses_punctuation() {
    let plan = compose_general_change_plan("Create a file punctuation.txt containing exactly: !!!")
        .expect("explicit literal");
    assert_eq!(plan.target, "punctuation.txt");
    assert_eq!(plan.content, "!!!");
}
#[test]
fn empty_or_unclosed_operands_remain_unlicensed() {
    for request in [
        "Write «» to punctuation.txt",
        "Write «!? to punctuation.txt",
        "Create a file punctuation.txt containing exactly: \"",
    ] {
        assert!(
            !compose_general_change_plan(request)
                .is_some_and(|plan| plan.mode == GeneralPlanMode::LiteralFile)
        );
    }
}
