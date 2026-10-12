use super::{negated_command_policy, states_a_command_policy};

#[test]
fn leading_negation_keeps_coordinated_commands_inside_policy() {
    for sentence in [
        "Do not edit the repository, run node report.mjs or download artifacts",
        "Never change files, run git status",
        "Don't alter notes, run node --version",
        "Никогда изменяй файлы, run node report.mjs",
        "Nunca cambies archivos, run node report.mjs",
    ] {
        assert!(negated_command_policy(sentence), "{sentence}");
        assert!(states_a_command_policy(sentence), "{sentence}");
    }
}

#[test]
fn operands_quoted_data_and_conditional_orders_do_not_issue_negation() {
    for sentence in [
        "Run node never.mjs",
        "Run node --version",
        "Run node -e 'console.log(\"do not run cargo\")'",
        "When finished, run node report.mjs",
    ] {
        assert!(!negated_command_policy(sentence), "{sentence}");
    }
    assert!(!states_a_command_policy(
        "When finished, run node report.mjs"
    ));
}
