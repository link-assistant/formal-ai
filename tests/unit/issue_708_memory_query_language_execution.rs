use formal_ai::execute_memory_query_with_options;
use formal_ai::link_store::memory_events_to_link_records;
use formal_ai::links_substitution_query::{
    link_substitution_effect, parse_link_substitution_query, render_link_substitution_query,
};
use formal_ai::memory::{MemoryEvent, MemoryStore};
use formal_ai::memory_program::{
    MemoryProgramAuthorization, MemoryProgramHalt, MemoryProgramLimits,
};
use formal_ai::memory_query_language::{
    MemoryQueryCompiler, MemoryQueryLearningApproval, MemoryQueryLearningGate,
    MemoryQueryLearningObservation, MemoryQueryValue, QueryDialect, compile_memory_query,
    execute_memory_query,
};
use formal_ai::substitution::CrudEvent;
use formal_ai::{
    ChatCompletionRequest, ResponsesRequest, SolverConfig, UniversalSolver,
    create_chat_completion_with_solver_and_memory, create_response_with_solver_and_memory,
};

const LIMITS: MemoryProgramLimits = MemoryProgramLimits {
    max_matches: 32,
    max_iterations: 4,
};

fn compile(
    source: &str,
    dialect: QueryDialect,
) -> formal_ai::memory_query_language::CompiledMemoryQuery {
    compile_memory_query(source, dialect, LIMITS)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

#[test]
fn graphql_crud_and_statistics_share_the_sql_semantics() {
    let pairs = [
        (
            "INSERT INTO memory (id, kind, content) VALUES ('m2', 'fact', 'created') RETURNING id, kind, content",
            "mutation { createMemory(input: { id: \"m2\", kind: \"fact\", content: \"created\" }) { id kind content } }",
        ),
        (
            "UPDATE memory SET content = 'updated' WHERE id = 'm1' RETURNING id, content",
            "mutation { updateMemory(where: { id: { eq: \"m1\" } }, set: { content: \"updated\" }) { id content } }",
        ),
        (
            "DELETE FROM memory WHERE id = 'm1' RETURNING id",
            "mutation { deleteMemory(where: { id: { eq: \"m1\" } }) { id } }",
        ),
        (
            "SELECT kind, COUNT(*) AS count, SUM(accessCount) AS accesses, AVG(writeCount) AS averageWrites, MIN(accessCount) AS minimumAccesses, MAX(accessCount) AS maximumAccesses, VAR_POP(accessCount) AS accessVariance, STDDEV_POP(accessCount) AS accessDeviation FROM memory GROUP BY kind ORDER BY kind ASC",
            "query { memoryAggregate(groupBy: [kind], orderBy: { kind: ASC }) { count accesses: sum(field: accessCount) averageWrites: average(field: writeCount) minimumAccesses: minimum(field: accessCount) maximumAccesses: maximum(field: accessCount) accessVariance: variance(field: accessCount) accessDeviation: standardDeviation(field: accessCount) } }",
        ),
    ];

    for (sql, graphql) in pairs {
        assert_eq!(
            compile(sql, QueryDialect::SqlAnsi).canonical_semantics(),
            compile(graphql, QueryDialect::GraphQl).canonical_semantics(),
            "{sql}\n{graphql}",
        );
    }
}

#[test]
fn exact_crud_executes_with_permissions_and_append_only_retraction() {
    let mut store = MemoryStore::from_events(vec![MemoryEvent {
        id: String::from("m1"),
        kind: Some(String::from("fact")),
        content: Some(String::from("original")),
        ..MemoryEvent::default()
    }]);

    let insert = compile(
        "INSERT INTO memory (id, kind, content, accessCount) VALUES ('m2', 'fact', 'created', 7) RETURNING id, content, accessCount",
        QueryDialect::SqlAnsi,
    );
    let refused = execute_memory_query(&insert, &mut store, MemoryProgramAuthorization::ReadOnly);
    assert!(matches!(
        refused.halt,
        MemoryProgramHalt::PermissionDenied { ref required } if required == "write"
    ));
    let inserted = execute_memory_query(&insert, &mut store, MemoryProgramAuthorization::Write);
    assert_eq!(inserted.changed, 1);
    assert_eq!(
        inserted.rows[0].get("id"),
        Some(&MemoryQueryValue::Text(String::from("m2")))
    );

    let update = compile(
        "UPDATE memory SET content = 'updated' WHERE id = 'm1' RETURNING content",
        QueryDialect::SqlAnsi,
    );
    let updated = execute_memory_query(&update, &mut store, MemoryProgramAuthorization::Write);
    assert_eq!(updated.changed, 1);
    assert_eq!(store.events()[0].content.as_deref(), Some("updated"));

    let delete = compile(
        "DELETE FROM memory WHERE id = 'm1' RETURNING id",
        QueryDialect::SqlAnsi,
    );
    let refused = execute_memory_query(&delete, &mut store, MemoryProgramAuthorization::Write);
    assert!(matches!(
        refused.halt,
        MemoryProgramHalt::PermissionDenied { ref required } if required == "destructive"
    ));
    let deleted = execute_memory_query(
        &delete,
        &mut store,
        MemoryProgramAuthorization::DestructiveConfirmed,
    );
    assert_eq!(deleted.changed, 1);
    assert!(store.events().iter().any(|event| {
        event.kind.as_deref() == Some("memory_retraction") && event.inputs.as_deref() == Some("m1")
    }));

    let selected_after_delete = execute_memory_query(
        &compile(
            "SELECT id FROM memory WHERE id = 'm1'",
            QueryDialect::SqlAnsi,
        ),
        &mut store,
        MemoryProgramAuthorization::ReadOnly,
    );
    assert!(selected_after_delete.rows.is_empty());
    assert!(selected_after_delete.matched_ids.is_empty());
}

#[test]
fn common_sql_is_shared_across_declared_vendor_dialects() {
    let source = "SELECT id, content FROM memory WHERE accessCount >= 2 ORDER BY content ASC LIMIT 10 OFFSET 1";
    let ansi = compile(source, QueryDialect::SqlAnsi);
    for dialect in [
        QueryDialect::SqlPostgreSql,
        QueryDialect::SqlMySql,
        QueryDialect::SqlSqlite,
        QueryDialect::SqlMsSql,
        QueryDialect::SqlBigQuery,
    ] {
        let vendor = compile(source, dialect);
        assert_eq!(vendor.canonical_semantics(), ansi.canonical_semantics());
        assert_eq!(vendor.parser.grammar, "sql-ansi");
    }
}

#[test]
fn exact_parsers_reject_incomplete_or_out_of_schema_queries() {
    for (source, dialect) in [
        ("SELECT FROM memory", QueryDialect::SqlAnsi),
        ("SELECT content FROM secrets", QueryDialect::SqlAnsi),
        (
            "query { memory(where: { nope: { eq: 1 } }) { id } }",
            QueryDialect::GraphQl,
        ),
        ("query { memory { id ", QueryDialect::GraphQl),
        (
            "UPDATE memory SET accessCount = 1.5 WHERE id = 'm1'",
            QueryDialect::SqlAnsi,
        ),
        (
            "SELECT SUM(content) AS invalid FROM memory",
            QueryDialect::SqlAnsi,
        ),
    ] {
        assert!(
            compile_memory_query(source, dialect, LIMITS).is_err(),
            "{source} should fail exact parsing or schema lowering",
        );
    }
}

#[test]
fn every_compiled_query_round_trips_through_the_link_cli_parser() {
    for (source, dialect) in [
        (
            "SELECT content FROM memory WHERE kind = 'fact'",
            QueryDialect::SqlAnsi,
        ),
        (
            "INSERT INTO memory (id, content) VALUES ('m2', 'created')",
            QueryDialect::SqlAnsi,
        ),
        (
            "UPDATE memory SET content = 'updated' WHERE id = 'm1'",
            QueryDialect::SqlAnsi,
        ),
        ("DELETE FROM memory WHERE id = 'm1'", QueryDialect::SqlAnsi),
    ] {
        let compiled = compile(source, dialect);
        let rendered = render_link_substitution_query(&compiled.link_program);
        assert_eq!(
            parse_link_substitution_query(&rendered, compiled.link_program.max_steps),
            Ok(compiled.link_program),
            "{rendered}",
        );
    }
}

#[test]
fn lowered_programs_execute_over_projected_doublets_and_drift_is_refused() {
    let event = MemoryEvent {
        id: String::from("m1"),
        kind: Some(String::from("fact")),
        content: Some(String::from("original")),
        ..MemoryEvent::default()
    };
    let links = memory_events_to_link_records(std::slice::from_ref(&event))
        .into_iter()
        .flat_map(|record| record.links)
        .collect::<Vec<_>>();
    let cases = [
        (
            "SELECT content FROM memory WHERE id = 'm1'",
            CrudEvent::Read,
        ),
        (
            "INSERT INTO memory (id, content) VALUES ('m2', 'created')",
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
        if expected == CrudEvent::Read {
            assert!(!query.link_program.matched_links(&links).is_empty());
        } else {
            let outcome = query.link_program.execute(&links);
            assert!(
                outcome.trace.iter().any(|step| step.effect == expected),
                "{source}"
            );
        }
        assert!(
            query
                .link_program
                .rules
                .iter()
                .all(|rule| link_substitution_effect(rule) == expected)
        );
    }

    let mut drifted = compile(
        "SELECT content FROM memory WHERE id = 'm1'",
        QueryDialect::SqlAnsi,
    );
    drifted.link_program =
        compile("DELETE FROM memory WHERE id = 'm1'", QueryDialect::SqlAnsi).link_program;
    let mut store = MemoryStore::from_events(vec![event]);
    let refused = execute_memory_query(
        &drifted,
        &mut store,
        MemoryProgramAuthorization::DestructiveConfirmed,
    );
    assert!(matches!(
        refused.halt,
        MemoryProgramHalt::ProgramGap { ref primitive }
            if primitive == "memory_query_link_lowering:link_effect_drift"
    ));
    assert_eq!(refused.changed, 0);
}

#[test]
fn learned_templates_reject_placeholder_drift_and_query_injection() {
    let mut compiler = MemoryQueryCompiler::new();
    assert!(
        compiler
            .learn_natural_language_template(
                "Show {field} memories.",
                "SELECT {different} FROM memory",
                QueryDialect::SqlAnsi,
            )
            .is_err()
    );
    compiler
        .learn_natural_language_template(
            "Show {field} memories.",
            "SELECT {field} FROM memory",
            QueryDialect::SqlAnsi,
        )
        .expect("matching placeholder");
    assert!(
        compiler
            .compile(
                "Show content; DELETE FROM memory memories.",
                QueryDialect::NaturalLanguage,
                LIMITS,
            )
            .is_err()
    );
}

#[test]
fn repeated_successes_propose_learning_but_only_green_human_review_promotes_it() {
    let observations = [
        MemoryQueryLearningObservation::new(
            "Show content memories.",
            "SELECT content FROM memory",
            QueryDialect::SqlAnsi,
        ),
        MemoryQueryLearningObservation::new(
            "Show role memories.",
            "SELECT role FROM memory",
            QueryDialect::SqlAnsi,
        ),
    ];
    let candidate = MemoryQueryCompiler::infer_candidate(&observations, LIMITS)
        .expect("two exact successes should produce an inert candidate");
    assert_eq!(
        candidate.natural_language_template,
        "Show {value} memories."
    );
    assert_eq!(candidate.exact_query_template, "SELECT {value} FROM memory");

    let mut compiler = MemoryQueryCompiler::new();
    assert!(
        compiler
            .compile("Show tool memories.", QueryDialect::NaturalLanguage, LIMITS,)
            .is_err()
    );
    assert!(
        compiler
            .promote_candidate(
                candidate.clone(),
                &MemoryQueryLearningGate {
                    suite: String::from("issue_708_held_out"),
                    passed: 2,
                    failed: 1,
                },
                &MemoryQueryLearningApproval {
                    reviewer: String::from("maintainer"),
                    granted: true,
                },
            )
            .is_err()
    );
    assert!(
        compiler
            .promote_candidate(
                candidate.clone(),
                &MemoryQueryLearningGate {
                    suite: String::from("issue_708_held_out"),
                    passed: 3,
                    failed: 0,
                },
                &MemoryQueryLearningApproval {
                    reviewer: String::from("maintainer"),
                    granted: false,
                },
            )
            .is_err()
    );
    compiler
        .promote_candidate(
            candidate,
            &MemoryQueryLearningGate {
                suite: String::from("issue_708_held_out"),
                passed: 3,
                failed: 0,
            },
            &MemoryQueryLearningApproval {
                reviewer: String::from("maintainer"),
                granted: true,
            },
        )
        .expect("a green named gate and explicit approval may promote");

    let learned = compiler
        .compile("Show tool memories.", QueryDialect::NaturalLanguage, LIMITS)
        .expect("promotion should change later solving");
    assert_eq!(
        learned.canonical_semantics(),
        compile("SELECT tool FROM memory", QueryDialect::SqlAnsi).canonical_semantics()
    );
    let ledger = compiler.learning_links_notation();
    assert!(ledger.contains("promotion_policy"));
    assert!(ledger.contains("human_gated"));
    assert!(ledger.contains("gate_suite"));
    assert!(ledger.contains("issue_708_held_out"));
    assert!(ledger.contains("reviewer"));
    assert!(ledger.contains("maintainer"));
}

#[test]
fn solver_routes_exact_sql_and_graphql_with_auditable_results() {
    let mut store = MemoryStore::from_events(vec![MemoryEvent {
        id: String::from("m1"),
        kind: Some(String::from("fact")),
        content: Some(String::from("original")),
        access_count: 2,
        ..MemoryEvent::default()
    }]);
    let selected = execute_memory_query_with_options(
        "SELECT id, content FROM memory WHERE id = 'm1'",
        &mut store,
        None,
        LIMITS,
        MemoryProgramAuthorization::ReadOnly,
    )
    .expect("exact SQL route");
    assert_eq!(selected.answer.intent, "memory_exact_query");
    assert!(selected.answer.answer.contains("memory_query_result"));
    assert!(
        selected
            .answer
            .links_notation
            .contains("memory_exact_query_compiled")
    );
    assert!(
        selected
            .answer
            .links_notation
            .contains("link_cli_substitution")
    );

    let created = execute_memory_query_with_options(
        "mutation { createMemory(input: { id: \"m2\", kind: \"fact\", content: \"created\" }) { id content } }",
        &mut store,
        None,
        LIMITS,
        MemoryProgramAuthorization::Write,
    )
    .expect("exact GraphQL route");
    assert_eq!(created.answer.intent, "memory_exact_query");
    assert!(created.changed);
    assert!(store.events().iter().any(|item| item.id == "m2"));

    let rejected = execute_memory_query_with_options(
        "SELECT FROM memory",
        &mut store,
        None,
        LIMITS,
        MemoryProgramAuthorization::ReadOnly,
    )
    .expect("recognized exact query reports its parser failure");
    assert_eq!(rejected.answer.intent, "memory_exact_query_rejected");
    assert!(rejected.answer.answer.contains("memory_query_error"));
    assert!(!rejected.changed);
}

#[test]
fn agent_protocol_surfaces_execute_exact_reads_and_refuse_implicit_writes() {
    let events = vec![
        MemoryEvent {
            id: String::from("issue-708-alpha"),
            kind: Some(String::from("fact")),
            conversation_id: Some(String::from("issue-708-fixture")),
            access_count: 3,
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("issue-708-beta"),
            kind: Some(String::from("fact")),
            conversation_id: Some(String::from("issue-708-fixture")),
            access_count: 2,
            ..MemoryEvent::default()
        },
    ];
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let agent_request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [
            {"role": "system", "content": "You are an Agent CLI coding assistant."},
            {"role": "user", "content": "SELECT id, accessCount FROM memory WHERE conversationId = 'issue-708-fixture' ORDER BY accessCount DESC LIMIT 1"}
        ],
        "tools": [{
            "type": "function",
            "function": {"name": "bash", "parameters": {"type": "object"}}
        }]
    }))
    .expect("Agent-style request");
    let sql = create_chat_completion_with_solver_and_memory(&agent_request, &solver, &events)
        .choices[0]
        .message
        .content
        .plain_text();
    assert!(sql.contains("memory_query_result"), "{sql}");
    assert!(sql.contains("issue-708-alpha"), "{sql}");

    let response_request = ResponsesRequest {
        input: serde_json::Value::String(String::from(
            "query { memoryAggregate(where: { conversationId: { eq: \"issue-708-fixture\" } }) { count accesses: sum(field: accessCount) } }",
        )),
        ..ResponsesRequest::default()
    };
    let graphql = create_response_with_solver_and_memory(&response_request, &solver, &events);
    let graphql = &graphql.output_messages()[0].content[0].text;
    assert!(graphql.contains("memory_query_result"), "{graphql}");
    assert!(graphql.contains("count \"integer:2\""), "{graphql}");
    assert!(graphql.contains("accesses \"integer:5\""), "{graphql}");

    let mutation_request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "messages": [{"role": "user", "content": "DELETE FROM memory WHERE id = 'issue-708-alpha'"}]
    }))
    .expect("mutation request");
    let refused =
        create_chat_completion_with_solver_and_memory(&mutation_request, &solver, &events).choices
            [0]
        .message
        .content
        .plain_text();
    assert!(refused.contains("permission_denied"), "{refused}");
    assert_eq!(events.len(), 2, "immutable protocol memory must not mutate");
}
