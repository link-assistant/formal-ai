use super::*;
#[test]
fn trailing_acceptance_goals_do_not_rebind_source_targets() {
    for task in [
        "Implement a resolver in output.mjs with meaningful tests.",
        "Write a program in result.py with regressions.",
        "Implement a method in folder/方法-α.mjs with tests.",
    ] {
        let mut result = None;
        assert!(test_expectation_question(task, &mut result).is_none());
        assert!(result.is_none());
    }
}
#[test]
fn owned_test_targets_keep_the_original_missing_expectation_question() {
    for task in [
        "Create a Python test for add in test_m.py and run it.",
        "Create the test in missing.test.mjs.",
        "Write tests in checks.mjs.",
    ] {
        let mut result = None;
        assert!(matches!(
            test_expectation_question(task, &mut result),
            Some(AgenticPlan::Final(_))
        ));
        assert_eq!(result.unwrap().disposition, FinalDisposition::Clarification);
    }
}
