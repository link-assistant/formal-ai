# Upstream report 1 - `scraper = "0.21"` pins the unmaintained `fxhash` into every downstream lockfile (RUSTSEC-2025-0057)

**Target:** `link-assistant/web-capture` (`rust/Cargo.toml`)

**Severity:** every consumer of `web-capture`'s `search` feature -- which
`runtime`, and therefore the default feature set, enables -- inherits a RustSec
`unmaintained` advisory that it cannot fix from its own manifest.

---

## Title

`rust/Cargo.toml` pins `scraper = "0.21"`, which resolves `selectors 0.26` and
pulls in the unmaintained `fxhash`; `scraper 0.25` clears the advisory and
compiles with no source changes

## The advisory

[RUSTSEC-2025-0057](https://rustsec.org/advisories/RUSTSEC-2025-0057), filed
2025-09-05 against `fxhash`, `informational = "unmaintained"`, `patched = []`:

> # fxhash - no longer maintained
>
> The fxhash crate is no longer maintained.
>
> The repository is stale and owner is no longer active on GitHub.
>
> Please take a look at [rustc-hash](https://github.com/rust-lang/rustc-hash) instead.

Because `patched` is empty, no version of `fxhash` clears it. The only way out
is for nothing in the graph to depend on `fxhash` at all.

## Reproducible example

Any crate that depends on `web-capture` with the `search` feature reproduces it.
The chain below is from `link-assistant/formal-ai` at commit `f971b8205`:

```console
$ cargo tree --locked --target all --all-features --edges all --invert fxhash@0.2.1
fxhash v0.2.1
└── fxhash feature "default"
    └── selectors v0.26.0
        └── selectors feature "default"
            └── scraper v0.21.0
                ├── scraper feature "default"
                │   └── web-capture v0.3.37
                │       └── web-capture feature "search"
                │           └── formal-ai v0.347.0
```

and the audit:

```console
$ cargo audit --file Cargo.lock
Crate:     fxhash
Version:   0.2.1
Warning:   unmaintained
Title:     fxhash - no longer maintained
Date:      2025-09-05
ID:        RUSTSEC-2025-0057
URL:       https://rustsec.org/advisories/RUSTSEC-2025-0057

warning: 2 allowed warnings found
```

A self-contained reproduction that needs nothing but a network connection:

```console
$ cargo new fxhash-repro && cd fxhash-repro
$ cargo add web-capture@0.3.37 --no-default-features --features search
$ cargo tree --invert fxhash
```

## Root cause

`rust/Cargo.toml:104`:

```toml
scraper = { version = "0.21", optional = true }
```

`selectors` replaced `fxhash` with `rustc-hash` in **0.32.0**. Which `selectors`
a given `scraper` resolves is fixed by `scraper`'s own requirement, and the
caret on a `0.x` requirement does not cross the minor boundary -- so `scraper`
must be at least **0.25** before the swap is reachable:

| `scraper` | requires `selectors` | resolves | hasher |
|---|---|---|---|
| 0.21.0 | `^0.26.0` | 0.26.x | `fxhash` |
| 0.22.0 | `^0.26.0` | 0.26.x | `fxhash` |
| 0.23.1 | `^0.26.0` | 0.26.x | `fxhash` |
| 0.24.0 | `^0.31.0` | 0.31.x | `fxhash` |
| **0.25.0** | `^0.33.0` | 0.33.x | **`rustc-hash`** |
| 0.26.0 | `^0.36.0` | 0.36.x | `rustc-hash` |
| 0.27.0 | `^0.38.0` | 0.38.x | `rustc-hash` |

(Read from the crates.io dependency API for each version; `selectors 0.31.0`
still lists `fxhash`, `selectors 0.32.0` lists `rustc-hash`.)

So `0.24` is *not* enough -- a bump has to reach `0.25`. `0.27.0`, published
2026-05-11, is the current release.

## Why a downstream consumer cannot fix it

`cargo update` cannot help: `scraper 0.21.0` is the newest release satisfying
`^0.21`, and the `selectors 0.26` it requires has no `fxhash`-free version. A
`[patch]` section would mean vendoring a fork of `scraper`. `cargo audit`'s
`[advisories] ignore` silences the finding without removing the code, which is
the failure mode an audit exists to prevent.

The fix has to be the one-line requirement bump in this repository's manifest.

## Suggested fix

`rust/Cargo.toml`:

```diff
-scraper = { version = "0.21", optional = true }
+scraper = { version = "0.27", optional = true }
```

`0.25` is the minimum that clears the advisory; `0.27` is current and clears it
with the same one-line change, so there is no reason to take the smaller step.

## Verification

Cloned `link-assistant/web-capture` at `89494a271d4d8eb1d0a3d8e8c40b4204c33f78c7`,
applied the bump, and built. **No source file needed changing.**

```console
$ sed -i 's|scraper = { version = "0.21"|scraper = { version = "0.25"|' rust/Cargo.toml
$ cargo check --no-default-features --features search
   Compiling selectors v0.33.0
    Checking rustc-hash v2.1.3
    Checking web-capture v0.3.37 (/tmp/wc/rust)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 27.58s

$ cargo tree --no-default-features --features search -i fxhash
error: package ID specification `fxhash` did not match any packages
```

Zero warnings, zero errors, and `fxhash` is gone from the graph. The same
check against the recommended `0.27` is equally clean:

```console
$ sed -i 's|scraper = { version = "0.25"|scraper = { version = "0.27"|' rust/Cargo.toml
$ cargo check --no-default-features --features search
    Checking scraper v0.27.0
    Checking web-capture v0.3.37 (/tmp/wc/rust)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.26s
```

and so is the full default feature set, which enables `runtime` and therefore
the whole crate:

```console
$ cargo check
    Checking browser-commander v0.9.1
    Checking web-capture v0.3.37 (/tmp/wc/rust)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 22s
```

This is unsurprising given the API surface actually used. Every `scraper` import
in the crate is from the part of the API that has not changed across these
releases:

```
rust/src/figures.rs:10    use scraper::{Html, Selector};
rust/src/gdocs.rs:37      use scraper::{node::Node, ElementRef, Html, Selector};
rust/src/latex.rs:11      use scraper::{ElementRef, Selector};
rust/src/markdown.rs:8    use scraper::{Html, Selector};
rust/src/metadata.rs:15   use scraper::{Html, Selector};
rust/src/search.rs:22     use scraper::{Html, Selector};
rust/src/search.rs:201    fn first_text(element: &scraper::ElementRef, selector: &Selector) -> String
```

`Html`, `Selector`, `ElementRef` and `node::Node` only -- none of the
`html5ever`/`selectors` re-exports whose types the intervening releases moved.

## Workaround for downstream consumers, until this lands

There is no way to remove the crate downstream, so the honest workaround is to
record the advisory as *blocked upstream* rather than as *unreachable*, and make
the record expire on its own. `link-assistant/formal-ai` does this in
`.cargo/audit.toml` with a machine-checked proof line, verified by
`scripts/check-rust-dependencies.sh`, which fails the build the moment
`cargo tree --invert` can no longer find the crate -- that is, the moment this
bump lands and the ignore becomes stale.

Consumers should also pass `--deny warnings`:

```console
$ cargo audit --file Cargo.lock
warning: 2 allowed warnings found       # exit status 0

$ cargo audit --file Cargo.lock --deny warnings
error: 2 denied warnings found!         # exit status 1
```

`cargo audit` classifies `unmaintained`, `unsound` and `yanked` findings as
warnings, and a warning does not move the exit status. Without `--deny warnings`
this advisory is invisible to CI even though it is printed on every run.

## Related

- `cbreeden/fxhash#20` -- the upstream "no longer maintained" issue the advisory
  cites.
- `rust-lang/rustc-hash` -- the replacement `selectors 0.32.0` adopted.
