use super::*;

#[must_use]
pub fn knowledge_graph() -> KnowledgeGraph {
    let mut nodes = vec![
        GraphNode {
            id: String::from("formal_ai_knowledge"),
            label: String::from("formal-ai knowledge root"),
            links_notation: String::from("formal_ai_knowledge"),
        },
        GraphNode {
            id: String::from("rule_greeting"),
            label: String::from("Greeting rule"),
            links_notation: format!("rule_greeting answer={}", greeting_answer()),
        },
        GraphNode {
            id: String::from("rule_identity"),
            label: String::from("Identity rule"),
            links_notation: format!("rule_identity answer={}", identity_answer()),
        },
        GraphNode {
            id: String::from("rule_assistant_name"),
            label: String::from("Assistant name rule"),
            links_notation: format!("rule_assistant_name answer={}", assistant_name_answer()),
        },
        GraphNode {
            id: String::from("rule_courtesy_response"),
            label: String::from("Courtesy response rule"),
            links_notation: format!(
                "rule_courtesy_response answer={}",
                courtesy_response_answer()
            ),
        },
        GraphNode {
            id: String::from("rule_assistant_free_time"),
            label: String::from("Assistant free-time rule"),
            links_notation: format!(
                "rule_assistant_free_time answer={}",
                assistant_free_time_answer()
            ),
        },
        GraphNode {
            id: String::from("rule_test_status"),
            label: String::from("Test status rule"),
            links_notation: format!("rule_test_status answer={}", test_status_answer()),
        },
        GraphNode {
            id: String::from("rule_unknown"),
            label: String::from("Unknown fallback rule"),
            links_notation: format!("rule_unknown answer={}", unknown_answer()),
        },
    ];
    let mut edges = vec![
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_greeting"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_identity"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_assistant_name"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_courtesy_response"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_assistant_free_time"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_test_status"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: String::from("rule_unknown"),
            role: String::from("contains"),
        },
        GraphEdge {
            from: String::from("rule_greeting"),
            to: String::from("response:greeting"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_identity"),
            to: String::from("response:identity"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_assistant_name"),
            to: String::from("response:assistant_name"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_courtesy_response"),
            to: String::from("response:courtesy_response"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_assistant_free_time"),
            to: String::from("response:assistant_free_time"),
            role: String::from("response_link"),
        },
        GraphEdge {
            from: String::from("rule_test_status"),
            to: String::from("response:test_status"),
            role: String::from("response_link"),
        },
    ];
    nodes.push(GraphNode {
        id: String::from("rule_write_program"),
        label: String::from("Write-program rule"),
        links_notation: format!(
            "rule_write_program parameters=language,task languages={} tasks={}",
            supported_program_languages(),
            supported_program_tasks()
        ),
    });
    edges.push(GraphEdge {
        from: String::from("formal_ai_knowledge"),
        to: String::from("rule_write_program"),
        role: String::from("contains"),
    });
    edges.push(GraphEdge {
        from: String::from("rule_write_program"),
        to: String::from("response:write_program"),
        role: String::from("response_link"),
    });
    let (package_nodes, package_edges) =
        crate::associative_package::default_package_graph_projection();
    nodes.extend(package_nodes);
    edges.extend(package_edges);
    KnowledgeGraph { nodes, edges }
}
