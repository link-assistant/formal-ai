# Verified computer use without vision

Formal AI turns natural-language computer-use requests into deterministic plans
over twelve named primitives:

`fs.read`, `fs.write`, `fs.list`, `fs.move`, `shell.run`, `http.fetch`,
`http.post`, `dom.query`, `dom.extract`, `archive.pack`, `archive.unpack`, and
`process.status`.

The planner is intentionally non-visual. It operates on files, structured JSON
and HTML, and recorded HTTP fixtures. A request that needs pixels, a rendered
page, or a GUI-only application returns the named `gui_rendering`
`capability_gap`; it does not claim that a visual action happened.

## Native CLI

Both agent mode and effect confirmation are explicit:

```bash
formal-ai computer-use \
  --prompt "Filter active customers into a report" \
  --agent-mode \
  --confirm-effects \
  --replay
```

The JSON result contains the isolated workspace, the plan, each primitive's
arguments and output, and three events for every step: `precondition`, `effect`,
and `postcondition`. `--replay` executes the same plan in a fresh workspace and
independently compares the verified outputs. Omitting either permission flag
fails before a workspace effect.

The complete multilingual prompt set is in
[`data/seed/computer-use-tasks.lino`](../data/seed/computer-use-tasks.lino).
The tasks cover file transforms, structured GET/POST and extraction, report
generation, directory operations, archives, and isolated process status.

## Learning the plans instead of recalling them

Those tasks are *examples*, not an answer table. Formal AI induces a schema per
operation from them and then synthesizes plans for requests it has never seen.

```bash
formal-ai computer-use --learn
```

The output is Links Notation: one schema per operation cue with the primitive,
the fields all its examples agree on (constants) and those they differ on
(slots), one binding per resource naming the steps that materialise it and the
parameters that describe it, plus two honesty sections — `rejected` operations
whose examples disagree on a signature, and `unexplained` recorded steps no cue
accounts for. Nothing is guessed to make the table look complete. The induced
schemas are committed as reviewable evidence at
[`docs/case-studies/issue-707/learned-schemas.lino`](case-studies/issue-707/learned-schemas.lino);
`tests/issue_707_learning.rs` fails if a code change makes them drift.

Synthesis binds each field from the source that owns it, which is what keeps a
plan from being a memorised one:

- **Paths** come from the data flow — the resource binding materialises a file,
  each step consumes the previous step's artifact. A path is never copied from
  the example that taught the operation.
- **Resource-scoped fields** (`selector`, `pointer`, `column`, `equals`) come
  only from the resource binding. If the corpus never evidenced one for the
  named resource, the request yields *no plan* rather than a plausible wrong one.
- **Operation-scoped constants** are inherited only when the primitive's own
  advertised schema declares the field *and* the learned operation schema
  actually uses it. Two independent gates keep a field learned for one operation
  out of another.

The held-out ratchet is
[`data/benchmarks/computer-use-generalization.lino`](../data/benchmarks/computer-use-generalization.lino):
twelve requests in four languages, none of which appears in the recorded corpus.
`tests/issue_707_generalization.rs` asserts each one synthesizes (plan ids start
with `synthesized-`), that the four languages of a case agree on a single plan,
that every step of every plan executes with all three verification events
passing in its own workspace, and that out-of-boundary requests are refused with
the named capability gap in all four languages.

The recipe that produced this stage — its ordered steps, invariants, seed roles,
handlers, and benchmarks — is
[`data/meta/computer-use-recipe.lino`](../data/meta/computer-use-recipe.lino),
kept honest by `tests/unit/specification/computer_use_meta_algorithm.rs`.

## Server and external agents

Start the server with computer-use tools enabled:

```bash
formal-ai serve --host 127.0.0.1 --port 8080 --agent-mode
```

The MCP endpoint at `http://127.0.0.1:8080/mcp` advertises
`formal_ai_<primitive>` tools alongside `formal_ai_chat`. Each primitive schema
requires a plan id, step id, precondition, and postcondition. Without
`--agent-mode`, calls return a structured policy refusal.

Set `FORMAL_AI_COMPUTER_USE_AUDIT_PATH` to an operator-owned JSONL path to
persist one complete step record per MCP call:

```bash
FORMAL_AI_COMPUTER_USE_AUDIT_PATH=/tmp/formal-ai-computer-use.jsonl \
  formal-ai serve --host 127.0.0.1 --port 8080 --agent-mode
```

The issue-707 acceptance harness connects the real Link Assistant Agent CLI to
both the OpenAI-compatible endpoint and MCP, executes all ten requests, restarts
the server, and executes all ten again:

```bash
cargo build --bin formal-ai
experiments/agent_cli_e2e/run_issue_707.sh
```

A second harness does the same for the twelve *held-out* requests. It first
synthesizes each plan locally as the reference, then requires the external
client to reach the same plan id and the same primitive sequence over the wire,
in both the record and the replay phase:

```bash
experiments/agent_cli_e2e/run_issue_707_generalization.sh
```

Both harnesses run in `release.yml`, and their recorded transcripts, audit
records, and manifests are committed under
[`docs/case-studies/issue-707/agent-cli-evidence/`](case-studies/issue-707/agent-cli-evidence/).

## Permissions and isolation

Every primitive has its own `tool:computer:<primitive>` permission. Native
plans receive those grants only after agent mode is selected; desktop calls
must receive the corresponding explicit grant from the renderer. State-changing
operations additionally require `confirmed: true`.

All native paths are relative, reject parent traversal and absolute paths, and
resolve beneath a fresh per-plan temporary workspace. `shell.run` accepts only
the structured `count_lines`, `filter_csv`, and `unique_csv` operations; it does
not pass user text to a shell. Desktop computer-use paths are separately
confined beneath the application's user-data `computer-use/<plan_id>` directory;
they never resolve against the repository workspace used by the older desktop
file tools. Native benchmark HTTP accepts only committed `fixture://` resources,
while the permission-gated desktop executor can use HTTP(S). GET and POST
outputs record method, URL, status, cache path, and SHA-256 provenance.

The deterministic archive format is `formal-ai-archive-v1`. It is designed for
verified task replay, not as a general replacement for ZIP or tar.
