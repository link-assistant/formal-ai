# Issue 843: evidence honesty and replayable source research

Issue [#843](https://github.com/link-assistant/formal-ai/issues/843) reported
that an external-search trace claimed an `example.org` retrieval, Unix-epoch
timestamp, prompt-derived digest, and cache hit even though no request or cache
entry existed. The old trace looked like provenance but was fabricated.

The replacement makes one rule structural: source evidence can be emitted only
by an exact `SourceCapture`. Search rankings, option observations, probability
updates, statement audits, and learning proposals all sit downstream of that
capture boundary.

## Evidence inventory

Authenticated GitHub API snapshots are retained under [`raw-data/`](raw-data/):

- `issue-843.json` and all issue comments;
- PR 853 metadata, all conversation comments, all inline review comments, and
  all reviews.

The issue and review have no image attachments, so there is no screenshot to
download. This is a provenance and execution defect, not a visual defect.

Reproducible runtime evidence is kept beside this README:

- [`self-hosting-authorship/`](self-hosting-authorship/) contains the raw
  external Agent CLI transcript, Formal AI server trace, authored invariant,
  and five-leaf decomposition for the same issue task;
- [`self-hosting-evidence/`](self-hosting-evidence/) preserves the earlier
  whole-repository source-to-links projection of the branch;
- [`requirements.md`](requirements.md) maps every issue and review requirement
  to implementation and executable evidence.

## Root cause

The prior system treated a requested lookup as if it were a completed
retrieval. It created provenance fields from the prompt and appended
`source:http` and `cache_hit` immediately. Other components had useful pure
models—RRF, option networks, statement verification, and statement audit—but
no common executor capable of supplying captured bytes. That made planning
events indistinguishable from observations and left downstream models open to
fictional inputs.

The fix separates the state transitions:

```text
request
  -> cache lookup
     -> verified exact capture
     -> or explicit opt-in transport -> exact capture
     -> or policy/error diagnostic

exact capture
  -> provider rankings -> RRF
  -> captured result pages -> option observations
  -> explicit byte classification -> statement probability and audit evidence
  -> deterministic proposal -> human review before durable promotion
```

A request, cache miss, or failed transport never crosses the exact-capture
boundary and therefore cannot become evidence.

## Implementation

`CachedSourceClient` defaults to cache-only operation. A successful live
transport stores response bytes at
`source-cache/objects/<sha256>.body` and atomically publishes URL metadata
containing the actual retrieval time and digest. Replay reads the same object,
validates that the metadata digest is a lowercase SHA-256 before resolving the
object path, recomputes the digest from the bytes, rejects corruption, and
marks only that replay as a cache hit. Existing version-1 captures remain
readable.

The adjacent probability boundary was tightened as well.
`ProbabilitySourceProvenance` no longer exposes fields from which callers can
assemble an arbitrary URL/time/digest tuple; it can only be derived from a
`SourceCapture`. Definition fusion still discloses the Wikipedia URLs recorded
in its seed registry, but labels them `definition_merge:source_declared` rather
than pretending that it fetched them.

The production HTTP and contextual web-search handlers use this client.
Contextual live search requires a non-offline solver configuration and the
explicit `FORMAL_AI_LIVE_FETCH` process opt-in; otherwise the handler can replay
a capture or returns a truthful unavailable response. `web_search:provider`,
rank, and fusion events are written only after the DuckDuckGo response has
been captured and parsed. Descriptions of unexecuted work use `*_planned`
events throughout the touched handlers.

`execute_source_research` is the shared driver. It captures the provider
response, fuses the parsed rankings, then captures a bounded number of fused
pages. Per-page failures stay diagnostic and do not erase other successful
captures. `execute_option_research` parses only those pages before calling
`OptionNetwork::observe`. `execute_statement_research` searches each extracted
statement and admits a page only when its exact capture is explicitly
classified. The resulting audit evidence is constructed from the capture, so
its URL, timestamp, and digest cannot drift from the bytes that were examined.

Each execution can render a deterministic Links Notation learning proposal.
The proposal contains queries, provider observations, ranks, URLs, timestamps,
digests, and derived option-network state. It deliberately excludes incidental
live-versus-cache state, so offline replay proposes the same lesson. This is
auto-learning at the observation and proposal stages only: no durable seed or
learning ledger is modified without the repository's human review gate.

## Reproduction

The minimum deterministic fixture performs one provider capture and two result
page captures. The first pass verifies exact-byte SHA-256 values, RRF input,
option observation, statement classification, and audit conversion. A second
cache-only pass verifies identical rankings, option network, learning proposal,
bytes, and trace payload without another transport request.

Run the focused checks with:

```sh
cargo test --test unit issue_843 -- --nocapture
cargo test --test unit source_cache -- --nocapture
cargo test --test unit web_requests -- --nocapture
cargo test --test unit total_closure::seed_has_total_reference_closure -- --nocapture
```

Run the complete repository gates with:

```sh
cargo test --test unit -- --nocapture
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
sh src/web/wasm-worker/build.sh
GITHUB_BASE_REF=main npm run --prefix tests/e2e check:language-test-coverage
rust-script scripts/check-file-size.rs
rust-script scripts/check-hardcoded-language.rs
```

No test uses the live network. The fixture transport enters through the same
public capture client used by production, and cache replay uses the same code
path rather than a test-only shortcut.

## Same-task self-application

Formal AI served the `formal-ai` model to the external Agent CLI and completed
the evidence-honesty task in four chat rounds. Session
`ses_052d57d6affe1f2GYFodXhrqPl` planned a general change, used the client-owned
write tool to create `source-evidence-honesty-invariant.lino`, used the shell
tool to read it back, and then reported completion. The canonical
[`data/meta/source-evidence-honesty-invariant.lino`](../../../data/meta/source-evidence-honesty-invariant.lino)
is byte-for-byte equal to the generated artifact.

The reviewed decomposition contains five smallest leaves. Four implementation
leaves are manually authored; the invariant leaf is Agent-CLI-authored. That is
one of five leaves, or 20%. The raw transcript and server trace contain the
session ID and the write, verification, and final transitions. The replay
harness is [`experiments/issue_843_self_authoring/run.sh`](../../../experiments/issue_843_self_authoring/run.sh).

The earlier whole-repository projection is retained as complementary
self-model evidence, but it is not counted as same-task authorship. The new
session closes that distinction explicitly.

## Residual policy

Live sources can change or disappear, which is why CI and deterministic replay
operate on verified captures. A successful retrieval is an observation, not
automatically support for a statement: a classifier must still assign stance,
tier, and strength from the captured bytes. Failed or irrelevant pages remain
diagnostics. Durable promotion remains human-gated.
