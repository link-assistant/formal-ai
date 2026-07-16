# Root cause and generalized design

## Why the screenshot happened

Two independent capability gaps collapsed into the same visible symptom. OpenCode
did not advertise `websearch` unless its documented Exa switch was enabled, and
Formal AI had no report-action recipe. Both requests therefore fell through to an
ordinary textual answer even though each required an external action.

Fixing only the two strings would preserve the deeper error: treating a changing
agent harness as a fixed list of product-specific names. The generalized unit is
instead a typed capability advertised by the current request.

## Architecture after the fix

```text
seed meanings + interface capability catalog
                    |
natural-language intent -> capability recipe -> protocol tool call
                                                |
                                     Agent CLI permission/execution
                                                |
protocol tool result -> planner completion -> durable linked memory evidence
                                                |
                                  association, recall, and dreaming
```

The layers have deliberately separate responsibilities:

1. Seed data owns natural-language action meanings, aliases, report templates,
   repository identity, and interface capability descriptions.
2. The planner selects Search, Fetch, Read, Write, Edit, or Run and tracks a
   recipe's progress. It is independent of OpenCode, Claude Code, Gemini CLI, or
   another harness name.
3. Protocol adapters preserve the same call/result semantics in OpenAI Chat,
   OpenAI Responses, Anthropic Messages, and Gemini `generateContent` envelopes.
4. The Agent CLI executes external code and retains authority over permissions.
5. On a completed answer, the server extracts verified client tool executions and
   stores their inputs/outputs as evidence linked to the final task.

This boundary avoids two bad outcomes: fabricated prose claiming an action ran,
and a second privileged tool runtime hidden inside the model server. It also fixes
the prior learning gap where only the final sentence survived and the actual `gh`
or web result disappeared.

## Auto-learning consequence

The recorded tool event uses a stable id over prompt, capability, inputs, and
outputs rather than a transient protocol call id. Replaying the same exchange
therefore merges evidence. The normal memory pipeline can associate tool-qualified
events, recall them across conversations, count access/write frequency, and use
verified task records in dreaming without a special issue-714 learner.

This is intentionally a schema-level improvement: future tools inherit the same
recording behavior as soon as a client advertises and returns them.

## Remaining execution variants

Embedded web/desktop tools may execute inside their surface because that surface
is the client. Hosted provider tools may execute at the provider. In every case,
the protocol transcript remains the source of truth for what actually happened;
Formal AI records observed results and never infers successful side effects from
intent alone.
