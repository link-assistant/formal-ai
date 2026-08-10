# Issue #988 — stock Rust installation without OpenSSL

Issue: <https://github.com/link-assistant/formal-ai/issues/988>

## Root cause

Formal AI used `web-capture` 0.3.34 and `web-search` 0.3.1 with their default
features. Those releases selected the complete browser/server graphs. Two
independent paths then enabled native TLS:

```text
web-capture -> reqwest 0.12 -> native-tls -> openssl-sys
web-capture -> browser-commander -> fantoccini -> native-tls -> openssl-sys
```

The production integration only calls `web_capture::search`'s deterministic URL
builder/parser and `web_search::merger`. The newer published crates expose those
APIs without their browser, HTTP, CLI, or server dependencies. Formal AI now
selects `web-capture` 0.3.36 with only `search`, and `web-search` 0.5.0 without
its default `server` feature.

## Reproduction and regression

Before the fix, this focused test failed because `Cargo.lock` contained
`openssl`:

```text
cargo test --test unit \
  issue_988_stock_rust_install::default_dependency_lock_has_no_system_openssl_stack \
  -- --exact

the default dependency lock must not contain openssl
```

After the dependency boundary change, the test passes and
`cargo tree --locked -i openssl-sys` reports that the package is absent. The
`Stock Rust Install` workflow additionally installs the complete binary inside
an unmodified `rust:1.96-slim-bookworm` container, then checks `ldd` for
`libssl`/`libcrypto` and executes `formal-ai --version`.

| Requirement | Verification |
| --- | --- |
| Default lock excludes the system TLS stack | `default_dependency_lock_has_no_system_openssl_stack` |
| Minimal published component features stay selected | `manifests_select_only_transport_independent_web_features` |
| Stock Rust image installs without `apt-get` | `.github/workflows/stock-rust-install.yml` |
| Installed binary has no OpenSSL runtime link | the workflow's `ldd` assertion |
| Existing native component behavior remains | all seven `issue_896_component_boundaries` tests |

## Self-authorship

The real Agent CLI was driven against the release-mode Formal AI server to
author the changelog leaf. Reproduce it with:

```bash
cargo build --release --bin formal-ai
experiments/issue_988_self_authoring/run.sh
```

The run produced session `ses_0132073d7ffeHL6POmzfQ29hoH` in four chat/tool
rounds. The generated fragment is compared byte-for-byte with the committed
changelog file. Raw Agent CLI and Formal AI server logs are retained under
`self-hosting-authorship/`; the reviewed leaf accounting is in
`decomposition.lino`.
