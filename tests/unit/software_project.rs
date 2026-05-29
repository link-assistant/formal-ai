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
        ("Build a FastAPI service that imports support tickets and sends status notifications", "service"),
        ("Create a Node.js API for uploading files with retries and progress logs", "API"),
        ("Generate a command line tool with shell commands for backup checks and upload validation", "command-line tool"),
        ("Develop a web app for incident reports and run commands in WebVM after approval", "web app"),
    ];

    assert!(
        examples.len() >= 20,
        "software-project regression coverage must include at least 20 popular tasks"
    );

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

// Issue #341: the second step of a decomposed agent plan ("test it by scraping
// wikipedia.org and show me the top 10 most frequent words") used to fall
// through to a `wikipedia` concept lookup (online) or the unknown opener
// (offline). It must instead stay bound to the active software-project dialogue
// and formalize a verification follow-up.
#[test]
fn software_project_followup_keeps_test_step_in_the_project_dialogue() {
    let solver = UniversalSolver::default();
    let plan_prompt = concat!(
        "Design a simple web scraper in Python that:\n",
        "1. Fetches a webpage\n",
        "2. Extracts all headings (h1, h2, h3)\n",
        "3. Counts word frequency\n",
        "4. Generates a markdown summary"
    );
    let plan = solver.solve(plan_prompt);
    assert_eq!(plan.intent, "software_project_plan");

    let history = [
        ConversationTurn::user(plan_prompt),
        ConversationTurn::assistant(plan.answer),
    ];
    let follow_up = solver.solve_with_history(
        "test it by scraping wikipedia.org and show me the top 10 most frequent words.",
        &history,
    );

    assert_eq!(
        follow_up.intent, "software_project_followup",
        "follow-up returned {}: {}",
        follow_up.intent, follow_up.answer
    );
    assert!(follow_up.answer.contains("software_project_followup"));
    assert!(follow_up.answer.contains("follow_up_kind verification"));
    assert!(follow_up.answer.contains("target_site \"wikipedia.org\""));
    assert!(follow_up
        .answer
        .contains("expected_output \"the top 10 most frequent words\""));
    assert!(follow_up.answer.contains("parent_artifact \"scraper\""));
    assert!(follow_up
        .answer
        .contains("approval_gate \"test_execution\""));
    assert!(follow_up
        .answer
        .contains("approval_gate \"network_access\""));
    assert!(follow_up.answer.contains("Verification plan"));
    // The misrouted answers we are guarding against:
    assert!(!follow_up
        .answer
        .contains("free, multilingual online encyclopedia"));
    assert!(!follow_up.answer.contains("cannot answer that from local"));
}

// The follow-up handler must only fire while a software-project dialogue is
// active; otherwise "test it" with no prior plan stays a normal prompt.
#[test]
fn software_project_followup_requires_an_active_plan() {
    let solver = UniversalSolver::default();
    let answer = solver.solve("test it and show me the result");
    assert_ne!(answer.intent, "software_project_followup");
}

// After approval the follow-up reports the approved state so the dialogue stays
// consistent with the generated implementation starter.
#[test]
fn software_project_followup_reports_approved_state_after_implementation() {
    let solver = UniversalSolver::default();
    let plan_prompt = "Write a Python scraper that imports product prices and stores history";
    let plan = solver.solve(plan_prompt);
    let history = [
        ConversationTurn::user(plan_prompt),
        ConversationTurn::assistant(plan.answer),
    ];
    let implementation = solver.solve_with_history("approve plan", &history);
    assert_eq!(implementation.intent, "software_project_implementation");

    let history = [
        ConversationTurn::user(plan_prompt),
        ConversationTurn::assistant(implementation.answer),
    ];
    let follow_up = solver.solve_with_history("run it and verify the output", &history);
    assert_eq!(follow_up.intent, "software_project_followup");
    assert!(follow_up.answer.contains("approval_state approved"));
    assert!(follow_up.answer.contains("The plan is approved"));
}

