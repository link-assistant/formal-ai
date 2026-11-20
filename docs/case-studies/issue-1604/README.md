# Issue 1604 · Dynamic substitutions and associative plugins

[Issue #1604](https://github.com/link-assistant/formal-ai/issues/1604) asks for a
runtime that can acquire computation as data, execute and explain it, and
reconsider which computation belongs in the compiled part of an instance.
[PR #1605](https://github.com/link-assistant/formal-ai/pull/1605) is the working
branch for the issue. The final issue description, issue discussion, and the
empty sub-issue response are in [raw data](raw-data/).

## Source evidence

The November 20 additions make plugin packaging, transactional CRUD events,
live self-modification, and dreaming material requirements. The
[discussion screenshot](raw-data/vision.png) was recovered as a 1290 × 2796 PNG.
It describes repeatedly applying normal Markov substitutions to data as a
computation model that can change at runtime. That is a property of the
unbounded model. This runtime measures bounded executions and reports when it
reaches an execution limit. The attached conversation also discusses using
the system to improve its own implementation. [Original attachment](https://github.com/user-attachments/assets/223b01b6-8d8a-414d-baef-58dfb52e2dc6)

The issue comment references an earlier Formal AI run whose serving identity was
unverified and whose capability count was zero. The authenticated runs below
used the branch's JavaScript server and recorded the observable outcomes.
[Repository case-study evidence](raw-data/comments.json)

## Investigation and alternatives

`links_substitution_query` already bridges pure text replacements and native
link substitutions. Rust's `normal_markov` gives an observable priority and
leftmost-match reference. Its `associative_packages` module already separates
artifact import from local handler permissions. The JavaScript server's shared
memory API records conversation events and exposes a read-only LinksQL route.
A distinct mutable workspace provides explicit write transactions without
changing the meaning of conversation log entries. The package manifest and
approved adapters follow the existing split between data and execution authority.
[Existing text and link query codec](../../../rust/src/links_substitution_query/mod.rs) ·
[Existing package sharing](../../../rust/src/associative_packages.rs)

Link Foundation's `transformer` applies structural rewrite rules to Links
Notation terms and demonstrates unary arithmetic and a small Turing machine.
It offers richer structural matching than literal text recipes. The host uses
the existing normal Markov codec for reversible text translation and the
doublet matcher for transactional graph changes. The `analyze` action can
detect priority shadowing and certify termination for a decreasing text
recipe before installation. The general termination and equivalence questions
remain open when these checks do not apply.
[Structural transformer examples](https://github.com/link-foundation/transformer/blob/main/examples/README.md) ·
[Recipe analysis](../../architecture/dynamic-meta.md#discovery-translation-and-compilation)

Link CLI describes CRUD as a two-sided substitution, and `ui-links` provides a
JavaScript matcher, in-memory doublets, names, and type helpers. Two newline-separated
sides fail upstream when the first contains two named patterns, because upstream
interprets that side as the wrapper. Wrapping the pair in a single group
disambiguates it before dispatch. The regression test asserts both resulting
updates. [link-cli README](https://github.com/link-foundation/link-cli) ·
[ui-links implementation](https://github.com/link-foundation/ui/tree/main/packages/ui-links) ·
[Upstream issue #270](https://github.com/link-foundation/ui/issues/270)

Deep Foundation builds typed links and packages around an explicit type relation.
The dynamic host adapts this to `Type → Type`, `Type → SubType`, and
`SubType → Value`, with ownership doublets and digest-addressed payloads. npm
and Cargo manifest metadata and GitHub file contents offer locators for a
reviewed plugin. A companion manifest keeps package registration and its exports
inspectable. Running npm lifecycle hooks, Cargo build scripts, or repository
source during discovery would grant those sources the serving process's authority.
[Deep Foundation deeplinks](https://github.com/deep-foundation/deeplinks) ·
[npm manifest](https://docs.npmjs.com/files/package.json/) ·
[Cargo manifest](https://doc.rust-lang.org/cargo/reference/manifest.html) ·
[GitHub repository contents API](https://docs.github.com/en/rest/repos/contents)

Per-rule writes with a fresh lookup on each action give different results when
one action changes another action's query input or activation state in the same
cycle. A regression replays two dependent reads with disjoint writes, plus an
incompatible write pair. Each query plans against the previous-cycle graph and
conflicting edits roll back. SQLite serializes writers and commits the state
and receipt together.
An out-of-process native ingress can use the same host and database, avoiding two
separate policy and transaction implementations. A native in-process host would
require replaying the transaction, guard, and trace corpus against its own storage
adapter. [SQLite transactions](https://www.sqlite.org/lang_transaction.html) ·
[Dynamic meta architecture](../../architecture/dynamic-meta.md)

Rebuilding the product binary per recipe would tie instance policy to a local
compiler and release process. Compiling the ordered patterns to an instance
dispatch table preserves priority, steps, and source, and can be reversed
without losing the original rules. An initial experiment found that the trie
alone took longer than native literal searches when no rule matched. The
compiled path now checks the first four priorities directly and can reject a
miss across the remaining patterns before walking the trie. A bounded replay
found equal outputs and traces and measured 140.8 ms interpreted and 9.4 ms
compiled over 50 runs on an input with repeated prefixes and 120 missed
patterns. The first-priority hit was sub-millisecond in both modes. Proposals
report usage. Timings are local observations for these inputs.
[Benchmark](../../../experiments/dynamic-meta/dispatch-benchmark.mjs) ·
[Captured timings](raw-data/dispatch-benchmark.txt)

## Requirements and verification

| Requirement | Implementation and check | Current boundary |
| --- | --- | --- |
| Discover online instructions and convert them to storable substitutions | The source reader bounds HTTPS retrieval, reports rejected blocks, and carries source digests into reviewed recipe and dataset imports. The live CLI fetched the pinned link-cli README and applied its create example as an installed dataset. Tests cover denied sources, redirects, and digest changes. | Candidate URLs can come from client research or operator selection. Text conversion accepts a controlled English and generated-code fragment. |
| Dynamic and static forms with two-way proposals | The same recipe runs ordered interpretation or a persisted compiled trie. Frequency tests exercise idle review, promotion, persistence across restart, and demotion. | Compilation selects an instance artifact used by the host. Product binary recompilation is outside that path. |
| Confirmation and server control, including admin text commands | CLI and HTTP accept commands such as `Compile recipe example/normalize` with explicit confirmation. Tests cover anonymous, member, and operator permissions. | One workspace is shared by the configured server credentials. |
| Use and compare recipes on demand with execution evidence | Chat, structured HTTP, and CLI expose recipe execution. Each run records priority, byte offsets, halt condition, and receipts. Analysis previews priority shadowing and certifies decreasing recipes before installation. Tests compare terminal and nonterminal variants. | Comparison observes the supplied input. General algorithm understanding or equivalence does not follow from a replay. |
| Associative package plugins with data, skills, functions, and integrations | Manifests expose dataset substitutions, CRUD-triggered rules, named recipes, and host adapters. Tests register npm, crate, and GitHub origins and inspect typed links. | Any origin can have a companion manifest. Downloaded package code has no automatic execution route. |
| Event-driven by default for link CRUD, including cascades | Test sequences create a link, update it, create another link, and schedule a delete reaction. Identical sides produce zero events. A repeating cascade rolls back its initiating write. | The mutable workspace has this CRUD path. Append-only conversation memory and prior read-only network views have separate semantics. |
| Real-time self-modification off by default with configuration and natural language controls | Runtime configuration and confirmed operator commands enable same-package rule activation. Tests observe the old rule set within one cycle and new selection on subsequent events. | Automatic changes select installed rules. Public calls cannot install arbitrary new rule source. |
| Independent transaction cycles and an upstream workaround when needed | Each rule query plans independently on the cycle snapshot. Regression tests exercise disjoint writes that share a query input and verify atomic rollback for competing writes. | The bounded scheduler evaluates plans locally and applies their edits in deterministic order. [Upstream report](https://github.com/link-foundation/ui/issues/270) |
| Basic translations across meta, programming, and natural languages | Tests round-trip the four formats in all directions and actually execute JavaScript and Python exports. | The portable fragment is reversible. General source-language imports are outside the fragment. |
| Overall architecture, vision, and dreaming equilibrium | Architecture guide describes data, authority, wire use, and shutdown. Vision names the usage-to-review loop. Idle review emits proposals. Related issue and PR received comments. | Comments are conditional on this PR landing. [Dreaming issue](https://github.com/link-assistant/formal-ai/issues/1446#issuecomment-7043871386) · [Dreaming PR](https://github.com/link-assistant/formal-ai/pull/1448#issuecomment-7043871367) |
| Use JavaScript Formal AI for implementation support and fact checks | A real Agent CLI invoked a branch recipe and a documentation audit. An exact-source request wrote executable JavaScript. The branch CLI generated the reviewed source rewrite that renamed a production binding. | Open-ended authoring returned `MissingContract`. The exact-source write also exposed a false-negative terminal verdict. Evidence below. |
| Collect evidence, inventory requirements, and evaluate existing libraries | This study links the authentic issue/PR material, screenshot, primary references, tests, and tool output. | No required sub-issues were present in the GitHub sub-issue response. |

## Formal AI exercises

An Agent CLI request initially received a 401 when its provider used an unresolved
environment token. With matching local test credentials, the open-ended request
for a JavaScript frequency-counting module reached Formal AI and returned
`MissingContract` with `SourceBoundImplementationAndIndependentValidationMissing`.
Providing the exact source bytes produced a real Agent CLI `write` and a file.
Importing that file in Node verified duplicate and empty-array cases. Formal AI's
follow-up response still said the declared write was unobserved. A subsequent
rename request produced a real source read, an `edit`, and a `sha256sum` readback.
Node independently checked the edited bytes and duplicate and empty-array cases.
The observed mechanical edits carry zero autonomous-authorship credit.
[Open-ended result](raw-data/agent-open-ended-authoring.json) ·
[Write transcript](raw-data/agent-write-literal.jsonl) ·
[Written source](raw-data/agent-written-frequency.mjs) ·
[Edit transcript](raw-data/agent-edit-function.jsonl) ·
[Edited source](raw-data/agent-edited-frequency.mjs)

A retry isolated the terminal verdict to workspace identity. Starting the server
and CLI in the same private working directory let the existing creation guard
bind the relative operand to the Agent CLI's absolute write path. The planner
requested a separate Read and reported the completed change. Node independently
executed the resulting function on duplicate and empty inputs. The same private
workspace still returned `MissingContract` on the open-ended request. The
paired readback exercised the existing guard with supplied source bytes.
[Literal request](raw-data/agent-paired-readback-request.txt) ·
[Paired readback transcript](raw-data/agent-paired-readback.jsonl)

The JavaScript branch CLI applied a reviewed replacement recipe to the complete
`discovery.mjs` source and produced both edits required by the abbreviation gate.
An independent byte comparison checked the output before it was written to the
working tree. The receipt records both offsets and the input and output hashes.
[Production source rewrite receipt](raw-data/source-dogfood.json)

The Agent CLI also asked the JavaScript server to run `example/normalize` on
`teh teh`. It returned `the the`, byte offsets 0 and 4, and receipt 2. An
independent CLI query of the SQLite store returned the same result.
[Chat transcript](raw-data/agent-run-recipe.jsonl) · [Store receipt](raw-data/recipe-receipt.json)

The local dynamic meta CLI fetched the upstream README at a pinned Git commit,
identified its untagged create query, persisted its provenance with an installed
dataset, and applied the dataset in a fresh SQLite workspace. The receipt shows
the resulting self-link `(1: 1 1)`.
[Discovery candidate](raw-data/link-cli-discovery.json) ·
[Dataset receipt](raw-data/link-cli-dataset-receipt.json) ·
[Upstream README](https://github.com/link-foundation/link-cli/blob/cc6008f06f19cf4aa81823025add2fec7dfe4ba5/README.md#substitution)

For the documentation fact check, the JavaScript planner selected a client
`statement-audit` command, which was executed using the installed Formal AI
0.355.27 CLI. The audit contrasted a deliberately incorrect default with the
host source, and checked the writer claim against fetched SQLite documentation.
The report assigned posterior 0 to “The default setting enables dynamic
self-modification” and 1 to “SQLite permits only one simultaneous write
transaction.” These were supplied source captures, so this verifies the
cross-check path using explicit evidence.
[Claims](raw-data/audit-claims.md) · [Capture metadata](raw-data/audit-captures.json) ·
[Agent transcript](raw-data/agent-audit.jsonl) · [Audit result](raw-data/statement-audit.lino)

## Reproduction

Run `timeout 600 node --test --test-concurrency=1 rust/tests/web/dynamic-meta.test.mjs`
after installing the root dependencies. The test worker limits the cascade and
uses the JavaScript host. With Agent CLI 0.27.0 on `PATH`, run
`node --test --test-concurrency=1 experiments/dynamic-meta/agent-cli.test.mjs`
to replay an installed dataset and compiled recipe through live HTTP and check
its persistent receipt. Use the [operator quick start](../../architecture/dynamic-meta.md#configure-and-run)
to inspect receipts manually. The review and hosted status history is in the
[pull request case study](../pull-request-1605/README.md).
