//! Issue #715: the substitution query language must cover links, not only text.
//!
//! Review feedback on #727 asked for link-cli's substitution patterns "for text
//! sequences and links in general". `issue_715_links_substitution_query` pins
//! the text domain; this pins the link one, using link-cli's own documented
//! queries as the fixtures. The control model is shared with
//! [`formal_ai::normal_markov`] — ordered rules, first applicable wins,
//! selection restarts at rule zero — so only the operand domain is new.

use formal_ai::link_store::DoubletLink;
use formal_ai::links_substitution_query::{
    link_substitution_effect, parse_link_substitution_query, render_link_substitution_query,
    LinkPattern, LinkRewriteProgram, LinkRewriteRule, Slot, SubstitutionQueryError,
};
use formal_ai::normal_markov::RewriteHalt;
use formal_ai::substitution::CrudEvent;

const STEPS: usize = 64;

fn parse(query: &str) -> LinkRewriteProgram {
    parse_link_substitution_query(query, STEPS)
        .unwrap_or_else(|error| panic!("{query} should parse: {error}"))
}

fn parse_error(query: &str) -> SubstitutionQueryError {
    match parse_link_substitution_query(query, STEPS) {
        Err(error) => error,
        Ok(program) => panic!("{query} should not parse, but produced {program:?}"),
    }
}

