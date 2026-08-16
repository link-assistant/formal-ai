# github/codeql#19982 — measured result of the sysroot pin

Posted as
<https://github.com/github/codeql/issues/19982#issuecomment-5309264165>, a
follow-up to `codeql-rust-macro-expansion-data-point.md`. The counts come from
`ci-logs/main-head-1858b338/run-31937348308.log` (baseline) and
`ci-logs/c71de5a40a7e396a99db8f18e71cbb056960c1d8/run-31967180539-security-codeql-sysroot-pinned.log`
(pinned).

---

## Comment body

Follow-up with measured numbers, since my earlier comment quoted the workaround
before I had run it myself.

Same repository, same CodeQL CLI 2.26.3, `build-mode: none`, only difference
being the two environment variables set from a pinned 1.94.0 toolchain with
`rust-src`:

| Diagnostic | Ambient toolchain (rustc 1.97.1) | Pinned sysroot (1.94.0) |
| --- | ---: | ---: |
| `macro expansion failed` | 20,725 | **0** |
| `proc-macro not yet built` | 0 | 355 |
| `` `OUT_DIR` not set `` | 0 | 3 |

The configuration dump changes from `sysroot: None, sysroot_src: None` to
`sysroot: Some(...), sysroot_src: Some(...)`, and every one of the 20,725
`macro expansion failed` warnings across 1,023 files disappears. So the
workaround is confirmed end to end at repository scale, not just in principle.

Two things worth noting for anyone applying it:

1. **Both variables really are required.** With only
   `CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT_SRC` set, rust-analyzer takes its
   `discover_with_src_override` path, keeps the discovered (ambient) binary
   sysroot, and the failures remain.

2. **The fix uncovers a second, much smaller class that the first one was
   masking.** With the sysroot resolved, extraction gets far enough to report
   `proc-macro not yet built` (355 occurrences — `#[derive(...)]` on
   `serde`-heavy types) because `proc_macro_server` is still `None`. That is
   two orders of magnitude smaller than what it replaced, but it is the same
   shape of silent coverage loss, and I could not find a documented way to
   point the extractor at a proc-macro server from the action. If there is one,
   it would be worth documenting alongside the sysroot options.

Full logs for both runs, and the counting commands, are in
`dev/log/issues/1017/pulls/1018/` of
[link-assistant/formal-ai#1018](https://github.com/link-assistant/formal-ai/pull/1018).
