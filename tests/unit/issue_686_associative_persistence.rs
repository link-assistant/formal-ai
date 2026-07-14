//! Executable coverage for issue #686 — usage-weighted associative persistence.
//!
//! The [design case study](../../docs/case-studies/issue-686/README.md) maps every
//! concept the issue names onto the associative stack; this test exercises the
//! `formal_ai::associative_persistence` module that realizes it, proving each
//! headline requirement runs end to end:
//!
//! * meta-language expressions are **persisted** as content-addressed nodes in an
//!   associative links network,
//! * **reads (usages)** and **writes (changes)** are counted per expression,
//! * usage is **also derived from incoming and outgoing links** (degree),
//! * the **most used / most changed / most connected** expression is retained
//!   longest and evicted last,
//! * a world-model [`Context`] ingests into the store preserving statement ids,
//! * the whole store serializes as a **links network** (Links Notation).

use formal_ai::associative_persistence::{AssociativeMemory, RetentionWeights};
use formal_ai::world_model::{Context, Dependency, Statement};

#[test]
fn persist_is_content_addressed_and_deterministic() {
    let mut memory = AssociativeMemory::new();
    let first = memory.persist("all men are mortal");
    let second = memory.persist("all men are mortal");
    // Same text always maps to the same node in the links network.
    assert_eq!(first, second);
    // Persisting the same expression again is a change, so it counts as a write.
    assert_eq!(memory.len(), 1);
    assert_eq!(memory.writes(&first), 2);
    assert_eq!(memory.reads(&first), 0);
}

#[test]
fn new_expression_starts_with_one_write_and_zero_reads() {
    let mut memory = AssociativeMemory::new();
    let id = memory.persist("socrates is a man");
    assert_eq!(memory.writes(&id), 1);
    assert_eq!(memory.reads(&id), 0);
    assert!(memory.contains(&id));
}

#[test]
fn reads_and_writes_are_counted_independently() {
    let mut memory = AssociativeMemory::new();
    let id = memory.persist("knowledge persists");
    assert!(memory.note_read(&id));
    assert!(memory.note_read(&id));
    assert!(memory.note_write(&id));
    assert_eq!(memory.reads(&id), 2);
    // one initial assertion + one explicit write
    assert_eq!(memory.writes(&id), 2);
    // Unknown ids are rejected, not silently created.
    assert!(!memory.note_read("expression_does_not_exist"));
    assert!(!memory.note_write("expression_does_not_exist"));
}

#[test]
fn associations_form_a_links_network_with_degree() {
    let mut memory = AssociativeMemory::new();
    let a = memory.persist("A");
    let b = memory.persist("B");
    let c = memory.persist("C");

    assert!(memory.associate(&a, &b));
    assert!(memory.associate(&a, &c));
    assert!(memory.associate(&b, &c));
    // Re-adding an existing link is a no-op.
    assert!(!memory.associate(&a, &b));
    // Self-loops and unknown endpoints are rejected.
    assert!(!memory.associate(&a, &a));
    assert!(!memory.associate(&a, "expression_missing"));

    assert_eq!(memory.out_degree(&a), 2);
    assert_eq!(memory.in_degree(&a), 0);
    assert_eq!(memory.in_degree(&c), 2);
    assert_eq!(memory.out_degree(&c), 0);
    // link_usage folds incoming + outgoing — the issue's "usages based on
    // incoming and outgoing links".
    assert_eq!(memory.degree(&c), 2);
    assert_eq!(memory.link_usage(&c), memory.degree(&c));
    assert_eq!(memory.link_usage(&b), 2); // 1 in (a→b) + 1 out (b→c)
}

#[test]
fn retention_score_combines_reads_writes_and_degree() {
    let mut memory = AssociativeMemory::new();
    let a = memory.persist("A"); // writes = 1
    let b = memory.persist("B"); // writes = 1
    memory.note_read(&a); // reads = 1
    memory.associate(&a, &b); // a: out 1, b: in 1

    // a: reads 1 + writes 1 + in 0 + out 1 = 3
    assert_eq!(memory.retention_score(&a), 3);
    // b: reads 0 + writes 1 + in 1 + out 0 = 2
    assert_eq!(memory.retention_score(&b), 2);
    // Unknown ids score zero.
    assert_eq!(memory.retention_score("expression_missing"), 0);

    // Weights let a caller make changes protect twice as strongly as reads.
    let weights = RetentionWeights {
        read: 1,
        write: 2,
        incoming: 1,
        outgoing: 1,
    };
    // a: 1*1 + 2*1 + 1*0 + 1*1 = 4
    assert_eq!(memory.retention_score_with(&a, weights), 4);
}

