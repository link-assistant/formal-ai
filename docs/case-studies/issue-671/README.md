# Issue #671: generated real-client contracts and auto-learning

Issue #647 asked for client compatibility but accepted three integrations
without direct execution. Issue #671 is the follow-up: drive every supported
client for real, make that coverage repeatable, and learn reusable constraints
from what the clients actually do.

## Result

The matrix has one source of truth:
`data/seed/client-integrations.lino`. Each integration now includes a
verification contract. `formal-ai clients --format json` exposes it;
`plan_matrix.sh` derives the CI legs; and `run_leg.sh` consumes the same
contract without branching on client identity.

The contracts distinguish four real surfaces:

- prompt CLIs, exercised headlessly and through a PTY;
- application servers, launched and probed for readiness;
- GUI applications, launched under a display with configuration proof;
- MCP tool-server integrations, verified through JSON-RPC and their vendor-auth
  boundary.

This keeps adding a client from meaning “copy another workflow row and infer
the rest from a similar adapter.” A seed entry needs a pin and a complete,
machine-readable verification contract before the coverage test passes.

## Learning from successful sessions

Each successful prompt leg records two observations of `read_file` using
different wording: one headless request and one interactive request. The
learner accepts a client/capability group only when at least two normalized
wordings agree. It then:

1. compares the observed delivery mode (`tool_call` or `in_band`) with the
   seeded contract;
2. intersects invoked tool names across every observation in the group;
3. emits only unseeded stable behavior as a proposed amendment;
4. attaches the original transcript paths; and
5. leaves every proposal `awaiting_human_review`.

The committed [observations](agent-cli-contract-learning/observations.jsonl)
contain 16 real-session facts: two wordings for each of eight prompt clients.
The deterministic
[learning report](agent-cli-contract-learning/client-contract-learning-report.lino)
confirms all eight delivery contracts and proposes seven repeatedly observed
response-tool requirements. Aider supplies file bytes in-band, so it correctly
has no tool-name proposal.

## Formal AI through the real Agent CLI

The maintainer requested that Formal AI execute this task through Agent CLI,
not merely produce a report directly. The reproducible driver is:

The recorded Formal AI session id is
`issue-671-agent-cli-contract-learning-20260726`; it is also carried by the
authored commit's `Formal-AI-Session` trailer.

```bash
cargo build --release --bin formal-ai
experiments/agent_cli_e2e/run_issue_671_contract_learning.sh
```

It starts the release server in Agent mode, launches the installed Agent CLI
against it, and asks Formal AI to execute:

```bash
formal-ai clients learn observations.jsonl
```

The task requires the exact standard output to be written into a workspace
file. The first real run found a separate general-planner defect: the planner
wrote the referential phrase `its exact stdout` as literal content and never
ran the command. The fixed planner recognizes a seed-defined command-output
frame, writes its formal plan first, executes the quoted command with output
redirection, reads the generated file back, and only then completes.

The successful evidence set contains:

- the [formal plan](agent-cli-contract-learning/general-change-plan.lino);
- the [Agent-authored report](agent-cli-contract-learning/agent-authored-client-contract-learning-report.lino),
  byte-identical to the direct deterministic report;
- the [structured Agent CLI stream](agent-cli-contract-learning/agent-stream.jsonl);
- the [Agent CLI stderr classification input](agent-cli-contract-learning/agent-stderr.log);
- the [Formal AI request trace](agent-cli-contract-learning/formal-ai.log).

The driver fails unless the output is byte-identical, contains the `agent`
finding, and retains `decision "awaiting_human_review"`. Failed workspaces are
preserved for diagnosis; successful ones are removed.

## Reproducing the deterministic learner without Agent CLI

```bash
target/release/formal-ai clients learn \
  docs/case-studies/issue-671/agent-cli-contract-learning/observations.jsonl \
  > /tmp/client-contract-learning.lino
cmp \
  docs/case-studies/issue-671/agent-cli-contract-learning/client-contract-learning-report.lino \
  /tmp/client-contract-learning.lino
```

The unit suite also rebuilds this report from the observations, checks every
evidence path, rejects one-wording “learning,” and verifies that no matrix
script selects behavior by client identity.
