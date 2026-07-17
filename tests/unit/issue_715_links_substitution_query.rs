//! Issue #715: the meta-language IR between natural language and harness tools
//! must be link-cli's substitution query language, not a bespoke rendering.
//!
//! `link-cli` expresses every CRUD operation as one substitution
//! `(matching pattern) (substitution pattern)`, which it documents as "Markov
//! algorithm ... which is turing complete". These tests pin that dialect over
//! text sequences and its lowering to [`RewriteProgram`].

use formal_ai::links_substitution_query::{
    parse_substitution_query, render_substitution_query, substitution_effect,
    SubstitutionQueryError,
};
use formal_ai::normal_markov::{RewriteHalt, RewriteProgram, RewriteRule};
use formal_ai::substitution::CrudEvent;

const STEPS: usize = 64;

fn parse(query: &str) -> RewriteProgram {
    parse_substitution_query(query, STEPS)
        .unwrap_or_else(|error| panic!("{query} should parse: {error}"))
}

/// link-cli: `() ((1 1))` creates. Creation is the empty sequence substituted
/// to a non-empty one.
#[test]
fn empty_matching_side_is_creation() {
    let program = parse(r#"() ((terminal: "hello"))"#);

    assert_eq!(
        program.rules,
        vec![RewriteRule::new("", "hello").terminal()]
    );
    assert_eq!(program.max_steps, STEPS);
    assert_eq!(substitution_effect(&program.rules[0]), CrudEvent::Create);
    assert_eq!(program.execute("").output, "hello");
}

/// An empty pattern always matches at offset zero, so a *non-terminal* creation
/// is a genuinely divergent normal algorithm rather than a parser bug. The
/// dialect stays faithful and lets the executor's bound catch it; callers that
/// mean "insert once" mark the rule terminal, as `code_artifact` does.
#[test]
fn nonterminal_creation_diverges_until_the_step_bound() {
    let program = parse(r#"() (("ha"))"#);

    assert_eq!(program.rules, vec![RewriteRule::new("", "ha")]);
    let outcome = program.execute("");
    assert_eq!(outcome.halt, RewriteHalt::StepLimit);
    assert_eq!(outcome.output, "ha".repeat(STEPS));
}

/// link-cli: `((1 1)) ()` deletes. Deletion is the reverse of creation.
#[test]
fn empty_substitution_side_is_deletion() {
    let program = parse(r#"(("hello ")) ()"#);

    assert_eq!(program.rules, vec![RewriteRule::new("hello ", "")]);
    assert_eq!(substitution_effect(&program.rules[0]), CrudEvent::Delete);
    assert_eq!(program.execute("hello world").output, "world");
}

/// link-cli: `((1: 1 1)) ((1: 1 2))` updates.
#[test]
fn differing_operands_are_an_update() {
    let program = parse(r#"(("old")) (("new"))"#);

    assert_eq!(program.rules, vec![RewriteRule::new("old", "new")]);
    assert_eq!(substitution_effect(&program.rules[0]), CrudEvent::Update);
    assert_eq!(program.execute("an old thing").output, "an new thing");
}

/// link-cli: `((1: 1 1)) ((1: 1 1))` reads. An identity substitution is a
/// no-op that still names what it selected.
#[test]
fn identical_operands_are_a_read() {
    let program = parse(r#"(("x")) (("x"))"#);

    assert_eq!(substitution_effect(&program.rules[0]), CrudEvent::Read);
    assert_eq!(program.execute("x").output, "x");
}

/// link-cli pairs multiple links positionally: `() ((1 1) (2 2))`.
#[test]
fn operands_pair_positionally_into_ordered_rules() {
    let program = parse(r#"(("a") ("b")) (("A") ("B"))"#);

    assert_eq!(
        program.rules,
        vec![RewriteRule::new("a", "A"), RewriteRule::new("b", "B")]
    );
    // Rule order is Markov priority order: `a` is tried before `b` on restart.
    assert_eq!(program.execute("ba").output, "BA");
}

/// A side with several operands and an empty counterpart distributes.
#[test]
fn elided_side_distributes_across_every_operand() {
    let creations = parse(r#"() (("a") ("b"))"#);
    assert_eq!(
        creations.rules,
        vec![RewriteRule::new("", "a"), RewriteRule::new("", "b")]
    );

    let deletions = parse(r#"(("a") ("b")) ()"#);
    assert_eq!(
        deletions.rules,
        vec![RewriteRule::new("a", ""), RewriteRule::new("b", "")]
    );
}

/// Markov terminal rules have no link-cli equivalent, so the dialect carries
/// them in link-cli's own named-reference slot (`(child: father mother)`).
#[test]
fn terminal_rules_use_the_named_reference_slot() {
    let program = parse(r#"(("old")) ((terminal: "new"))"#);
    assert_eq!(
        program.rules,
        vec![RewriteRule::new("old", "new").terminal()]
    );

    // A deletion elides the substitution side, so the name rides the match.
    let deletion = parse(r#"((terminal: "gone")) ()"#);
    assert_eq!(
        deletion.rules,
        vec![RewriteRule::new("gone", "").terminal()]
    );
}

#[test]
fn quoted_operands_carry_escapes_and_empty_text() {
    let program = parse(r#"(("say \"hi\"")) ((""))"#);

    assert_eq!(program.rules, vec![RewriteRule::new("say \"hi\"", "")]);
    assert_eq!(substitution_effect(&program.rules[0]), CrudEvent::Delete);

    let backslash = parse(r#"(("a\\b")) (("c"))"#);
    assert_eq!(backslash.rules, vec![RewriteRule::new("a\\b", "c")]);
}

#[test]
fn rendering_round_trips_through_the_parser() {
    for rules in [
        vec![RewriteRule::new("", "created")],
        vec![RewriteRule::new("deleted", "")],
        vec![RewriteRule::new("old", "new").terminal()],
        vec![RewriteRule::new("read", "read")],
        vec![RewriteRule::new("gone", "").terminal()],
        vec![RewriteRule::new("", "made").terminal()],
        vec![
            RewriteRule::new("a", "A"),
            RewriteRule::new("b", "B").terminal(),
        ],
        // Degenerate: both sides empty must not collapse into zero rules.
        vec![RewriteRule::new("", "")],
        vec![],
    ] {
        let program = RewriteProgram::new(rules, STEPS);
        let query = render_substitution_query(&program);
        assert_eq!(
            parse_substitution_query(&query, STEPS).as_ref(),
            Ok(&program),
            "{query} should round-trip",
        );
    }
}

#[test]
fn canonical_renderings_match_the_link_cli_shorthands() {
    let render = |rules| render_substitution_query(&RewriteProgram::new(rules, STEPS));

    assert_eq!(render(vec![RewriteRule::new("", "a")]), r#"() (("a"))"#);
    assert_eq!(render(vec![RewriteRule::new("a", "")]), r#"(("a")) ()"#);
    assert_eq!(
        render(vec![RewriteRule::new("a", "b").terminal()]),
        r#"(("a")) ((terminal: "b"))"#
    );
    assert_eq!(render(vec![]), "() ()");
    // Only a genuinely ruleless program may render as `() ()`. An identity on
    // the empty sequence keeps both sides so it cannot parse back as no rules.
    assert_eq!(render(vec![RewriteRule::new("", "")]), r#"(("")) ((""))"#);
}

#[test]
fn malformed_queries_are_rejected_with_a_reason() {
    for (query, expected) in [
        (r#"(("a") ("b")) (("A"))"#, "operand count"),
        (r#"(("a"))"#, "two sides"),
        (r#"(("a")) (("b")) (("c"))"#, "two sides"),
        (r#"(("a")) (("b")"#, "unbalanced"),
        (r#"(("a")) (("b"#, "unbalanced"),
        (r#"((oops: "a")) (("b"))"#, "terminal"),
        (r#"((a)) (("b"))"#, "quoted"),
    ] {
        let error = parse_substitution_query(query, STEPS)
            .expect_err(&format!("{query} should be rejected"));
        let message = error.to_string();
        assert!(
            message.contains(expected),
            "{query} should report {expected:?}, reported {message:?}",
        );
    }
}

#[test]
fn whitespace_between_operands_is_insignificant() {
    let spaced = parse("(  (\"a\")   (\"b\")  )\n  (  (\"A\") (\"B\") )");

    assert_eq!(
        spaced.rules,
        vec![RewriteRule::new("a", "A"), RewriteRule::new("b", "B")]
    );
}

#[test]
fn error_type_is_a_std_error() {
    fn assert_error<E: std::error::Error>() {}
    assert_error::<SubstitutionQueryError>();
}
