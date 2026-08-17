## Reproduction

At commit `56aa18ac041398afa037cec0da3cf5cae2553e07`, the template has a committed
`Cargo.lock` and `actions/dependency-review-action@v5`, but no `cargo audit` (or
equivalent RustSec) gate. Dependency review catches a vulnerable dependency
when a pull request introduces it, but not an advisory published later for the
unchanged lock.

```bash
cargo install cargo-audit --locked
cargo audit --file Cargo.lock
```

The audit is therefore a manual check rather than a required property of the
current default branch.

## Workaround

Run `cargo audit --file Cargo.lock` locally and on a schedule.

## Suggested code fix

Pin `cargo-audit`, add a required lock audit on pull requests and pushes, add a
scheduled run for newly published advisories, and add a workflow regression
that proves the committed `Cargo.lock` is covered. Cache the advisory database
or the audit binary if runtime becomes significant.

