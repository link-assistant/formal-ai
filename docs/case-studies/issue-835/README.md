# Issue 835: multi-jurisdiction file legal-risk assessment

Issue [#835](https://github.com/link-assistant/formal-ai/issues/835) correctly
rejects a blanket “legal worldwide” database and decomposes the question into
national-security/location restrictions, forbidden-content controls, and
copyright/IP evidence. Pull request
[#900](https://github.com/link-assistant/formal-ai/pull/900) implements that
decomposition as an evidence-oriented file inspection pipeline.

The exact acceptance leaves are in [`requirements.md`](requirements.md), the
implementation sequence and safety boundaries are in
[`solution-plan.md`](solution-plan.md), dated primary-source research is in
[`raw-data/online-research.md`](raw-data/online-research.md), and Formal AI's
GitHub collector preserves issue, PR, review, diff, and run data under
[`raw-data/github/`](raw-data/github/).

## Result

The public `check_file_legality` and `check_file_legality_with` functions read
the actual file and return one structured assessment per legal category and
jurisdiction. The `formal-ai file-legality` command exposes the same schema as
JSON. Reports retain:

- media family, detected media type, byte size, and a streaming SHA-256 during
  ordinary processing;
- bounded Exif author/copyright/camera/capture-time/GPS fields, each linked to
  its embedded tag;
- detector observation IDs, confidence, provider/source URI, jurisdiction, and
  restriction codes;
- policy ID, version, source URI, trigger codes, and required review action;
- completed, failed, or fail-closed-skipped provider adapter runs; and
- explicit `no_global_verdict`, `not_legal_advice`, and unverified-provider
  limitations.

The global verdict is structurally limited to `not_provided`. An absent
detector result stays `unknown`; a negative result is only
`no_risk_signal_detected`, never legal clearance.

## Safety boundary

A confirmed child-safety match is not inferred locally and cannot be created
from a repository dataset. It must arrive as a receipt from an authorized
provider integration. The pipeline then reads only the header needed for media
classification, suppresses hash and embedded-metadata derivatives, skips every
ordinary provider adapter, returns safe provider case/report references, and
requires refusal plus provider escalation. No prohibited fixtures, provider
hash database, or pseudo-PhotoDNA implementation is present.

## Provider and policy extension

Object/scene classification, symbol classification, reverse-image search, and
license lookup differ in authorization and availability. The
`LegalityEvidenceProvider` trait is the explicit integration boundary. Each
adapter declares its categories; Formal AI overwrites the observation's
provider identity with the registered adapter ID, ignores observations outside
the declaration, and records timeout/failure kinds without blocking other
categories. Arbitrary serializable policy packs then map those observations to
jurisdiction-specific actions.

The synthetic [`examples/file-legality/`](../../../examples/file-legality/)
sidecar demonstrates the schema without presenting example evidence as law.

## TDD and verification

Commit `9470916b` preserves the failing public contract before implementation:
the compiler reported that `formal_ai::file_legality` did not exist. The
focused suite now covers all three categories, two-jurisdiction independence,
versioned policies, evidence provenance, Exif/GPS extraction from a generated
TIFF, five media families, independent provider failure, and the confirmed-hash
safety boundary. A separate executable test drives the JSON CLI and verifies
that neither matched content nor its local hash appears in output.

Reproduce the focused checks with:

```bash
cargo test --test unit issue_835_file_legality
cargo test --test issue_835_cli
```

## Formal AI and Agent CLI evidence

The five reviewed contribution leaves are scope boundary, category
independence, jurisdiction/policy provenance, metadata/generalization, and
provider safety. Formal AI served the real Agent CLI and authored the fifth
leaf byte-for-byte in session `ses_03d2a3c95ffe1gfVxnh24MtxFi`:

> Confirmed child-safety hash matches must come from an authorized provider receipt, suppress local derivatives, stop ordinary detector execution, and escalate through the provider's reporting channel.

That sentence is the exact
[`agent-authored-provider-safety-boundary.md`](agent-cli-evidence/agent-authored-provider-safety-boundary.md)
artifact. The [`agent-cli-evidence/`](agent-cli-evidence/) directory retains the
raw and normalized stream, server trace, classified stderr, task, session ID,
and resulting workspace status. Reproduce it with:

```bash
cargo build --bin formal-ai
experiments/issue_835_agent_cli.sh
```

The paired trailers appear only on the isolated evidence commit. Manually
authored implementation and documentation commits do not claim self-authorship.
