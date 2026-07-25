# Issue 840: grounded action recipe

Issue [#840](https://github.com/link-assistant/formal-ai/issues/840) asks Formal
AI to generalize several brittle user journeys into one topic-neutral
procedure. The implementation models an eight-step recipe in
[`grounded-action-recipe.lino`](../../../data/meta/grounded-action-recipe.lino):
declare meanings, recognize the route, extract typed slots, take the smallest
action, observe its result, widen only when justified, synthesize scoped
evidence, and preserve runtime parity.

## Reproduction and outcome

The 24-node task ladder decomposes issues #838, #827, and #826 through four
levels. On the pre-change `v0.303.0` baseline it passed 8/24 nodes. The local
search failure was especially diagnostic: changing “Find” to “Search” or
dropping “my” changed a local Desktop request into a web search.

After the change, the same release-mode ladder passes 24/24: 3/3 at L1, 6/6
at L2, 7/7 at L3, and 8/8 at L4. Its deterministic web fixture keeps the
measurement repeatable; the independent Agent CLI journeys below exercise
the corresponding research plans through real MCP tool calls.

The fix gives explicit local scope precedence over a generic search verb and
uses one observable command per step:

```text
exact name → stable substring → bounded inventory → scoped answer
```

An empty result advances the ladder; a failed command stops it. Candidate kind
and name are checked before synthesis, so a PEM file cannot satisfy a folder
request and a near match is reported as a mismatch. Definition follow-ups bind
only to their antecedent, comparisons research both sides separately, and the
report flow lowers each selected destination to one action.

## Independent evidence

The integration fixture contains:

```text
Desktop/
└── Archive/
    ├── hive-control-center/
    └── hive-mind-bot.2025-12-26.private-key.pem
```

The real Agent CLI journey drives that local search and the definition and
comparison cases through a release Formal AI server. Its native streams,
server traces, and dialogs are under [`agent-cli-e2e/`](agent-cli-e2e/).

The scoped uncertainty is deliberate: local absence means “not found after
exact, substring, and bounded inventory checks in the requested scope,” not
“does not exist anywhere.” Web synthesis is bounded by the evidence returned
by the available tools, and names sources when page fetches are available.

Formal AI also drove Agent CLI to author and verify one of the five smallest
reviewed leaves. Session `ses_069af4151ffep7T8HoP6ObsuBY` produced
[`grounded-action-authored-invariant.lino`](self-hosting/grounded-action-authored-invariant.lino);
the raw Agent stream and server trace are retained beside it. See
[`self-hosting/decomposition.lino`](self-hosting/decomposition.lino) for the
20% self-authorship accounting.