fn link(index: &str, from: &str, to: &str) -> DoubletLink {
    DoubletLink {
        index: index.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

fn value(text: &str) -> Slot {
    Slot::Value(text.to_owned())
}

fn variable(name: &str) -> Slot {
    Slot::Variable(name.to_owned())
}

/// link-cli: `clink '() ((1 1))'` creates. Creation is the empty link
/// substituted to a present one — issue #715's "absence to non-empty sequence",
/// in the link domain.
#[test]
fn empty_matching_side_creates_a_link() {
    let program = parse("() ((1 1))");

    assert_eq!(program.rules.len(), 1);
    assert_eq!(
        link_substitution_effect(&program.rules[0]),
        CrudEvent::Create
    );

    let outcome = program.execute(&[]);

    assert_eq!(outcome.links, vec![link("1", "1", "1")]);
    assert_eq!(outcome.halt, RewriteHalt::NoApplicableRule);
    assert_eq!(outcome.trace.len(), 1, "trace: {:?}", outcome.trace);
    assert_eq!(outcome.trace[0].effect, CrudEvent::Create);
    assert_eq!(outcome.trace[0].link, link("1", "1", "1"));
}

/// link-cli: `clink '((1 1)) ()'` deletes. Deletion is creation reversed.
#[test]
fn empty_substitution_side_deletes_a_link() {
    let program = parse("((1 1)) ()");

    assert_eq!(
        link_substitution_effect(&program.rules[0]),
        CrudEvent::Delete
    );

    let outcome = program.execute(&[link("1", "1", "1"), link("2", "3", "4")]);

    assert_eq!(outcome.links, vec![link("2", "3", "4")]);
    assert_eq!(outcome.trace[0].effect, CrudEvent::Delete);
}

/// link-cli: `clink '((1: 1 1)) ((1: 1 2))'` updates.
#[test]
fn paired_sides_update_a_link() {
    let program = parse("((1: 1 1)) ((1: 1 2))");

    assert_eq!(
        link_substitution_effect(&program.rules[0]),
        CrudEvent::Update
    );

    let outcome = program.execute(&[link("1", "1", "1")]);

    assert_eq!(outcome.links, vec![link("1", "1", "2")]);
    assert_eq!(outcome.trace.len(), 1, "trace: {:?}", outcome.trace);
}

/// The `(index: source target)` shape is link-cli's, so it must parse into
/// exactly those three slots rather than into a flattened operand.
#[test]
fn link_cli_index_syntax_parses_into_the_documented_slots() {
    assert_eq!(
        parse("((1: 1 1)) ((1: 1 2))").rules,
        vec![LinkRewriteRule {
            pattern: Some(LinkPattern {
                index: Some(value("1")),
                source: value("1"),
                target: value("1"),
            }),
            replacement: Some(LinkPattern {
                index: Some(value("1")),
                source: value("1"),
                target: value("2"),
            }),
        }]
    );
}

/// link-cli: "`$i` stands for variable named i, that stands for index. `$s` is
/// for source and `$t` is for target."
#[test]
fn dollar_prefixed_slots_parse_as_variables() {
    assert_eq!(
        parse("(($i: $s $t)) (($i: $t $s))").rules,
        vec![LinkRewriteRule {
            pattern: Some(LinkPattern {
                index: Some(variable("i")),
                source: variable("s"),
                target: variable("t"),
            }),
            replacement: Some(LinkPattern {
                index: Some(variable("i")),
                source: variable("t"),
                target: variable("s"),
            }),
        }]
    );
}

/// link-cli documents `(($i: $s $t)) (($i: $s $t))` as reading "all links
/// without modification". A read must therefore terminate and leave the store
/// alone: over a set-valued store, a substitution that changes nothing is not a
/// state transition, so it is never selected.
#[test]
fn identical_sides_read_every_link_without_modifying_the_store() {
    let program = parse("(($i: $s $t)) (($i: $s $t))");

    assert_eq!(link_substitution_effect(&program.rules[0]), CrudEvent::Read);

    let store = vec![link("1", "1", "1"), link("2", "1", "2")];
    let outcome = program.execute(&store);

    assert_eq!(outcome.links, store);
    assert!(outcome.trace.is_empty(), "trace: {:?}", outcome.trace);
    assert_eq!(outcome.halt, RewriteHalt::NoApplicableRule);
    assert_eq!(
        program.matched_links(&store),
        store,
        "a read is answered by matching, not by rewriting"
    );
}

/// A variable used twice constrains the match instead of rebinding, so
/// `($i: $s $s)` selects exactly the links whose source and target agree.
#[test]
fn a_repeated_variable_matches_only_where_the_slots_agree() {
    let program = parse("(($i: $s $s)) ()");

    let outcome = program.execute(&[
        link("1", "1", "1"),
        link("2", "1", "2"),
        link("3", "7", "7"),
    ]);

    assert_eq!(outcome.links, vec![link("2", "1", "2")]);
}

/// An elided index on a substitution keeps the matched link's own index, which
/// is what lets a rule rewrite endpoints without renumbering the store.
#[test]
fn an_elided_index_on_the_substitution_side_keeps_the_matched_index() {
    let program = parse("((7: 1 $t)) ((9 $t))");

    let outcome = program.execute(&[link("7", "1", "2")]);

    assert_eq!(outcome.links, vec![link("7", "9", "2")]);
    assert_eq!(outcome.halt, RewriteHalt::NoApplicableRule);
}

/// link-cli: "Identical sub-links are created once and reused." Re-creating a
/// link the store already holds is therefore not a change — which is also what
/// makes a creation rule halt instead of appending forever.
#[test]
fn creating_a_link_the_store_already_holds_is_not_a_change() {
    let program = parse("() ((1 1))");
    let store = vec![link("4", "1", "1")];

    let outcome = program.execute(&store);

    assert_eq!(outcome.links, store);
    assert!(outcome.trace.is_empty(), "trace: {:?}", outcome.trace);
}

/// The ordered/restart semantics are the whole basis of the Turing-completeness
/// claim, so they are pinned over links exactly as they are over text. Rule 1
/// enables rule 0 here: if selection continued past rule 1 rather than
/// restarting, the store would stop at `b` and never reach `c`.
#[test]
fn selection_restarts_at_rule_zero_after_every_step() {
    let program = parse("((b $t) (a $t)) ((c $t) (b $t))");

    let outcome = program.execute(&[link("1", "a", "1")]);

    assert_eq!(outcome.links, vec![link("1", "c", "1")]);
    assert_eq!(
        outcome
            .trace
            .iter()
            .map(|step| step.rule_index)
            .collect::<Vec<_>>(),
        vec![1, 0]
    );
}

/// Turing completeness means a program may not halt, so the bound must be
/// observable rather than silent. A swap rule is the link-domain twin of the
/// text dialect's `a -> b`, `b -> a` cycle.
#[test]
fn a_swap_rule_cycles_until_the_step_bound_stops_it() {
    let program = parse("(($i: $s $t)) (($i: $t $s))");

    let outcome = program.execute(&[link("1", "1", "2")]);

    assert_eq!(outcome.halt, RewriteHalt::StepLimit);
    assert_eq!(outcome.trace.len(), STEPS);
}

/// The whole-side `()` shorthand abbreviates a per-operand empty link, so the
/// general form can mix effects in one query. Rule order still decides: the
/// creation is tried before the deletion on every step.
#[test]
fn one_query_can_mix_creation_and_deletion() {
    let program = parse("(() (2 2)) ((1 1) ())");

    assert_eq!(
        link_substitution_effect(&program.rules[0]),
        CrudEvent::Create
    );
    assert_eq!(
        link_substitution_effect(&program.rules[1]),
        CrudEvent::Delete
    );

    let outcome = program.execute(&[link("5", "2", "2")]);

    assert_eq!(outcome.links, vec![link("6", "1", "1")]);
}

/// A slot value that would not read back as one bare token has to survive
/// rendering, or a round trip would silently change the query's meaning.
#[test]
fn slots_that_need_quoting_round_trip_through_quotes() {
    let query = r#"() (("hello world" "a:b"))"#;
    let program = parse(query);

    let outcome = program.execute(&[]);

    assert_eq!(outcome.links, vec![link("1", "hello world", "a:b")]);
    assert_eq!(render_link_substitution_query(&program), query);
}

/// Slot values are not ASCII-gated, matching the rest of #715's multilingual
/// requirement: a value that is one word stays bare whatever script it is in.
#[test]
fn unicode_values_read_and_render_without_quotes() {
    let program = parse("() ((привет мир))");

    let outcome = program.execute(&[]);

    assert_eq!(outcome.links, vec![link("1", "привет", "мир")]);
    assert_eq!(
        render_link_substitution_query(&program),
        "() ((привет мир))"
    );
}

/// Rendering is canonical: link-cli's shorthand comes back out for the shapes
/// that have one, and every rendering parses to the program it came from.
#[test]
fn rendering_round_trips_through_the_parser() {
    for query in [
        "() ((1 1))",
        "((1 1)) ()",
        "((1: 1 1)) ((1: 1 2))",
        "(($i: $s $t)) (($i: $s $t))",
        "(() (2 2)) ((1 1) ())",
        "() ()",
    ] {
        let program = parse(query);
        let rendered = render_link_substitution_query(&program);

        assert_eq!(rendered, query, "{query} should render canonically");
        assert_eq!(
            parse(&rendered),
            program,
            "{query} should survive a round trip"
        );
    }
}

/// A variable that the matching side never bound has no value to resolve, so it
/// is rejected at parse time rather than resolving to something invented
/// mid-run. This is what keeps every parsed program total.
#[test]
fn an_unbound_substitution_variable_is_rejected_at_parse_time() {
    let error = parse_error("((1 1)) (($x 1))");

    assert!(
        error.message.contains("$x") && error.message.contains("never bound"),
        "{}",
        error.message
    );
}

/// link-cli's creation shorthand is `() ((1 1))`, whose operands are source and
/// target; the store assigns the index. What forcing a taken index should mean
/// is undefined, so it is refused rather than guessed.
#[test]
fn creation_may_not_force_an_index() {
    let error = parse_error("() ((5: 1 2))");

    assert!(error.message.contains("index"), "{}", error.message);
}

#[test]
fn mismatched_operand_counts_are_rejected() {
    let error = parse_error("((1 1) (2 2)) ((3 3))");

    assert!(error.message.contains("operand count"), "{}", error.message);
}

#[test]
fn a_link_needs_both_a_source_and_a_target() {
    for query in ["((1)) ()", "((1: 1)) ()", "((1 2 3)) ()"] {
        let error = parse_error(query);
        assert!(
            error.message.contains("link is written")
                || error.message.contains("unbalanced parentheses"),
            "{query} reported {}",
            error.message
        );
    }
}

#[test]
fn a_query_needs_two_sides() {
    assert!(parse_error("((1 1))").message.contains("found only one"));
    assert!(parse_error("() ((1 1)) ()")
        .message
        .contains("trailing input"));
}
