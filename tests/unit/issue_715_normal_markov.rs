use formal_ai::normal_markov::{quoted_segments, RewriteHalt, RewriteProgram, RewriteRule};

#[test]
fn rules_use_priority_order_and_replace_the_leftmost_occurrence() {
    let program = RewriteProgram::new(
        vec![
            RewriteRule::new("ab", "X").terminal(),
            RewriteRule::new("a", "Y").terminal(),
        ],
        8,
    );

    let outcome = program.execute("zababa");
    assert_eq!(outcome.output, "zXaba");
    assert_eq!(outcome.halt, RewriteHalt::TerminalRule(0));
    assert_eq!(outcome.trace[0].byte_offset, 1);
}

#[test]
fn nonterminal_rules_restart_from_rule_zero_and_support_state_symbols() {
    let unary_increment = RewriteProgram::new(
        vec![
            RewriteRule::new("[q]1", "1[q]"),
            RewriteRule::new("[q]", "1").terminal(),
        ],
        64,
    );

    let outcome = unary_increment.execute("[q]111");
    assert_eq!(outcome.output, "1111");
    assert_eq!(outcome.halt, RewriteHalt::TerminalRule(1));
    assert_eq!(
        outcome
            .trace
            .iter()
            .map(|step| step.rule_index)
            .collect::<Vec<_>>(),
        [0, 0, 0, 1]
    );
}

#[test]
fn empty_and_nonempty_sequences_are_creation_and_deletion_rules() {
    let create = RewriteProgram::new(vec![RewriteRule::new("", "prefix").terminal()], 4);
    assert_eq!(create.execute("body").output, "prefixbody");

    let delete = RewriteProgram::new(vec![RewriteRule::new("x", "")], 4);
    let outcome = delete.execute("xx");
    assert_eq!(outcome.output, "");
    assert_eq!(outcome.halt, RewriteHalt::NoApplicableRule);
}

#[test]
fn nonterminating_programs_return_a_step_limit_instead_of_hanging() {
    let program = RewriteProgram::new(vec![RewriteRule::new("", "x")], 3);
    let outcome = program.execute("");

    assert_eq!(outcome.output, "xxx");
    assert_eq!(outcome.trace.len(), 3);
    assert_eq!(outcome.halt, RewriteHalt::StepLimit);
}

#[test]
fn literal_slots_preserve_empty_unicode_and_fenced_content_across_prose() {
    let prompt = "Don't phrase-match; map '' to 「内容」, then ```a\\nb``` to ‘’.";
    assert_eq!(quoted_segments(prompt), ["", "内容", "a\\nb", ""]);
}

#[test]
fn single_quoted_slots_preserve_ascii_apostrophes_inside_values() {
    assert_eq!(
        quoted_segments("Replace 'can\'t' with 'can'."),
        ["can't", "can"]
    );
}
