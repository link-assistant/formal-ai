# Root-cause analysis (source level)

The failing type lives in `src/protocol.rs`.

## `ChatMessage.content` field — `src/protocol.rs:128`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default)]
    pub content: MessageContent,
    /// Tool calls an `assistant` turn is requesting (OpenAI `tool_calls`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    // ... tool_call_id, name, thinking_steps, reasoning_content, reasoning ...
}
```

## `MessageContent` enum — `src/protocol.rs:206`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<MessageContentPart>),
}

impl Default for MessageContent {
    fn default() -> Self {
        Self::Text(String::new())
    }
}
```

## Why explicit `null` fails

- `MessageContent` is `#[serde(untagged)]` with two variants — `Text(String)` and
  `Parts(Vec<MessageContentPart>)`. Neither accepts a JSON `null` (there is no unit variant).
- The field uses `#[serde(default)]`. In serde, `default` only supplies the default value when
  the **key is absent** from the object. When the key is present with value `null`, serde still
  hands that `null` to `MessageContent`'s deserializer, which fails → the whole untagged enum
  reports "data did not match any variant".
- Hence: omit `content` → default kicks in → 200; `content: ""` → matches `Text("")` → 200;
  `content: null` → 400.

## Where the fix goes

Change the `content` field attribute on `ChatMessage` (line 131) to route an explicit `null`
through the default, per the reporter's verified patch:

```rust
#[serde(default, deserialize_with = "deserialize_null_content")]
pub content: MessageContent,
```

with a free function:

```rust
fn deserialize_null_content<'de, D>(d: D) -> Result<MessageContent, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<MessageContent>::deserialize(d)?.unwrap_or_default())
}
```

## Test placement notes

- OpenAI-compatibility parsing tests live in `tests/unit/specification/openai_compatibility.rs`
  (994 lines; e.g. `chat_completion_refuses_tool_call_without_agent_mode` at line 931). This is
  the natural home for the suggested regression test
  `chat_completion_accepts_explicit_null_content_on_tool_call_turns`.
- Note there is a **second, older copy** of the protocol types at `tests/source/protocol.rs`
  (wired via `Cargo.toml` `[[test]] name = "source"`, `path = "tests/source/mod.rs"`). It is a
  vendored snapshot and differs from `src/protocol.rs`; the live fix belongs in `src/protocol.rs`.

## Consumers of `MessageContent` (impact surface)

`grep -rn "MessageContent" src/` — the type is referenced by:

- `src/protocol.rs` — definition, `ChatMessage`, `MessageContent::plain_text()`, constructors.
- `src/gemini.rs:14,71` — builds `MessageContent::Text(...)`.
- `src/anthropic.rs:38,282,294` — converts to/from `MessageContent::Text(...)`.
- `src/lib.rs:202` — re-export.

The fix is purely additive on the deserialize path, so these producers are unaffected.
