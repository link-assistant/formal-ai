use formal_ai::links_substitution_query::link_substitution_effect;
use formal_ai::memory::{MemoryEvent, MemoryStore};
use formal_ai::memory_program::{MemoryProgramAuthorization, MemoryProgramLimits};
use formal_ai::memory_query_language::{
    compile_memory_query, execute_memory_query, AggregateFunction, MemoryField,
    MemoryQueryCompiler, MemoryQueryValue, QueryDialect,
};
use formal_ai::substitution::CrudEvent;

const LIMITS: MemoryProgramLimits = MemoryProgramLimits {
    max_matches: 64,
    max_iterations: 8,
};

fn compile(source: &str, dialect: QueryDialect) -> formal_ai::memory_query_language::CompiledMemoryQuery {
    compile_memory_query(source, dialect, LIMITS).unwrap_or_else(|error| panic!("{source}: {error}"))
}

#[test]
fn exact_sql_and_graphql_reads_share_one_plan_and_identity_substitution() {
    let sql = compile(
        "SELECT id, kind, role, intent, tool, inputs, outputs, content, sentAt, demoLabel, conversationId, conversationTitle, evidence, accessCount, writeCount FROM memory WHERE kind = 'fact' AND role = 'user' ORDER BY accessCount DESC LIMIT 5",
        QueryDialect::SqlAnsi,
    );
    let graphql = compile(
        "query { memory(where: { kind: { eq: \"fact\" }, role: { eq: \"user\" } }, orderBy: { accessCount: DESC }, first: 5) { id kind role intent tool inputs outputs content sentAt demoLabel conversationId conversationTitle evidence accessCount writeCount } }",
        QueryDialect::GraphQl,
    );

    assert_eq!(sql.canonical_semantics(), graphql.canonical_semantics());
    assert_eq!(sql.plan.projection, MemoryField::ALL);
    for parsed in [&sql, &graphql] {
        assert_eq!(parsed.parser.engine, "meta_language");
        assert!(parsed.parser.full_match);
        assert!(parsed.parser.text_preserved);
        assert!(parsed
            .link_program
            .rules
            .iter()
            .any(|rule| link_substitution_effect(rule) == CrudEvent::Read));
        assert!(parsed.links_notation().contains("effect read"));
    }
}

#[test]
fn sql_crud_lowers_to_the_four_link_cli_substitution_shapes() {
    let cases = [
        (
            "SELECT content FROM memory WHERE id = 'm1'",
            CrudEvent::Read,
        ),
        (
            "INSERT INTO memory (id, kind, content) VALUES ('m2', 'fact', 'created')",
            CrudEvent::Create,
        ),
        (
            "UPDATE memory SET content = 'updated' WHERE id = 'm1'",
            CrudEvent::Update,
        ),
        ("DELETE FROM memory WHERE id = 'm1'", CrudEvent::Delete),
    ];

    for (source, expected) in cases {
        let query = compile(source, QueryDialect::SqlAnsi);
        assert!(
            query
                .link_program
                .rules
                .iter()
                .any(|rule| link_substitution_effect(rule) == expected),
            "{source} did not lower to {expected:?}: {}",
            query.links_notation(),
        );
    }
}

#[test]
fn aggregations_and_statistics_execute_over_the_shared_memory_schema() {
    let query = compile(
        "SELECT kind, COUNT(*) AS count, SUM(accessCount) AS accesses, AVG(writeCount) AS averageWrites, MIN(accessCount) AS minimumAccesses, MAX(accessCount) AS maximumAccesses, VAR_POP(accessCount) AS accessVariance, STDDEV_POP(accessCount) AS accessDeviation FROM memory GROUP BY kind ORDER BY kind ASC",
        QueryDialect::SqlAnsi,
    );
    assert_eq!(
        query
            .plan
            .aggregates
            .iter()
            .map(|aggregate| aggregate.function)
            .collect::<Vec<_>>(),
        [
            AggregateFunction::Count,
            AggregateFunction::Sum,
            AggregateFunction::Average,
            AggregateFunction::Minimum,
            AggregateFunction::Maximum,
            AggregateFunction::PopulationVariance,
            AggregateFunction::PopulationStandardDeviation,
        ]
    );

    let mut store = MemoryStore::from_events(vec![
        MemoryEvent {
            id: String::from("a"),
            kind: Some(String::from("fact")),
            access_count: 1,
            write_count: 1,
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("b"),
            kind: Some(String::from("fact")),
            access_count: 3,
            write_count: 3,
            ..MemoryEvent::default()
        },
    ]);
    let outcome = execute_memory_query(
        &query,
        &mut store,
        MemoryProgramAuthorization::ReadOnly,
    );
    assert_eq!(outcome.rows.len(), 1);
    assert_eq!(outcome.rows[0].get("count"), Some(&MemoryQueryValue::Integer(2)));
    assert_eq!(outcome.rows[0].get("accesses"), Some(&MemoryQueryValue::Integer(4)));
    assert_eq!(
        outcome.rows[0].get("averageWrites"),
        Some(&MemoryQueryValue::Float(2.0))
    );
    assert_eq!(
        outcome.rows[0].get("accessVariance"),
        Some(&MemoryQueryValue::Float(1.0))
    );
    assert_eq!(
        outcome.rows[0].get("accessDeviation"),
        Some(&MemoryQueryValue::Float(1.0))
    );
}

#[test]
fn a_learned_natural_language_template_reuses_the_exact_sql_plan() {
    let mut compiler = MemoryQueryCompiler::new();
    compiler
        .learn_natural_language_template(
            "Show {field} for {kind} memories.",
            "SELECT {field} FROM memory WHERE kind = '{kind}'",
            QueryDialect::SqlAnsi,
        )
        .expect("safe placeholders should be learnable");

    let learned = compiler
        .compile(
            "Show content for fact memories.",
            QueryDialect::NaturalLanguage,
            LIMITS,
        )
        .expect("the learned surface should compile");
    let exact = compile(
        "SELECT content FROM memory WHERE kind = 'fact'",
        QueryDialect::SqlAnsi,
    );

    assert_eq!(learned.canonical_semantics(), exact.canonical_semantics());
    assert!(compiler
        .learning_links_notation()
        .contains("natural_language_template"));
    assert!(learned.links_notation().contains("learned_from"));
}