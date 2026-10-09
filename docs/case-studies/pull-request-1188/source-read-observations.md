# Typed source-read observations

Source payloads and provider diagnostics are separate from process status. Bare unframed source content remains unknown in status and completeness; current-window call ID and exact non-conflicting path are required before assigning a source observation. Declared incomplete/truncated/interrupted payloads cannot certify bytes or digests. Auxiliary persistence with unknown complete-file Read must remain unavailable while independently authorized target delivery continues with a truthful gap. Provider metadata is an optional outer ChatMessage.source_read field (sourceRead alias), separate from exact message.content bytes. The actual filesystem provider may set {path, success: true, complete: true, format: "raw"}; this declares a complete successful Read, never a process exit status. Consumers require an exact current-window call ID, nonconflicting requested path, matching provider metadata path and tool name. Bare JSON source, even a matching source-read-receipt/v1 object or source keys that look like failures, cannot certify its own transport status or completeness. Known complete numbered Read framing retains its existing decoder contract; it does not certify omitted terminal-byte information or a process status. Existing historical rationale is retained below while source contracts stay concise.

## Prior rationale 1

rust/src/agentic_coding/tool_result.rs

```rust
    /// The process exit status the harness reported, when it reported one.
    /// This is the *primary* success signal (issues #905 and #908); the failure
    /// lexicon below is only consulted when no harness reported a status.
```

## Prior rationale 2

rust/src/agentic_coding/tool_result.rs

