# Issue 534: Development disk usage

## Result

The 12 GiB workspace was dominated by recomputable Cargo output, not by a test
that forgot to delete one 12 GiB temporary directory. A full-history checkout
of commit `64242856` occupied 869 MiB before any command ran:

| Component | Size |
| --- | ---: |
| tracked checkout (sum of file sizes) | 528 MiB |
| `.git` | 337 MiB |
| complete working directory | 869 MiB |
| `target/` | absent |

One clean `cargo build --release --bin formal-ai` then created a 2.0 GiB
`target/release` tree (1.7 GiB dependencies, 247 MiB build-script output) and a
158 MiB stripped executable. The complete workspace became 2.8 GiB. That is
only one optimized profile and one binary: the ordinary development workload
also builds debug libraries, each integration-test binary, examples, Clippy,
coverage-instrumented objects, feature combinations, and incremental state.
The decisive reproduction was the documented
`cargo clippy --all-targets --all-features` command. It grew `target/` to
13 GiB. Of that, `target/debug/examples` consumed 7.0 GiB on disk: Cargo linked
122 example executables whose aggregate logical size was 13.86 GiB.
`target/debug/deps` added another 3.6 GiB. This reproduces and explains the
reported 12 GiB without relying on a leaked temporary directory.

After `cargo clean`, the replacement Clippy plus example-check commands
completed from scratch with a 1.1 GiB `target/`: dependencies occupied 836 MiB,
`target/debug/examples` occupied 256 KiB, and it contained zero executable
example binaries. This is a reduction of more than 90% for the reproduced
validation workload.

The complete replacement test command then passed 2,002 unit tests plus all
integration targets (two tests ignored by design) and left `target/` at
3.9 GiB. Real test harnesses must still be linked, but the peak stayed 70%
below the reproduced 13 GiB tree because examples were not linked as tests.

The repository itself is unusually large too. At this revision, `dev/log`
occupies 322 MiB and `docs` occupies 173 MiB. Removing those files in the
current revision would not make a full-history clone proportionally smaller,
because their Git objects remain reachable. This PR records the baseline rather
than rewriting shared history.

## Cleanup audit

All runtime uses of `std::env::temp_dir()`, `tempfile`, `mktemp -d`, and explicit
`target/` output were searched across `src`, `tests`, `scripts`, `examples`, and
`experiments`. Shell runs use an `EXIT` trap where they own a temporary
directory; Rust tests either use scoped temporary-directory guards or explicit
removal. The remaining textual hits are documentation/examples or paths under
Cargo's already-recomputable `target/`. No independent large leaked tree was
found.

The immediate recovery command remains:

```sh
cargo clean
```

It removes only recomputable Cargo artifacts. Source, Git history, case-study
evidence, and user data are not cleanup targets.

## Fix

- Development and test profiles retain debug assertions but disable full
  debuginfo and incremental state. Both are large; either can be enabled for a
  debugger session with Cargo's `CARGO_PROFILE_*` environment overrides.
- GitHub Actions no longer caches whole `target/` directories. Every Rust job
  that previously did so installs Mozilla's sccache action and caches downloaded
  Cargo sources separately. Compiler objects remain reusable without restoring
  stale branch-specific test binaries and incremental trees.
- Routine Clippy and test commands select library, binary, and test targets.
  Examples retain compile coverage through `cargo check --examples`, which
  type-checks them without linking 122 large standalone executables.
- `scripts/check-disk-usage-policy.rs` fails CI if either bounded local profile
  is removed, broad validation starts linking every example again, or a
  workflow starts caching `target/` again.
- The host/container-level follow-through is tracked by
  [hive-mind#2100](https://github.com/link-assistant/hive-mind/issues/2100):
  isolated Rust tasks should share a bounded sccache service rather than
  duplicate build trees.

## Self-coding evidence

The real external Agent CLI was connected to a local `formal-ai serve` process.
Formal AI planned and authored
[`self-coding/agent-authored-policy.lino`](self-coding/agent-authored-policy.lino)
from a natural-language request. The captured stream, server log, and replay
session are in [`self-coding/`](self-coding/). The authored policy is the
smallest of five named leaves:

1. measure the clean checkout and controlled build;
2. audit temporary-output ownership;
3. bound local Cargo profiles;
4. replace whole-target CI caches with sccache;
5. record the source-preserving cleanup policy.

Only leaf 5 is attributed to Formal AI (`1/5 = 20%`); the investigation and
implementation are not relabeled as self-authored.

## Reproduction

```sh
rust-script scripts/check-disk-usage-policy.rs
cargo test --test unit data_files
cargo fmt --all -- --check
cargo clippy --lib --bins --tests --all-features -- -D warnings
cargo check --examples --all-features
```

For an isolated size measurement, set `CARGO_TARGET_DIR` to a new temporary
directory, run the desired Cargo command, measure it with `du`, then remove that
explicit temporary directory.
