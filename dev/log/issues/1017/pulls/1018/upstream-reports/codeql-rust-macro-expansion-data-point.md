# github/codeql#19982 — additional data point

Intended as a comment on the open issue
[`github/codeql#19982`](https://github.com/github/codeql/issues/19982)
("[Rust] macro expansion failed warnings 2"), which `geoffw0` reopened after
`PaulDance` showed the earlier rust-analyzer fix did not close it. It
cross-references [`github/codeql#22244`](https://github.com/github/codeql/issues/22244)
("Arguments to macro calls to `format` and `print` are not registered as
data-flow nodes"), which covers the analysis consequence rather than the
warning.

---

## Comment body

Another data point at repository scale, on **CodeQL CLI 2.26.3** (one patch
newer than the 2.26.2 measured above), from the default GitHub-hosted
`ubuntu-latest` image with **no `rust-toolchain.toml`** — so the extractor sees
whatever the runner ships, which for us was `rustc 1.97.1`.

One run of `github/codeql-action` with `build-mode: none` over a ~1,500-file
workspace produced:

```
20,725  macro expansion failed
 1,023  distinct source files affected
```

Broken down by tree:

| Tree | Diagnostics | Files |
| --- | ---: | ---: |
| `tests/` | 16,923 | 624 |
| `src/` | 3,016 | 285 |
| `examples/` | 499 | 103 |
| `scripts/` (rust-script) | 282 | 10 |
| `build.rs` | 5 | 1 |

The distribution by macro is the part I think is diagnostic. Every failing
macro is defined in `std`/`core` or in a dependency — **not one is defined in
the project being analysed**:

| Macro | Failures |
| --- | ---: |
| `assert` | 7,727 |
| `assert_eq` | 4,953 |
| `format` | 3,311 |
| `vec` | 953 |
| `$crate::format_args_nl` | 581 |
| `writeln` | 564 |
| `$crate::panic::panic_2021` | 549 |
| `matches` | 372 |
| `json` / `serde_json::json` | 494 |
| `env` | 316 |
| `include_str` | 256 |
| `write` | 203 |
| `assert_ne` | 156 |
| `$crate::__export::format_args` | 82 |
| `concat` | 78 |
| `include_bytes` | 18 |
| `cfg` | 16 |
| `unreachable` | 12 |
| others (`include`, `debug_assert`, `format_args`, `thread_local_inner`, …) | ≤ 11 each |

This matches @mario4tier's conclusion that the failure is a function of the
`std` version rather than of the project: we have no project-defined macro in
the list at all.

Two differences from the measurements already in this thread, in case they help
narrow it:

1. **The extractor had no sysroot at all, not a mismatched one.** The
   configuration dump at the start of extraction reads:

   ```
   INFO configuration: {
       ...
       sysroot: None,
       sysroot_src: None,
       rustc_src: None,
       ...
       proc_macro_server: None,
   }
   ```

   `rust-src` is not part of the `ubuntu-latest` image's preinstalled
   toolchain, so unlike the runs above there were no `std` sources on disk to
   fall back to.

2. **Zero failures inside `std` itself** — no `cfg_select`, no `pattern_type`,
   which the 1.97 row in @mario4tier's table shows 38 and 29 of. That is
   consistent with (1): `std` was never extracted, so it could not fail. The
   user-code column is what we reproduce, at three orders of magnitude more
   volume.

## Why this is not cosmetic

These are `WARN`-severity log lines and produce no GitHub annotation, so a run
with 20,725 of them still reports success. But a file whose macro calls do not
expand is recorded as *extracted with errors*, and the bodies behind those
macros are not available to the queries. In a codebase where `assert!` and
`format!` are the two most common macros, that is a large and completely
invisible hole in coverage — which is why I think this deserves more weight
than a warning-noise issue. `#22244` describes the same hole from the query
side.

## Minimal reproduction

@mario4tier's `cargo init --lib` repro reproduces it for us verbatim, with no
`rust-toolchain.toml` at all (so the runner's ambient stable is used):

```rust
pub fn f(n: usize) -> Vec<f64> {
    let v = vec![0.0f64; n];
    assert!(v.len() == n);
    let s = format!("{}", v.len());
    println!("{}", s);
    v
}
```

```yaml
- uses: github/codeql-action/init@v4
  with:
    languages: rust      # build-mode: none is the default for Rust
- uses: github/codeql-action/analyze@v4
```

## Workaround, confirmed on 2.26.3

@mario4tier's sysroot pin works here too, and both variables are genuinely
required — setting only `_SYSROOT_SRC` leaves the discovered binary sysroot in
place and the failures remain:

```yaml
    env:
      CODEQL_RUST_SYSROOT_TOOLCHAIN: '1.94.0'
    steps:
      - name: Pin the extractor sysroot to a std the CodeQL bundle can parse
        run: |
          rustup toolchain install "$CODEQL_RUST_SYSROOT_TOOLCHAIN" \
            --profile minimal --component rust-src
          sysroot="$(rustup run "$CODEQL_RUST_SYSROOT_TOOLCHAIN" rustc --print sysroot)"
          echo "CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT=${sysroot}" >> "$GITHUB_ENV"
          echo "CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT_SRC=${sysroot}/lib/rustlib/src/rust/library" >> "$GITHUB_ENV"
```

As already noted, this needs advanced setup; default setup cannot inject
environment variables, so any repository on default setup has no workaround at
all.

## Suggestions

1. **Make the extractor's `std` expectation explicit rather than implicit.**
   The bundle knows which rust-analyzer it vendors; it could compare the
   discovered `std` version against that and emit *one* diagnostic — "vendored
   rust-analyzer supports std ≤ X, found Y; macro expansion will be degraded" —
   instead of one warning per call site. That single line would have saved
   every reader of this thread the reverse-engineering, and it turns 20,725
   lines into one.

2. **Surface degraded extraction in the run's status, not only in the log.**
   A run where 1,023 of 1,496 indexed files were extracted with errors reports
   success today. Emitting a summary diagnostic (the extractor already has a
   `diagnostic_dir`) would make the coverage loss visible where people look.

3. **Ship the fallback the workaround performs manually.** If no sysroot
   override is set and the discovered `std` is newer than supported, the
   extractor could look for a supported toolchain via `rustup` before falling
   back to the unsupported one. That would fix default-setup repositories,
   which cannot apply the workaround at all.

## Reference

Full evidence, counts and the reproduction of the derivation:
`link-assistant/formal-ai` PR
[#1018](https://github.com/link-assistant/formal-ai/pull/1018),
`dev/log/issues/1017/pulls/1018/`.