```rust
/// What a finished tool step actually did, read from the harness's own report.
///
/// Issues #905 and #908 are the two directions of one defect: the verdict was
/// keyed on whether the result *text* looked like a failure, so `cat: …: No such
/// file or directory` with `Exit Code: 1` was read as success and `Exit Code: 0`
/// with `Output: (empty)` / `Error: (none)` was read as failure. Every harness in
/// the #902–#909 corpus reports the status explicitly, so it is read, not inferred.
```

## Prior rationale 3

rust/src/agentic_coding/tool_result.rs

```rust
/// The workspace's veto over a completion claim (issue #905).
///
/// "Completed … and verified it" is a claim about the workspace, so the
/// workspace gets the last word: issue #905 watched that sentence go out after
/// `cat hello.txt` had exited 1 because the file was never created. The
/// verification the plan itself named had already disproved the claim. When the
/// last observed step failed, the report of that failure replaces the claim.
```

## Prior rationale 4

rust/src/agentic_coding/tool_result.rs

```rust
/// Return the client-owned failure detail when a result failed, whether the
/// signal arrived as protocol metadata, a structured transport envelope, or a
/// raw adapter message.
```

## Prior rationale 5

rust/src/agentic_coding/tool_result.rs

```rust
/// Status-less adapters sometimes return only a provider's denial or error
/// notice. Inspect the leading diagnostic region for that fallback: successful
/// pages and documents can legitimately contain error vocabulary or HTTP codes
/// later in their contents.
```

## Prior rationale 6

rust/src/agentic_coding/tool_result.rs

```rust
/// How many cited lines it takes before a result counts as quoting.
///
/// One is not enough. `HTTP/1.1 404: Not Found` cites a place by the letter of
/// [`citation_offset`] -- `404` stands as its own token before a colon -- and it
/// is a diagnosis, not a quotation. A body of quotations comes in a list.
```

## Prior rationale 7

rust/src/agentic_coding/tool_result.rs

```rust
/// The part of a result the result is saying itself, before it starts quoting.
///
/// A search answers with other files' text, and every line it hands back says
/// where it was found -- `./scripts/install.sh:260:    log "the 'code' CLI was
/// not found on PATH."` from one harness, `  Line 65: - Failure-driven
/// splitting: ...` under a file heading from another. The failure vocabulary in
/// those lines belongs to `install.sh` and to the changelog, not to the search.
/// The diagnostic prefix cannot tell them apart by wording, so issue #1066
/// watched a `grep` that had matched a hundred lines report itself as the
/// command that failed, and the node whose only evidence was that search
/// recorded the failure as its proof.
///
/// The distinguishing property is not vocabulary and not the tool's name --
/// naming the tool would fix one caller and nothing else. It is that a quotation
/// says where it came from, so the result's own voice is what it says *before*
/// the first citation: a count, a heading, or nothing at all. The cut is made at
/// the citation itself rather than at the start of its line, because a step that
/// does fail often says so and then points at the place -- `Error: cannot read
/// src/lib.rs:12: No such file` keeps `Error: cannot read`, and is still read as
/// the failure it is. A harness announcing its own refusal cites no place at
/// all, so `grep: /etc/shadow: Permission denied` keeps the old reading in full.
```

## Prior rationale 8

rust/src/agentic_coding/tool_result.rs

```rust
/// Where in `line` the result stops speaking and starts naming what it quotes.
///
/// Both quoting shapes above put a decimal number immediately before the colon
/// that introduces the quoted text, and in both the number stands as a token of
/// its own -- after the path's colon in `install.sh:260:`, after a space in
/// `Line 65:`. Neither the word before it nor the punctuation around it is
/// English, which is why this asks about the number's position rather than about
/// any phrasing. The citation begins where the word carrying it begins.
```

## Prior rationale 9

rust/src/agentic_coding/tool_result.rs

```rust
/// Report a step the planner itself observed to have failed, naming the tool or
/// path it failed on. The plan's own execution state machine uses this when a
/// client-owned attempt came back marked as an error (issue #905).
```

## Prior rationale 10

rust/src/agentic_coding/tool_result.rs

```rust
/// Report a failed step. When the harness reported the status, the status is
/// named: attributing the failure to the harness hid which command failed and
/// why (issue #908, suggested fix 3).
```

## Prior rationale 11

rust/src/agentic_coding/tool_result.rs

```rust
/// Whether the latest tool result of this turn succeeded with no output.
///
/// An edit tool's empty reply is its success (PR #1188 T181).
```

## Prior rationale 12

rust/src/agentic_coding/tool_result.rs

```rust
/// The fixed report every agent-CLI shell adapter wraps a command result in:
/// `Command:` / `Directory:` / `Output:` / `Error:` / `Exit Code:` / `Signal:` /
/// `Process Group PGID:`. The exit code is the harness's own verdict, so reading
/// it is what keeps `Error: (none)` from being read as an error (issue #908) and
/// `No such file or directory` with `Exit Code: 1` from being read as output
/// (issue #905).
```

## Prior rationale 13

rust/src/agentic_coding/tool_result.rs

```rust
/// The text a connector's `structuredContent` stands for: a record with a
/// `title` and a `body` is a work item and reads as both, anything else as
/// itself.
```

## Prior rationale 14

rust/src/agentic_coding/tool_result.rs

```rust
/// The shell command a run-tool call carried, under whichever argument key its
/// client declares (issue #1154).
///
/// Every agentic CLI names the command argument differently: Claude Code's
/// `Bash` sends `command`, Codex's `exec_command` sends `cmd`, and `shell`-style
/// adapters send either `script` or an array of argv words. Seven modules once
/// kept seven local readings of this, and the two that read only `command`
/// (`progress.rs`, `workspace_change.rs`) went blind on Codex: the work-item
/// read URL never resolved, the read was never recorded as attempted, and the
/// planner re-planned the identical `gh issue view` 78 times in a row. One
/// reading now serves the whole module, so a new client spelling is fixed once.
///
/// The key preference is plain: `command`, then `cmd`, then `script`. A client
/// whose schema names the command property differently can be recognized before
/// any of those match through [`command_argument_key`], which resolves the
/// property whose own schema says it carries the shell command.
```

## Prior rationale 15

rust/src/agentic_coding/tool_result/source_observation.rs

```rust
/// The process exit status the harness reported for `raw`, when it reported one.
///
/// Callers that build their own report of a stopped step need the status the
/// workspace answered with, not a rendering of it: a recipe that stops on a
/// precondition says which check stopped it and with what code (issue #944).
```

## Prior rationale 16

rust/src/agentic_coding/tool_result/source_observation.rs

```rust
/// Whether the *harness itself* judged this step a failure: it named an error
/// field, or it reported a non-zero exit status (rung `R916-01`).
///
/// The failure lexicon [`step_outcome`] falls back on is deliberately not
/// consulted here. Callers that hold bytes which are legitimately file contents
/// use this: a file that merely *reads* like an error message is still that
/// file's contents, and only the harness can say the read failed.
```

## Prior rationale 17

rust/src/agentic_coding/tool_result/source_observation.rs

```rust
/// Read a shell-harness envelope, if the result is one. Callers that report a
/// step's outcome use this to quote the command's own text instead of the
/// transport wrapper; [`step_outcome`] reads the status the harness observed.
```

## Prior rationale 18

rust/src/agentic_coding/tool_result/source_observation.rs

```rust
/// Remove client transport wrappers while preserving the tool's actual text.
/// Agentic planners consume this form; durable protocol recording still keeps
/// the original result byte-for-byte.
```

## Prior rationale 19

rust/src/agentic_coding/tool_result/source_observation.rs

```rust
/// Return output after transport normalization when the transport itself
/// succeeded. Unlike [`normalized_payload`], this does not classify arbitrary
/// output vocabulary: verification targets are allowed to contain words such
/// as `error` or `failed` when those are the requested bytes.
```

## Prior rationale 20

rust/src/agentic_coding/progress.rs

```rust
    /// Whether this attempt read a work item through the shell (`gh issue
    /// view …`, or one of the REST fallbacks of issue #1155). Its successful
    /// payload is page evidence, while its retry budget remains independent
    /// from the fetch capability.
