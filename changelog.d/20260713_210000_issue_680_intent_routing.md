---
bump: minor
---

### Added
- Tool-call emission in `formal-ai serve` is now **intent-based** rather than
  phrasing-gated (issue #680). When a client advertises a web-search, web-fetch, or
  write/edit tool, a request expressing that intent in *any* phrasing — across en, ru,
  hi, and zh — routes to the matching `tool_call` instead of a prose description. The
  routing holds over all three wire surfaces the target CLIs use (OpenAI Chat
  Completions, OpenAI Responses, and Gemini `generateContent`), and only fires when the
  matching capability tool is actually advertised, so a request that cannot be honoured
  still falls through to the prose answer.
- A file-creation intent that names a relative target file and literal content now
  routes to the advertised write tool in any phrasing/language. The write intent is
  recognised entirely from the seed lexicon (the new `file_write_*` roles in
  `data/seed/meanings-file-write.lino`) rather than from hardcoded English or Russian
  phrasings (CONTRIBUTING §2), and is probed before the file-read router so
  "create file X containing Y" is a *write*, not a read of X.
- A file-modification intent that names a target file plus an old→new replacement
  ("In greeting.txt, change hello to goodbye", "Replace foo with bar in notes.txt",
  «замени привет на пока в файле заметки.txt») now routes to the advertised edit tool,
  whatever the CLI calls it (`edit`, `replace`, `apply_patch`, `str_replace`). The
  new `Capability::Edit` recovers the `(target, old, new)` triple entirely from the
  seed lexicon (the new `file_edit_*` roles in `data/seed/meanings-file-edit.lino`),
  emits every common argument-key alias so one plan drives any CLI's edit tool, and is
  probed after the create-file write router and before the file-read router so an edit
  is never mistaken for a write or a read.
- A semantic shell request that never names the command — expressing an *intent* such as
  "Print the current working directory", "How much disk space is free?", or "What is my
  username?" — now routes to the advertised run tool carrying the concrete command
  (`pwd`, `df -h`, `whoami`) instead of a prose answer. The intent→command table,
  including multilingual cue phrases and per-intent argument recovery (`wc -l Cargo.toml`,
  `mkdir build`), lives in the new `data/seed/shell-intents.lino`, so coverage is retuned
  by editing seed data rather than the planner (CONTRIBUTING §2). It runs as a fallback
  after the named-command (#676) and directory-listing routers, so existing shell
  behaviour is unchanged, and only fires when a run/shell tool is advertised.

### Fixed
- The Russian navigation verb "загрузи" (load) is no longer misclassified as an
  `http_fetch`; it stays with `url_navigate`, while "скачай" (download the bytes)
  remains the fetch verb, so bare-domain navigation prompts resolve to an HTTPS link
  without fetch advice (issue #680).
- The general write router no longer mistakes a sentence-ending word for a target file:
  a token whose only dot is a terminal `.`/`!`/`?` ("… add the plural to томат.") is no
  longer treated as a dotted filename, so stored recipe requests are not hijacked
  (issue #680).