// Issue #341 + generalization (R5): a multi-step dialogue must stay inside the
// project across every supported language. A user can design in English and
// then ask to exercise the artifact in Russian, Hindi, or Chinese; each
// follow-up verb has to route to `software_project_followup` instead of leaking
// to a concept lookup or the unknown opener.
#[test]
fn software_project_followup_detects_verbs_across_supported_languages() {
    let solver = UniversalSolver::default();
    let plan_prompt = "Write a Python scraper that imports product prices and stores history";
    let plan = solver.solve(plan_prompt);
    assert_eq!(plan.intent, "software_project_plan");
    let history = [
        ConversationTurn::user(plan_prompt),
        ConversationTurn::assistant(plan.answer),
    ];

    // (language tag, follow-up phrase, expected follow-up kind)
    let cases = [
        ("en english", "test it on wikipedia.org", "verification"),
        (
            "ru русский",
            "теперь протестируй его на wikipedia.org",
            "verification",
        ),
        ("hi हिंदी", "अब इसका परीक्षण करो", "verification"),
        ("zh 中文", "现在测试它", "verification"),
    ];
    for (language, prompt, kind) in cases {
        let follow_up = solver.solve_with_history(prompt, &history);
        assert_eq!(
            follow_up.intent, "software_project_followup",
            "language {language} prompt {prompt:?} returned {}: {}",
            follow_up.intent, follow_up.answer
        );
        assert!(
            follow_up.answer.contains(&format!("follow_up_kind {kind}")),
            "language {language} prompt {prompt:?} should record {kind}: {}",
            follow_up.answer
        );
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

#[test]
fn software_project_dialogue_examples_formalize_plan_then_implement_after_approval() {
    struct Example {
        prompt: &'static str,
        artifact: &'static str,
        delivery_mode: &'static str,
        language: &'static str,
        starter_label: &'static str,
        code_fence: &'static str,
        implementation_needle: &'static str,
        extra_gate: &'static str,
    }

    let solver = UniversalSolver::default();
    let examples = [
        Example {
            prompt: "Write an extension for Owlbear that tracks HP, Protection, Resistance, damage, and cooldowns",
            artifact: "extension",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "mitigateDamage",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Build a browser extension for reading progress that tracks pages and exports CSV",
            artifact: "browser extension",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Create a JavaScript Discord bot for scheduling game sessions with reminders",
            artifact: "bot",
            delivery_mode: "code_generation",
            language: "javascript",
            starter_label: "JavaScript",
            code_fence: "```javascript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Implement a React web app for invoices that tracks overdue payments and exports reports",
            artifact: "web app",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Make a plugin for a tabletop map that tracks unit status effects",
            artifact: "plugin",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Develop a Rust command line tool for renaming photos by date",
            artifact: "command-line tool",
            delivery_mode: "code_generation",
            language: "rust",
            starter_label: "Rust",
            code_fence: "```rust",
            implementation_needle: "pub enum ProjectCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Generate a mobile app for habit tracking with notifications and backups",
            artifact: "mobile app",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Design a service for importing customer invoices and sending payment reminders",
            artifact: "service",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Scaffold a website for event schedules that exports calendar data",
            artifact: "website",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Create a Python API for tracking equipment status and maintenance dates",
            artifact: "API",
            delivery_mode: "code_generation",
            language: "python",
            starter_label: "Python",
            code_fence: "```python",
            implementation_needle: "def apply_command",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Build a bot for project reports that sends weekly notifications",
            artifact: "bot",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Make an add-on for a tabletop token that tracks hp and damage",
            artifact: "extension",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "mitigateDamage",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Build a Python CLI tool for importing CSV tasks and exporting weekly reports with manual instructions",
            artifact: "command-line tool",
            delivery_mode: "manual_instructions",
            language: "python",
            starter_label: "Python",
            code_fence: "```python",
            implementation_needle: "def apply_command",
            extra_gate: "manual_instructions",
        },
        Example {
            prompt: "Write a Python scraper that imports product prices and stores history",
            artifact: "scraper",
            delivery_mode: "code_generation",
            language: "python",
            starter_label: "Python",
            code_fence: "```python",
            implementation_needle: "def apply_command",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Implement a Rust library for validating configuration files",
            artifact: "library",
            delivery_mode: "code_generation",
            language: "rust",
            starter_label: "Rust",
            code_fence: "```rust",
            implementation_needle: "pub enum ProjectCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Build an admin dashboard that filters users and exports audit logs",
            artifact: "dashboard",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Make a GitHub Action that checks changelog fragments on pull requests",
            artifact: "action",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Implement a plugin for a design tool that syncs assets and reports conflicts",
            artifact: "plugin",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Build a TypeScript SDK for uploading files with retries and progress events",
            artifact: "SDK",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Create a Telegram bot that tracks expenses and sends weekly reports",
            artifact: "bot",
            delivery_mode: "code_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_code",
        },
        Example {
            prompt: "Generate a command line tool with shell commands for backing up project files and validating upload status",
            artifact: "command-line tool",
            delivery_mode: "script_generation",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "generated_script",
        },
        Example {
            prompt: "Develop a web app for incident reports, run commands in WebVM, and approve each step",
            artifact: "web app",
            delivery_mode: "immediate_execution",
            language: "typescript",
            starter_label: "TypeScript",
            code_fence: "```typescript",
            implementation_needle: "export function applyCommand",
            extra_gate: "each_step",
        },
    ];

    assert!(
        examples.len() >= 20,
        "issue #80 review requested at least 20 full dialogue examples"
    );

    for example in examples {
        let plan = solver.solve(example.prompt);
        assert_eq!(
            plan.intent, "software_project_plan",
            "prompt: {:?} answer: {}",
            example.prompt, plan.answer
        );
        assert!(plan.answer.contains("```lino"));
        assert!(plan.answer.contains("software_project_request"));
        assert!(plan.answer.contains("approval_state proposed"));
        assert!(plan
            .answer
            .contains(&format!("artifact \"{}\"", example.artifact)));
        assert!(
            plan.answer
                .contains(&format!("delivery_mode {}", example.delivery_mode)),
            "prompt {:?} should expose delivery mode {} in {}",
            example.prompt,
            example.delivery_mode,
            plan.answer
        );
        assert!(plan
            .answer
            .contains(&format!("implementation_language \"{}\"", example.language)));
        assert!(plan.answer.contains("approval_gate \"task_formalization\""));
        assert!(plan
            .answer
            .contains("approval_gate \"implementation_plan\""));
        assert!(plan
            .answer
            .contains(&format!("approval_gate \"{}\"", example.extra_gate)));
        assert!(plan.answer.contains("requirement_category"));
        assert!(plan.answer.contains("requirement graph"));
        assert!(plan.answer.contains("implementation subtask(s)"));
        assert!(plan.answer.contains("Reasoning steps"));
        assert!(plan.answer.contains("Classify the impulse as a request"));
        assert!(plan.answer.contains("Select delivery mode"));
        assert!(plan.answer.contains("Ask for approval"));
        assert!(plan.answer.contains("Requirement model"));
        assert!(plan.answer.contains("Subtasks"));
        assert!(plan.answer.contains("Approval gates"));
        assert!(plan.answer.contains("Proposed plan"));
        assert!(plan.answer.contains("Review the formalized task"));
        assert!(plan.answer.contains("approve plan"));
        assert!(
            !plan.answer.contains(example.code_fence),
            "first turn must not generate code before approval: {}",
            plan.answer
        );

        let history = [
            ConversationTurn::user(example.prompt),
            ConversationTurn::assistant(plan.answer),
        ];
        let implementation = solver.solve_with_history("approve plan", &history);
        assert_eq!(
            implementation.intent, "software_project_implementation",
            "prompt: {:?} answer: {}",
            example.prompt, implementation.answer
        );
        assert!(implementation.answer.contains("approval_state approved"));
        assert!(implementation.answer.contains("software_project_request"));
        assert!(implementation.answer.contains("Implementation steps"));
        assert!(implementation.answer.contains("Generated code checks"));
        assert!(implementation
            .answer
            .contains(&format!("Starter {} core", example.starter_label)));
        assert!(implementation.answer.contains(example.code_fence));
        assert!(
            implementation
                .answer
                .contains(example.implementation_needle),
            "prompt: {:?} expected {} in {}",
            example.prompt,
            example.implementation_needle,
            implementation.answer
        );
    }
}
