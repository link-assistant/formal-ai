# Dynamic Meta Algorithms and Plugins

The dynamic meta host stores executable recipes and event rules beside a mutable
doublet workspace. Enable it in the JavaScript server with
`FORMAL_AI_DYNAMIC_META_STORE=/path/to/meta.sqlite` on Node 22.13 or later.
The TypeScript projection runs the same protocol and transaction model. A native
server can forward the protocol to a locally configured host. Conversation
memory remains an append-only event log with its own storage and retention.

## Configure and run

The local CLI acts with the authority of the OS user who can open the store.
These commands register the example and apply its dataset. The dataset creates a
doublet and the installed rule updates it in cycle 1.

```sh
export FORMAL_AI_DYNAMIC_META_STORE=/path/to/meta.sqlite
jq -Rs '{action:"install",manifest:.,confirm:true}' experiments/dynamic-meta/package.json |
  node js/server/main.mjs meta
printf '%s\n' '{"action":"dataset","dataset":"example/sample","confirm":true}' |
  node js/server/main.mjs meta
printf '%s\n' '{"action":"run","recipe":"example/normalize","input":"teh teh"}' |
  node js/server/main.mjs meta
```

Start `node js/server/main.mjs serve` with the same store to use
`POST /v1/dynamic-meta`. Set `FORMAL_AI_API_BEARER_TOKEN` for members of the
shared workspace and `FORMAL_AI_DYNAMIC_META_ADMIN_TOKEN` for its operator.
The latter is sent in `x-formal-ai-meta-admin`. Use different credentials for
these roles. The shared bearer can inspect data, run recipes, and confirm
workspace substitutions. Keep the feature off for anonymous clients or tenants
that require separate stores. Ephemeral serving leaves the store closed.

The OpenAI-compatible chat endpoint also accepts a prompt such as
`Run recipe example/normalize on "teh teh".` The recipe returns a JSON result
with the output, trace, halt reason, and receipt revision. Use the shared bearer.
For a native HTTP server, start the JavaScript host on a private loopback port,
use the same application bearer for both servers, and set
`FORMAL_AI_DYNAMIC_META_LOOPBACK=127.0.0.1:8181` on the native process.
Requests for the dynamic meta path and recognized OpenAI recipe prompts go to
that host, which owns the store and its permissions.

| Action | Request fields | Effect |
| --- | --- | --- |
| `inspect` | | Read the graph, active rules, package types, and current and last reviewed proposals. |
| `receipt` | `revision` | Read one of the latest 64 stored results. |
| `discover` | `sources`, `confirm` | Fetch and preview up to eight recipes or package manifests. Operator only. |
| `translate` | `from`, `to`, `source` | Convert a portable recipe between `links`, `english`, `javascript`, and `python`. |
| `run` | `recipe`, `input`, optional `maxSteps` | Run a named recipe and record usage and its step trace. |
| `compare` | `left`, `right`, `input`, optional `maxSteps` | Compare both executions on the supplied input. The trace records whether both halted normally. |
| `substitute` | `query`, `confirm` | Apply link-cli replacement syntax and run the resulting cascade. |
| `dataset` | `dataset`, `confirm` | Apply a registered dataset as a substitution. |
| `install` | `manifest`, `confirm`, optional `sha256` | Register the reviewed bytes. Operator only. |
| `uninstall` | `package`, `confirm` | Remove the registrations and their compilation choices. Previously committed workspace data persists. Operator only. |
| `command` | `command`, `confirm` | Change compilation, rule activation, or self-modification. Operator only. |
| `integration` | `integration`, `input`, `confirm` | Call a server-configured adapter. Operator only. |

For state changes, `expectedRevision` can guard the revision that was reviewed.
External integration calls have their own effects and do not accept revision
conditions. `discover`, `install`, `substitute`, `dataset`, `command`, `uninstall`,
and `integration` require the JSON Boolean `confirm: true`.

## Link transactions and associative packages

`(restriction) (substitution)` reads when both sides are identical, creates
from an empty left side, deletes to an empty right side, and updates by replacing
matched patterns. Rules can filter the event kind and either current endpoint.
`@id`, `@source`, and `@target` refer to the event link. Delete uses the previous
link. Updates and deletes also supply `@old-source` and `@old-target`.

