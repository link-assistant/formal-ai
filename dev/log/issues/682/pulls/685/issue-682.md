# Issue #682

- Title: **OpenAI chat-completions: explicit `"content": null` on assistant tool-call turns returns 400 (breaks qwen)**
- URL: https://github.com/link-assistant/formal-ai/issues/682
- State: OPEN · Label: `bug` · Author: konard (Konstantin Diachenko) · Comments: 0

---

## Summary

The OpenAI Chat Completions request parser rejects an assistant message that has an
**explicit `"content": null`** together with `tool_calls`, with:

```
HTTP 400 invalid chat request: data did not match any variant of untagged enum MessageContent
```

`content: null` on an assistant tool-call turn is the standard OpenAI shape and is exactly
what **Qwen Code (`qwen`)** emits. As a result the qwen agent loop dies mid-conversation as
soon as it has to send back a prior assistant tool-call turn.

## Environment

- `formal-ai 0.282.0` (global `cargo install`), agent mode
- qwen 0.7.1 driven via `with-formal-ai --non-interactive qwen "<task>"`

## Reproduction

Minimal, server-only:

```bash
curl -sS -o /dev/null -w "%{http_code}\n" \
  http://127.0.0.1:8080/api/openai/v1/chat/completions \
  -H 'content-type: application/json' -H 'authorization: Bearer formal-ai' \
  -d '{"model":"formal-ai","messages":[
        {"role":"assistant","content":null,
         "tool_calls":[{"id":"c1","type":"function",
           "function":{"name":"grep_search","arguments":"{\"query\":\"x\"}"}}]}]}'
# => 400  invalid chat request: data did not match any variant of untagged enum MessageContent

# Control — omit "content" entirely, everything else identical:
# => 200
```

`content: ""` and omitting `content` both return 200; only the explicit JSON `null` fails.

End-to-end: a qwen autonomous run walks several turns and then aborts with
`[API Error: 400 invalid chat request: data did not match any variant of untagged enum
MessageContent ...]` and exits non-zero, so no task completes. In a 300-run live matrix
(5 CLIs × 6 tools × 10 phrasings) **only qwen** hit this — **9 qwen runs** died with this
400; the other four CLIs never send an explicit `content: null` and had zero occurrences.

## Root cause (as stated in the issue)

`ChatMessage.content` is typed `MessageContent` (an untagged enum `Text(String) | Parts(Vec<..>)`,
no null variant) with `#[serde(default)]`. `#[serde(default)]` only supplies the default when
the key is **absent**; an explicit `null` is still fed to the untagged enum, which has no unit
variant, so deserialization fails.

## Suggested fix (verified by reporter)

Map an explicit `null` to the default via a small deserializer:

```rust
#[serde(default, deserialize_with = "deserialize_null_content")]
pub content: MessageContent,

fn deserialize_null_content<'de, D>(d: D) -> Result<MessageContent, D::Error>
where D: serde::Deserializer<'de> {
    Ok(Option::<MessageContent>::deserialize(d)?.unwrap_or_default())
}
```

The reporter applied exactly this against a local checkout: the minimal repro went 400 → 200
and the qwen autonomous run went from exit 1 (400 after ~8 requests) to exit 0 (full multi-turn
loop, no wire error). They offered a PR with this plus a regression test
(`chat_completion_accepts_explicit_null_content_on_tool_call_turns`).

## Related

- #680 — broader phrasing-gated tool-routing defect.
- #681 — write→read misclassification.