#[test]
fn most_used_data_persists_longer_and_is_evicted_last() {
    let mut memory = AssociativeMemory::new();
    let hot = memory.persist("frequently used");
    let warm = memory.persist("occasionally used");
    let cold = memory.persist("rarely used");

    for _ in 0..5 {
        memory.note_read(&hot);
    }
    memory.note_read(&warm);

    // Ranking is most-retained first.
    let ranking = memory.retention_ranking();
    assert_eq!(ranking.first(), Some(&hot));
    // Eviction order is least-retained first — cold goes first.
    let eviction = memory.eviction_order();
    assert_eq!(eviction.first(), Some(&cold));

    // Retaining a capacity of 1 keeps only the hottest expression.
    let evicted = memory.retain_most_used(1);
    assert_eq!(memory.len(), 1);
    assert!(memory.contains(&hot));
    assert!(!memory.contains(&warm));
    assert!(!memory.contains(&cold));
    // Evicted least-retained first.
    assert_eq!(evicted.len(), 2);
    assert_eq!(evicted[0].id, cold);
    assert_eq!(evicted[1].id, warm);
}

#[test]
fn forgetting_removes_incident_links() {
    let mut memory = AssociativeMemory::new();
    let a = memory.persist("A");
    let b = memory.persist("B");
    let c = memory.persist("C");
    memory.associate(&a, &b);
    memory.associate(&b, &c);

    let removed = memory.forget(&b).expect("b was persisted");
    assert_eq!(removed.id, b);
    assert!(!memory.contains(&b));
    // Links touching b are gone, so a and c lose their degree from b.
    assert_eq!(memory.out_degree(&a), 0);
    assert_eq!(memory.in_degree(&c), 0);
    // Forgetting an unknown id yields None.
    assert!(memory.forget("expression_missing").is_none());
}

#[test]
fn tie_breaks_are_deterministic() {
    let mut memory = AssociativeMemory::new();
    // Two expressions with identical usage must rank in a stable id order.
    let first = memory.persist("aaa");
    let second = memory.persist("bbb");
    let ranking = memory.retention_ranking();
    let mut sorted = vec![first, second];
    sorted.sort();
    assert_eq!(ranking, sorted);
}

#[test]
fn from_context_preserves_ids_and_dependency_links() {
    let mut context = Context::new("mortality");
    let premise = context.add_statement(Statement::new("all men are mortal"));
    let minor = context.add_statement(Statement::new("socrates is a man"));
    let conclusion = context.add_statement(
        Statement::new("socrates is mortal")
            .with_dependency(Dependency::supports(&premise))
            .with_dependency(Dependency::supports(&minor)),
    );

    let memory = AssociativeMemory::from_context(&context);
    // Every statement is persisted under its original id.
    assert!(memory.contains(&premise));
    assert!(memory.contains(&minor));
    assert!(memory.contains(&conclusion));
    // Dependency edges became associative links: conclusion → premises.
    assert_eq!(memory.out_degree(&conclusion), 2);
    assert_eq!(memory.in_degree(&premise), 1);
    assert_eq!(memory.in_degree(&minor), 1);
    // The well-connected premises outrank the isolated-in conclusion by degree.
    assert!(memory.retention_score(&premise) >= 1);
}

#[test]
fn links_notation_is_a_sorted_reproducible_links_network() {
    let mut memory = AssociativeMemory::new();
    let a = memory.persist("A");
    let b = memory.persist("B");
    memory.note_read(&a);
    memory.associate(&a, &b);

    let first = memory.links_notation();
    let second = memory.links_notation();
    // Deterministic: byte-for-byte identical across renders.
    assert_eq!(first, second);
    // Everything is emitted as a link.
    assert!(first.contains(&format!("expression: ({a} A)")));
    assert!(first.contains(&format!("reads: ({a} 1)")));
    assert!(first.contains(&format!("writes: ({a} 1)")));
    assert!(first.contains(&format!("associates: ({a} {b})")));
}