The initiating query produces cycle 0. Each next cycle selects rules and plans
their queries from a shared snapshot of the graph, active rules, and
previous-cycle events. Planning can be partitioned by event. The host applies
edits in event and rule order. Incompatible writes to the same link, competing
new names or anonymous doublets, and edits that would leave missing references
roll back the transaction. Actual CRUD events are scheduled by default.
Set `FORMAL_AI_DYNAMIC_META_TRIGGERS=off` to
disable cascade scheduling. A single SQLite write transaction commits the graph,
rule state and receipt together. Excess work or a failed rule rolls back every
cycle. The trace records the executed query, rule, and before/after doublets.

Live rule activation is off by default. Configure
`FORMAL_AI_DYNAMIC_META_SELF_MODIFICATION=true` or confirm the operator command
`Enable dynamic self-modification.` An installed `setRule` action can then change
another installed rule in its own package. The new state applies on the next
cycle. Administrator commands can also enable or disable a named rule.

A `formal-ai-plugin/v1` manifest associates a pinned local directory, npm package,
Rust crate, or GitHub repository with executable recipes, rule-based skills,
dataset substitutions, and named integration adapters. The package type and
export kinds are encoded as doublets. `Package` has type `Type`, a category such
as `Package/npm` has type `Package`, and each package is an instance of its
category with a digest-addressed value. Ownership doublets link each package to
its typed exports. Package authors or operators can supply the manifest. npm
and Cargo installation, build scripts, and arbitrary downloaded source have no
execution path in the plugin catalog. An embedding JavaScript server can pass
`metaAdapters`, a map of explicitly configured implementations, to `startServer`.

## Discovery, translation, and compilation

An operator can call `discover` with `sources` containing `url`, `language`,
and optionally `sha256`. Set `FORMAL_AI_DYNAMIC_META_SOURCE_ORIGINS` to a
comma-separated list of approved HTTPS origins. Discovery obeys the server's
read-only network policy, follows no redirects, and limits each source to 128
KiB. It retains the URL and source digest. With `language: "plugin"`, the result
contains the exact manifest bytes to approve and an installation digest. With
`language: "markdown"`, it extracts at most eight supported fenced blocks and
records where unsupported tagged code was encountered. The reader recognizes
link-cli queries in untagged query blocks, `lino`, and `formal-ai-link` fences.
Use `language: "link-substitution"` to read a single query document. The preview
includes a `substitute` field for datasets and `links` for text recipes. Copy
the candidate's `provenance` into the chosen manifest export to retain the
source URL, content digest, and line in the installed package.

The working translation fragment is an ordered normal Markov program. For
example, `Replace "teh" with "the" and stop.` converts to
`(("teh")) ((terminal: "the"))` and can round-trip through the generated
JavaScript and Python rewrite template. Each step chooses the first rule that
matches and replaces its leftmost occurrence. The model is computationally
universal with unbounded resources. Actual execution returns after a terminal
rule, no match, a step limit, or an output limit. Import for JavaScript and Python
accepts the portable template and validates its rule table without evaluating
source. The runtime reports byte offsets. To inspect one step, set `maxSteps`
to `1`. When the halt reason is `step_limit`, pass the result's `output` as the
next input to advance the same recipe. A comparison reports agreement for
the supplied input.

The dynamic runtime interprets rules. `Compile recipe example/normalize`
builds and persists a failure-linked trie for static dispatch, and
`Interpret recipe example/normalize` removes that trie while keeping the recipe.
The compiled path handles the first four priorities with literal searches and
uses a compiled literal prefilter to skip trie traversal when no lower-priority
pattern can match. The trie preserves the leftmost occurrence of the highest
priority match.
A 64-invocation window with a minimum sample of 16 suggests compiling after at
least eight uses or interpreting a compiled recipe with at most one use.
The server recomputes the proposals after successful idle dreaming and stores
changed proposals as a receipt. Operators confirm implementation changes through
`command`. The dispatch table is an instance artifact built from data. Native
binary rebuilds and broad program translation remain distinct facilities.

The graph matcher lives in [`substitutions.mjs`](../../js/server/dynamic-meta/substitutions.mjs),
the transaction and policy in [`runtime.mjs`](../../js/server/dynamic-meta/runtime.mjs),
and the reproducible cases in [`dynamic-meta.test.mjs`](../../rust/tests/web/dynamic-meta.test.mjs).
