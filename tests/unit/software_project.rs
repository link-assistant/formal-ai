use formal_ai::{ConversationTurn, FormalAiEngine, UniversalSolver};

#[test]
fn software_project_plan_exposes_requirement_graph_and_approval_preferences() {
    let prompt = concat!(
        "Build a Python CLI tool for importing CSV tasks, validating due dates, ",
        "and exporting weekly reports. I prefer manual instructions and approval ",
        "for each step before any shell command."
    );

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(response.intent, "software_project_plan");
    assert!(response.answer.contains("software_project_request"));
    assert!(response.answer.contains("Requirement model"));
    assert!(response.answer.contains("Subtasks"));
    assert!(response.answer.contains("requirement_category"));
    assert!(response
        .answer
        .contains("delivery_mode manual_instructions"));
    assert!(response
        .answer
        .contains("implementation_language \"python\""));
    assert!(response.answer.contains("approval_gate \"each_step\""));
    assert!(response.answer.contains("approval_gate \"bash_command\""));
    assert!(response.answer.contains("requirement graph"));
    assert!(!response.answer.contains("```python"));
}

#[test]
fn popular_software_project_prompts_use_the_general_formalization_path() {
    let examples = [
        ("Create a React web app for tracking workout progress with charts", "web app"),
        ("Build a Python REST API for invoices with overdue payment reminders", "API"),
        ("Write a Rust command line tool for renaming photos by date", "command-line tool"),
        ("Make a GitHub Action that checks changelog fragments on pull requests", "action"),
        ("Develop a browser extension that saves highlighted quotes and exports CSV", "browser extension"),
        ("Create a Discord bot for scheduling game sessions and sending reminders", "bot"),
        ("Scaffold a Node.js service that imports customer records and validates email addresses", "service"),
        ("Generate a mobile app for habit tracking with notifications and backups", "mobile app"),
        ("Design a website for event schedules that exports calendar data", "website"),
        ("Implement a plugin for a design tool that syncs assets and reports conflicts", "plugin"),
        ("Build a TypeScript SDK for uploading files with retries and progress events", "SDK"),
        ("Create a Telegram bot that tracks expenses and sends weekly reports", "bot"),
        ("Write a Python scraper that imports product prices and stores history", "scraper"),
        ("Implement a Rust library for validating configuration files", "library"),
        ("Build an admin dashboard that filters users and exports audit logs", "dashboard"),
        ("Make an Owlbear extension that tracks HP, protection, resistance, and cooldowns", "extension"),
    ];

    for (prompt, artifact) in examples {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "software_project_plan",
            "prompt {prompt:?} returned {}: {}",
            response.intent, response.answer
        );
        assert!(response.answer.contains("software_project_request"));
        assert!(response.answer.contains("Requirement model"));
        assert!(response.answer.contains("Subtasks"));
        assert!(response.answer.contains("Reasoning steps"));
        assert!(response.answer.contains("Proposed plan"));
        assert!(
            response
                .answer
                .contains(&format!("artifact \"{artifact}\"")),
            "prompt {prompt:?} should formalize artifact {artifact:?}: {}",
            response.answer
        );
        assert!(response
            .answer
            .contains("approval_gate \"implementation_plan\""));
        assert!(!response.answer.contains("intent: unknown"));
    }
}

#[test]
fn approval_returns_language_aware_generated_code_surface() {
    let solver = UniversalSolver::default();
    let examples = [
        (
            "Build a Python CLI tool for importing CSV tasks and exporting weekly reports",
            "```python",
            "def apply_command",
        ),
        (
            "Write a Rust command line tool for renaming photos by date",
            "```rust",
            "pub enum ProjectCommand",
        ),
        (
            "Create a JavaScript Discord bot for scheduling game sessions with reminders",
            "```javascript",
            "export function applyCommand",
        ),
        (
            "Build a TypeScript SDK for uploading files with retries and progress events",
            "```typescript",
            "export function applyCommand",
        ),
    ];

    for (prompt, code_fence, code_needle) in examples {
        let plan = solver.solve(prompt);
        let history = [
            ConversationTurn::user(prompt),
            ConversationTurn::assistant(plan.answer),
        ];
        let implementation = solver.solve_with_history("approve plan", &history);

        assert_eq!(implementation.intent, "software_project_implementation");
        assert!(implementation.answer.contains("approval_state approved"));
        assert!(implementation.answer.contains(code_fence));
        assert!(implementation.answer.contains(code_needle));
        assert!(implementation.answer.contains("Generated code checks"));
    }
}