```

## Prior rationale 21

rust/src/agentic_coding/progress.rs

```rust
    /// Work-item URLs already read through a shell read call.
    ///
    /// This is deliberately separate from `attempted_fetches`: an empty or
    /// failed CLI read should fall back to the fetch capability, not make that
    /// independent retrieval route appear exhausted.
```

## Prior rationale 22

rust/src/agentic_coding/progress.rs

```rust
    /// Work-item URLs whose shell read failed, with the reason (issue #1155).
    ///
    /// A read that a client echoed without its status — or whose output was a
    /// banner rather than an issue — is a failed read, and the honest close of
    /// such a run lists every read tried and its result instead of planning
    /// against text that was never the issue.
```

## Prior rationale 23

rust/src/agentic_coding/progress.rs

```rust
    /// Every shell result of this turn, in arrival order.
    ///
    /// A report runs one command per destination (#839), so keeping only the
    /// last one would drop the export results the moment the issue was filed.
```

## Prior rationale 24

rust/src/agentic_coding/progress.rs

```rust
    /// Shell observations keyed by the exact command supplied to the client.
    /// General plans use this to keep an auxiliary `gh` read from being
    /// mistaken for their verification command.
```

## Prior rationale 25

rust/src/agentic_coding/progress.rs

```rust
    /// Whether this turn already tried to fetch `url`.
    ///
    /// A fetch the harness echoed back without its `url` -- the argument was
    /// projected away onto a schema that has no such property -- still counts:
    /// the planner asked for exactly one page this turn, so a fetch attempt
    /// that names no URL was the attempt on that page. Reading only the echoed
    /// URL is what let issue #1133's Kotlin run plan the same call 547 times.
```

## Prior rationale 26

rust/src/agentic_coding/progress.rs

```rust
    /// Why the work item at `url` could not be read, when no read of it
    /// succeeded (issue #1155). The reason is the latest failure recorded for
    /// that URL — the last word the retrieval got.
```

## Prior rationale 27

rust/src/agentic_coding/progress.rs

```rust
    /// Every read attempted for `url` this turn, as report lines — the command
    /// (or fetch) tried, what it answered, and why it was not page evidence.
    /// An honest close lists these rather than declaring the issue unread
    /// without evidence.
```

## Prior rationale 28

rust/src/agentic_coding/progress.rs

```rust
    /// The prior attempt one planned call would repeat, if any (issue #1154).
    ///
    /// The planner is stateless: it re-derives the next step from the
    /// transcript, so a step that asks for a call this turn already *completed*
    /// is the signature of a loop, not a plan -- the earlier result is in the
    /// transcript, and a planner that still derives the same call ignored it
    /// (Codex planned the identical `gh issue view` 78 times, every one of them
    /// successful). A *failed* call is not matched here: re-planning it once is
    /// the existing bounded retry, and the second identical failure is already
    /// stopped by [`Progress::identical_failures_of`]. The match is
    /// byte-identical arguments or the same canonical operand, because the
    /// protocol layer may project the planner's `command` key onto the
    /// client's `cmd` before the transcript echoes the call back.
    ///
    /// A completed call is a loop only when the transcript made no progress
    /// since it: every attempt after the latest matching one must itself
    /// repeat an earlier attempt. A recipe that re-checks a precondition after
    /// a state-changing step (`test -e a.txt` again once `cp a.txt b.txt` ran,
    /// confirming the copy kept its source) asks the same question of a
    /// changed workspace, which is a plan, not a loop.
