Parent: #710

Audit verdict: still-broken.

Current evidence
- CI retains LCOV and can fail closed when a configured remote upload fails.
- No checked threshold or non-decreasing ratchet makes the double-tests-toward-100-percent requirement concrete.

Acceptance
- Publish human-readable and machine-readable coverage artifacts.
- Check a baseline threshold and reject decreases unless an explicit reviewed baseline update is included.
- Cover Rust and browser production paths or document separate honest denominators.
- Add regression tests for threshold enforcement and baseline updates.
