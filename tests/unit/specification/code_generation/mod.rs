//! Code-generation tests covering the top programming languages and the
//! execution-evidence requirements from issue #8.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: parameterized write_program templates.
// ---------------------------------------------------------------------------

fn assert_write_program_parameters(response: &SymbolicAnswer, language: &str, task: &str) {
    assert_eq!(response.intent, "write_program");
    assert!(
        response
            .links_notation
            .contains(&format!("program_parameter:language {language}")),
        "Links Notation trace should include language={language}, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains(&format!("program_parameter:task {task}")),
        "Links Notation trace should include task={task}, got: {}",
        response.links_notation
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| { link == &format!("response:write_program:{task}:{language}") }),
        "evidence links should include the parameterized response link, got: {:?}",
        response.evidence_links
    );
}

// ---------------------------------------------------------------------------
// Issue #252 acceptance: the popular-language sweep, shared by the single-turn
// and task-catalog children below as `(display name, slug, code fence)`.
// ---------------------------------------------------------------------------
const POPULAR_LANGUAGES: &[(&str, &str, &str)] = &[
    ("Rust", "rust", "```rust"),
    ("Python", "python", "```python"),
    ("JavaScript", "javascript", "```javascript"),
    ("TypeScript", "typescript", "```typescript"),
    ("Go", "go", "```go"),
    ("C", "c", "```c"),
    ("C++", "cpp", "```cpp"),
    ("Java", "java", "```java"),
    ("C#", "csharp", "```csharp"),
    ("Ruby", "ruby", "```ruby"),
];

mod follow_up;
mod single_turn;
mod task_catalog;