```

## Prior rationale 29

rust/src/agentic_coding/progress.rs

```rust
    /// Whether the recipe the latest successful `capability` attempt opened
    /// is still progressing: nothing attempted after it has failed. A later
    /// failure retires the record -- kept armed, it would re-plan the step
    /// that just failed round after round (the opencode greeting loop, where
    /// one `hi` search kept re-authorizing dictionary fetches that 403'd).
```

## Prior rationale 30

rust/src/agentic_coding/progress.rs

```rust
    /// Successful read payload for one concrete workspace path.
    ///
    /// Multi-step transformations read both their source and their written
    /// destination. Keying by call arguments keeps a later read-back from
    /// being mistaken for the source observation (or vice versa).
```

## Prior rationale 31

rust/src/agentic_coding/progress.rs

```rust
    /// Content supplied to the latest successful write of `path`.
    ///
    /// A composed request can deliver one observation to more than one file.
    /// The later delivery must recover the observation from the earlier write,
    /// not use the earlier writer's human-facing completion status as data.
```

## Prior rationale 32

rust/src/agentic_coding/progress.rs

```rust
    /// The arrival index of the latest successful write of `path`, so a
    /// caller can tell which of two files was written last (issue #1185 R3:
    /// a fix already rendered into the artifact is not rendered twice).
```

## Prior rationale 33

rust/src/agentic_coding/progress.rs

```rust
    /// The capability of the most recent tool result in this turn.
    ///
    /// `completed` is in arrival order, so this distinguishes *which phase* a
    /// multi-round loop is in — a search that has not been read yet, versus a
    /// completed read — which [`Progress::done`] alone cannot, since it stays
    /// true for every later round.
