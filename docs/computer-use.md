# Verified computer use without vision

Formal AI validates and executes computer-use plans over twelve named primitives.

`fs.read`, `fs.write`, `fs.list`, `fs.move`, `shell.run`, `http.fetch`,
`http.post`, `dom.query`, `dom.extract`, `archive.pack`, `archive.unpack`, and
`process.status`.

The planner is intentionally non-visual. It operates on files, structured JSON
and HTML, and authorized HTTP responses. A request that needs pixels, a rendered
page, or a GUI-only application returns the named `gui_rendering` capability gap.

## Native CLI

Supply an ordered JSON array of primitive names and their arguments in a fenced
`computer-use` block. Agent mode grants the primitive permissions, and effect
confirmation authorizes the run.

````bash
formal-ai computer-use --agent-mode --confirm-effects --prompt "$(cat <<'PLAN'
Write the supplied note, then verify the saved file.
```computer-use
[
  {"primitive":"fs.write","arguments":{"path":"note.txt","content":"Review Thursday","confirmed":true}},
  {"primitive":"fs.read","arguments":{"path":"note.txt"}}
]
```
PLAN
)"
````

The JSON result contains the isolated workspace, the plan, and each scheduled
primitive's arguments, output, and three events. The events record the
precondition, effect, and postcondition. A failed step halts the plan.
`--replay` repeats all effects in a fresh workspace and compares the observed
outputs. Use it only for operations that may safely be repeated.

## Learn from a supplied corpus

Pass a Links Notation teaching file through `--from`. Formal AI uses this file
when planning a named operation or learning how to compose multiple steps.
`--plan` prints a validated request for an external tool client without
executing the plan.

```bash
formal-ai computer-use --learn --from teaching-corpus.lino
```

The multilingual [replay corpus](../rust/tests/fixtures/computer-use/tasks.lino)
is an explicit input to the test harnesses.

The output is Links Notation: one schema per operation cue with the primitive,
the fields all its examples agree on (constants) and those they differ on
(slots), one binding per resource naming the steps that materialise it and the
parameters that describe it, plus two honesty sections — `rejected` operations
whose examples disagree on a signature, and `unexplained` recorded steps no cue
accounts for. Nothing is guessed to make the table look complete. The induced
schemas are committed as reviewable evidence at
[`docs/case-studies/issue-707/learned-schemas.lino`](case-studies/issue-707/learned-schemas.lino);
`rust/tests/issue_707_learning.rs` fails if a code change makes them drift.

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
`rust/tests/issue_707_generalization.rs` asserts each one synthesizes (plan ids start
with `synthesized-`), that the four languages of a case agree on a single plan,
that every step of every plan executes with all three verification events
passing in its own workspace, and that out-of-boundary requests are refused with
the named capability gap in all four languages.

The recipe that produced this stage — its ordered steps, invariants, seed roles,
handlers, and benchmarks — is
[`data/meta/computer-use-recipe.lino`](../data/meta/computer-use-recipe.lino),
kept honest by `rust/tests/unit/specification/computer_use_meta_algorithm.rs`.

## Server and external agents

Start the server with computer-use tools enabled:

```bash
formal-ai serve --host 127.0.0.1 --port 8080 --agent-mode
```

The MCP endpoint at `http://127.0.0.1:8080/mcp` advertises the twelve tools and
`formal_ai_chat`. Clients such as Link Assistant Agent prefix the tool names
with `formal_ai_`. Use the same fenced plan syntax in a chat request. The
planner supplies the plan id, step id, precondition, and postcondition for
each call. Tool execution requires agent mode.

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
cargo build --manifest-path rust/Cargo.toml --bin formal-ai
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
they resolve beneath the desktop application's own data directory.

Set `FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN` to the authorized HTTP(S) origin for the
native CLI and the native or Node MCP server. Each request must use that origin,
and the existing operator network deny settings still apply. GET and confirmed
POST record the observed method, URL, status, media type, cache path, and
SHA-256. The transport uses curl, accepts a UTF-8 response within 1 MiB, and
requires a successful HTTP status. The replay harnesses configure a loopback
publisher.

The deterministic archive format is `formal-ai-archive-v1`. It is designed for
verified task replay, not as a general replacement for ZIP or tar.
