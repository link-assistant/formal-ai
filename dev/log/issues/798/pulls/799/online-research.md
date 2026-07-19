# Online research

Primary sources consulted on 2026-07-19:

- Apple, Code Signing Guide — nested code must be signed inside-out and code in
  nonstandard locations is treated as a resource:
  https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html
- Apple TN2206 — framework structure and `unsealed contents` validation:
  https://developer.apple.com/library/archive/technotes/tn2206/_index.html
- electron/osx-sign source and API — `ignore` accepts strings/functions and is
  applied while walking `Contents`:
  https://github.com/electron/osx-sign
- VS Code extension bundling — Microsoft recommends esbuild and excluding
  `node_modules` from the VSIX:
  https://code.visualstudio.com/api/working-with-extensions/bundling-extension
- Cargo configuration — `RUSTFLAGS` is Cargo configuration for compiler
  invocations Cargo launches:
  https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags
- GitHub Actions secure use — a full commit SHA is the only immutable action
  reference (recorded as a future supply-chain hardening item because the three
  requested templates themselves use moving majors):
  https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions
- npm clean installs — `npm ci` requires package and lock files to agree:
  https://docs.npmjs.com/cli/v11/commands/npm-ci

No upstream defect was filed for the signing failure: electron/osx-sign already
provides the required ignore facility, and the bug was this repository's choice
to feed an opaque browser runtime into its recursive application-code signer.