```

## Prior rationale 34

rust/src/agentic_coding/progress.rs

```rust
/// Resolve which capability the tool result at `index` answers. Prefer the
/// result's own `name`; otherwise map its `tool_call_id` back to the tool name in
/// a prior assistant `tool_calls` turn.
```

## Prior rationale 35

rust/src/agentic_coding/progress.rs

```rust
/// The GitHub work-item URL a shell read command targets, normalized to its
/// `github.com` form. `gh issue view <url> …` hands its URL over directly;
/// the REST fallbacks name the API path, which folds back onto the same
/// `https://github.com/{owner}/{repo}/{kind}/{number}` the plan targets —
/// REST `pulls/{n}` becomes `pull/{n}`, the way GitHub itself prints it.
```

## Prior rationale 36

rust/src/agentic_coding/progress.rs

```rust
/// The exit status a read command printed about itself, when it printed one.
///
/// Since issue #1155 every read the planner plans ends with
/// `printf '__formal_ai_exit=%s' $?`, because clients echo tool output with
/// the status field dropped — a failed read then looks successful, and its
/// banner was accepted as an issue body. The last sentinel wins, as the shell
/// itself does for `$?`.
```

## Prior rationale 37

rust/src/agentic_coding/progress.rs

```rust
/// Why a work-item read's output is not page evidence, when it is not.
///
/// Three gates, in order of authority: the client's own error echo, the exit
/// status the command printed (which outranks a status-less success echo),
/// and the shape of the output. The `gh` reads print a title line, a blank
/// separator, then the body, so output without the separator is a banner, not
/// an issue; the raw-body `curl` read prints the body alone, so it is held to
/// non-empty text that does not begin like the JSON error blob the API
/// returns instead.
```

## Prior rationale 38

rust/src/agentic_coding/general_execution.rs

```rust
/// The same state machine entered from a resolved work item. Every step is
/// identical; only the completion differs — the harness voice instead of the
/// conversational one — because a user who hands over an issue reference
/// commissioned an artifact, while a user who types the task asked for a
/// conversation about a change (issue #1133, the literal-file run).
```

## Prior rationale 39

rust/src/agentic_coding/general_execution.rs

```rust
/// Plan the next step from what the fetched work item actually asks for.
///
/// The real corpus decides the shape here. The issues Hive Mind dispatches say
/// *"implement a Hello World program in Scala"* — a described artifact in a
/// named language, not literal bytes — so the same coding catalog that answers
/// that request when a user types it directly answers it here, through the
/// execution recipe its [`SymbolicAnswer`](crate::solver::SymbolicAnswer)
/// carries. The literal-file composer follows, for a work item that does spell
/// out a path and its contents.
///
/// Returning [`None`] means the work item named nothing this sandbox can
/// produce — an unsupported language, or prose with no artifact in it at all —
/// which keeps `planned_not_executed` truthful rather than inventing an
/// artifact the issue never asked for.
```

## Prior rationale 40

rust/src/agentic_coding/general_execution.rs

```rust
/// The step that reads the work item, through whichever client tool reaches it.
///
/// GitHub's structured CLI read comes first when the client can run it. Unlike
/// model-backed `WebFetch` tools, it returns source bytes rather than asking a
/// nested model to interpret the whole solve request. This keeps the read in
/// the checkout's credential and prevents a fetched issue from being solved
/// once inside the fetch tool and then misread as issue text by the outer plan.
///
/// A client with no run capability falls back to its fetch tool. That call
/// carries a data-declared extraction instruction rather than the user's solve
/// request, so required `prompt` fields cannot recursively execute the task.
/// A protocol-hosted fetch tool runs server-side, so the fetch itself is the
/// read and `gh` is never planned ahead of it (issue #904).
```

## Prior rationale 41

rust/src/agentic_coding/general_execution.rs

```rust
/// The pull-request URL the prompt names beside the work item ("Your prepared
/// Pull Request: …"), when it names one. It is the last resort of the read
/// fallbacks (issue #1155): the PR's title and body restate the issue it
/// resolves.
```

## Prior rationale 42

rust/src/agentic_coding/general_execution.rs

```rust
/// The REST segments of a GitHub work-item URL — `{owner}`, `{repo}`, the REST
/// spelling of the kind (`issues` / `pulls`), and the number. `None` for a
/// URL that is not one, in which case the REST fallbacks are simply not
/// planned for it.
```

## Prior rationale 43

rust/src/agentic_coding/general_execution.rs

```rust
/// The work-item read that needs no `gh` and no credential: GitHub's REST API
/// returns the raw body to `curl` for a public repository (issue #1155). It
/// is the route that serves the very case an unauthenticated `gh` used to end
/// a session for.
```

## Prior rationale 44

rust/src/agentic_coding/general_execution.rs

```rust
/// The authenticated REST read, for a checkout whose `gh` answers `api` but
/// could not render the view (issue #1155). Same two-call shape as the view
/// read, so its output carries the title-and-body form the reader validates.
```

## Prior rationale 45

rust/src/agentic_coding/general_execution.rs

```rust
/// The honest close of a work item no read could deliver (issue #1155).
///
/// `None` while any read route is still untried — [`plan_work_item_read`]
/// owns that order — so the report exists only once the retrieval is
/// genuinely exhausted, and then lists every read attempted for the target
/// with the result each one answered with.
```

## Prior rationale 46

rust/src/agentic_coding/general_execution.rs

```rust
/// The text of the issue this work item names, once the client has fetched it.
///
/// Only the page fetched from the plan's own target counts. A repository work
/// item carries one URL, and answering it with whatever page happened to be
/// fetched for some other reason this turn would plan against the wrong issue.
```

## Prior rationale 47

rust/src/agentic_coding/general_execution.rs

```rust
/// The completion a resolved work item earns: the harness voice the execution
/// recipe already uses — the artifact created, the command that verified it,
/// the output that command produced. A literal file the work item spelled out
/// is an artifact whose side effects belong to the requesting client, so it is
/// reported through the same recipe shape a composed program is (issue #1133,
/// the literal-file run). A task the user typed into the conversation keeps
/// the seeded conversational claim; its words are pinned by issues #905
/// and #916.
```
