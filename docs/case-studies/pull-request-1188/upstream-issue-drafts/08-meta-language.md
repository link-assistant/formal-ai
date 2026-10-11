<!-- repo: link-foundation/meta-language -->
<!-- title: Release self-translation (PR #196) to crates.io and npm, so consumers stop pinning a main commit -->

### Problem

PR #196 was merged on 2026-10-06 as `679a3b3c`. It brings self-translation (`selfTranslate` in `js/src/self-translation.js`, `self_translate` in `rust/src/self_translation.rs`, and `meta-language translate --to rust`). No release has been cut since then:

- crates.io: the latest release is `0.58.2` (2026-08-20). Neither `js/src/self-translation.js` nor `rust/src/self_translation.rs` exists at tag `v0.58.2`.
- npm: `npm view meta-language versions` lists only `0.46.0`. The package on `main` says `0.58.2`. So the npm package is twelve minor versions behind the crate, although #171 asked to keep them in lockstep.
- #199 lists a release as part of the remaining #195 scope. It also says `main` was red at merge: the JavaScript test `the scope gate reads the full committed ledger …` failed on `ddcde321`.

### Consumer need

link-assistant/formal-ai writes requirements in JavaScript first and gets Rust by translation through meta-language. Its js -> rust leg now runs `selfTranslate(source, 'JavaScript', 'Rust')` over 123 of its JavaScript modules ([link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188): `scripts/translate-js-rust.mjs`, `data/meta/js-rust-translation.lino`). At `679a3b3c` it translates 214 top-level items in 72 modules. The translated Rust compiles with rustc, and 72 constant and call assertions agree with the JavaScript.

There is no release to depend on, so the leg pins the main commit. CI checks out the repository at that SHA (`actions/checkout` with `ref:`) and runs `npm ci` in `js/`. A local run downloads the commit's tarball. A consumer cannot declare this in `package.json` or `Cargo.toml`: the npm package lives in `js/`, so a `github:` dependency spec does not resolve it. Every upstream fix also means a manual SHA bump.

### Request

1. Cut a release from `main` that includes PR #196: the crate and the npm package at the same version, each exporting `selfTranslate` / `self_translate`, `translateProgram` and the `translate` CLI command.
2. If #199's full scope is far off, cut an interim release once `main` is green. Self-translation is usable as it is: it carries what it cannot translate, so consumers can measure and ratchet.
3. Publish the npm package again: `meta-language@latest` should match the crate's version.

Acceptance: `npm install meta-language@<new>` and `meta-language = "<new>"` both expose self-translation. formal-ai then replaces its SHA pin with the version.

---
Found while wiring the js -> rust leg in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188) (`docs/case-studies/pull-request-1188/js-rust-translation.md`).
