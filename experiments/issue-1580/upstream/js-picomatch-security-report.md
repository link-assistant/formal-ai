The root lock and universal-app lock currently contain affected picomatch releases. The security workflow audits committed npm locks; its policy is correctly rejecting the newly reviewed advisory.

Current source checked at 74de898dd6ce548af5bcb9359e1dae75fd4ebd75 on 2026-11-17T18:31:28.384369+00:00. Each audited lock exactly matches its current default-branch Git blob.

- [package-lock.json](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/blob/74de898dd6ce548af5bcb9359e1dae75fd4ebd75/package-lock.json): picomatch 4.0.7, audit exits 1, 13 high-severity affected dependency nodes. Blob 81c8b9f06dd688e9333ca3f6a76cca642fa880ca.

```sh
cd .
npm audit --package-lock-only --audit-level=moderate --json
```

- [examples/universal-app/package-lock.json](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/blob/74de898dd6ce548af5bcb9359e1dae75fd4ebd75/examples/universal-app/package-lock.json): picomatch 4.0.4, audit exits 1, 10 high-severity affected dependency nodes. Blob 1db5b1bbdf49f62b8feeb0d1a54853caa294c0d9.

```sh
cd examples/universal-app
npm audit --package-lock-only --audit-level=moderate --json
```

All affected-node chains in these responses lead to [GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf) / CVE-2026-112366. Reviewed on 17 November, it affects picomatch through 4.0.7; no patched release is listed. Registry and GitHub release metadata still identify 4.0.7 as latest. The npm fixAvailable suggestions downgrade consumers; they do not identify a patched picomatch release.

The published 4.0.7 source reproduces the single-branch guard gap with +(*) and +(*a). A finite 42-byte hostile subject fails to complete within 1.25 seconds in isolated Node children (64 MiB heap / 256 KiB stack); each process is killed. Exact output, source hashes, audit responses, a local candidate backport and normal micromatch/tinyglobby/fdir controls are retained in the associated investigation.

Retain complete development-dependency auditing and the existing severity policy. Adopt a reviewed upstream security release when available, or an explicitly maintained backport with bounded hostile-pattern tests, source/license provenance and consumer compatibility checks. Regenerate affected locks and verify the real audit gate. Omitting dev dependencies or raising the severity threshold would hide this finding.

This is a reproduced dependency-gate failure, not an upstream repair or proof that arbitrary untrusted glob patterns are reachable in the runtime application.

Evidence: [template lock audits](https://github.com/link-assistant/formal-ai/tree/83882c0ec7c26b28dc8ac274fada2b39d9dc3a67/experiments/issue-1580/evidence/checks/picomatch-template-locks), [bounded parser reproduction and backport](https://github.com/link-assistant/formal-ai/tree/83882c0ec7c26b28dc8ac274fada2b39d9dc3a67/experiments/issue-1580/evidence/checks/picomatch-single-branch)
Related investigation: https://github.com/link-assistant/formal-ai/pull/1581
