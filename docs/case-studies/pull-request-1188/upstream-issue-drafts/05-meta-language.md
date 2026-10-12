<!-- repo: link-foundation/meta-language -->
<!-- title: Ship tree-sitter-bash and tree-sitter-haskell as runtime grammars in the next crates.io release -->

### Problem

The latest crates.io release, `meta-language` 0.58.2, does not depend on `tree-sitter-bash` or `tree-sitter-haskell`, so a consumer cannot parse Bash or Haskell. `main` now pins `tree-sitter-bash = "=0.25.1"` and `tree-sitter-haskell = "=0.24.1"`, but neither is released yet. Recent commits on `main` also demote several grammars to development oracles only (go, java, regex, graphql, proto, make, solidity), so it is unclear whether these two will ship as runtime grammars.

### Consumer need

link-assistant/formal-ai rediscovers each catalog language's Hello World program from its official documentation page. It decomposes the page's code block into parts with the meta-language CST, with no stored answers. This is done for 14 languages in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188). Bash and Haskell are the two catalog languages left on stored snapshots, because 0.58.2 has no grammar for them.

### Request

In the release that finishes #199, ship `tree-sitter-bash` and `tree-sitter-haskell` as runtime grammars, not development oracles only, so `parse` works for `bash` and `haskell` from the published crate (Rust, and the npm package if it carries the same grammar set).

---
Found while retiring stored programs in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188).
