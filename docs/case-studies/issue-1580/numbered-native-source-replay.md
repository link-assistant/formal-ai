# Native numbered source requirements

The [CI/CD Pipeline](https://github.com/link-assistant/formal-ai/actions/runs/45383970333)
on `b92691d3294b059d242a1e734e7176455cc11b58` failed the existing
`nested_delivery::numbered_source_requirements_deliver_both_digested_edits_as_evidence`
case in the first coding shard. Full workflow log lines 76603–76626 show
the unchanged `WEB_SEARCH_PROVIDERS` array. The preceding JavaScript
replay completed all four edits from the captured depth-three request and
wrote both requested evidence files.

The native test artifact was produced from GitHub's PR merge commit
`c134a8a9ec6890d245862ec5f887a8c5cd873b0c`. The extraction probe read
284,671,289 of its 1,128,576,796 archive bytes and verified the compiled
test's SHA-256
`0ce201013777850eaf2ff51891a60d61c489f7d1756d6a1c1bf52f46f4ea3cbf`
against the producer receipt.

```bash
scratch=$(mktemp -d)
gh api repos/link-assistant/formal-ai/actions/artifacts/14915133676 > "$scratch/artifact.json"
python experiments/issue-1580/replay-hosted-coding-contract.py \
  "$scratch/artifact.json" c134a8a9ec6890d245862ec5f887a8c5cd873b0c \
  "$scratch/coding_contracts"
(
  ulimit -s 16384
  "$scratch/coding_contracts" --exact \
    nested_delivery::numbered_source_requirements_deliver_both_digested_edits_as_evidence \
    --nocapture --test-threads=1
)
```

The hosted binary reproduced the failing assertion in 2.6 seconds. An
LLDB breakpoint showed the strict member editor receiving the complete
54-byte fixture. The matcher contained a literal `0x08` byte inside a Rust
raw string. That byte required a backspace preceding `const` or `static`.
The corrected word boundary matches the actual declaration and preserves
the rejection of a different or nonunique declaration. The existing opt-in
planner trace now reports an unbound declaration without including the
observed source content.

```bash
scratch=$(mktemp -d)
python experiments/issue-1580/probe-native-declaration-match.py \
  b92691d3294b059d242a1e734e7176455cc11b58 "$scratch/matcher"
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo run --locked -j 1 \
  --manifest-path "$scratch/matcher/Cargo.toml"
```

The isolated crate uses the repository's locked regex engine and the two
source revisions of the matcher. The published matcher rejects the
fixture, and the corrected one selects its initializer. The static,
token-boundary, wrong-name and duplicate-declaration checks pass. The
full native delivery test still requires a passing hosted replay on the
corrected head. These supplied changes earn zero independent Formal AI
authorship credit.
