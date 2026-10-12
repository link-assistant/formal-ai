# Issue #1175 Case Study: One Surface Word Routed the Whole Sentence

Issue [#1175](https://github.com/link-assistant/formal-ai/issues/1175) (E140,
part of the #1183 umbrella). Verified on `main` at `d209aac64`; fixed on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

Three prompts, three wrong frames, one shared cause — a single word matched a
recognition surface, and the detector never asked what that word was *doing*
in the sentence:

1. `Find the bug: def average(xs): return sum(xs) / len(xs) - 1` — a code
   review task — was answered:
   > It looks like you want to run a terminal command: `Find the bug: def
   > average(xs): return sum(xs) / len(xs) - 1`.
   >
   > Running shell commands requires Agent mode. …
   The whole sentence was echoed back as the "command".

2. `Make a 3-day itinerary for a first visit to Rome.` — a planning request —
   got the same terminal-command answer, with the itinerary request as the
   "command".

3. `Write a regular expression that matches a US ZIP code with an optional
   4-digit extension.` — a regex request — became a software-project plan:
   > Implementation plan pending approval for a extension targeting the
   > requested environment.
   >
   > Formalized meaning: … Reasoning steps: … Requirement model: … Subtasks: …
   a full multi-section implementation plan whose formalized meaning carried
   `artifact "extension"`. There `extension` modifies *ZIP code*; the head of
   what is written is "regular expression".

## Root cause

1. **The leading-token path.** `leading_shell_command`
   (`rust/src/solver_terminal.rs`) fired whenever a prompt's first word was in
   the seed's `shell_tokens` — and that list necessarily contains ordinary
   English words (`find`, `make`, `file`, `which`, `head`, `tail`, `touch`,
   `kill`, `export`, `cat`): they are command names. "Find the bug…" and
   "Make a 3-day itinerary…" start with two of them, so the prompt itself was
   returned as the command and `agent_suggestion` claimed the request.

2. **The any-position artifact scan.** `from_prompt`
   (`rust/src/solver_handlers/software_project.rs`) scanned the whole prompt
   for the first `software_artifact_kind` surface. "…a US ZIP code with an
   optional 4-digit **extension**" contains one, so the request was formalised
   as building an *extension*, whatever the writing verb's object actually
   was.

3. **The vocabulary was right; the syntax was missing.** Both lexicons are
   correct — `find` and `make` *are* shell tokens, `extension` *is* an
   artifact kind. What was missing is the question of role: a shell token in
   *command position* with *argument-shaped words* after it, and an artifact
   noun as the *head* of the authoring verb's object phrase.

## The change

| File | Change |
| --- | --- |
| `rust/src/solver_terminal.rs` | The leading-token path now requires the remainder to parse as command arguments. `parses_as_command_arguments` rejects a question mark at the end (ASCII or full-width), a colon followed by a space outside quotes, a bare token ending in sentence punctuation (except a token that is entirely dots, because `.` and `..` are paths), and a bare function word from the closed `NATURAL_LANGUAGE_MARKER_WORDS` list; `mask_quoted_spans` blanks quoted spans first, so `git commit -m 'fix the parser bug'` keeps working. The backtick+verb and verb+phrase trigger paths are unchanged. |
| `js/worker/formal_ai_worker_09.js` | The JavaScript twin: `NATURAL_LANGUAGE_MARKER_WORDS`, `maskQuotedSpans`, `parsesAsCommandArguments`, and the same gate closing `leadingShellCommand`. |
| `rust/src/solver_handlers/software_project.rs` | The artifact kind now comes only from the head of the authoring verb's object phrase. `object_phrase_artifact` iterates the action-verb matches left to right (multi-verb requests like "design and build a dashboard" work), skips benefactives and determiners (`skip_object_phrase_lead`: "write **me a** web app", "write **for me** an extension"), splits the object phrase at function-word boundaries and punctuation (`object_phrase_segments` with `boundary_cut`), and head-matches the first segment against the artifact table (`match_artifact_head`: longest surface, word-boundary-checked). A verb-final (subject-object-verb) request — nothing but boundaries after the verb — falls back to the last pre-verb segment, bounded left by sentence punctuation. `software_project_claims` and `from_prompt` both route through it; the old any-position `match_artifact`/`scan_match` are deleted. |
| `rust/tests/unit/issue_1175_routing.rs` | New acceptance suite (below). |

## Why real commands and real requests still pass

- `find . -name '*.log' -size +10M` — flags and paths; `.` is an all-dots
  token, `*.log` sits inside quotes and is masked, `-size`/`+10M` are
  argument-shaped.
- `git commit -m 'fix the parser bug'` — the prose is quoted data; the marker
  words inside the quotes are blanked before the checks run.
- `ls ~`, `make test`, `head -n 5 main.rs`, `export FOO=bar`,
  `touch newfile.txt` — bare argument-shaped words; an empty remainder
  vacuously qualifies.
- `Write a browser extension that blocks ads` — "browser extension" heads the
  object phrase; the relative clause is cut at the boundary word `that`.
- `Создай расширение для браузера, …` (ru) — the object head «расширение»
  ends the segment before the ` для ` boundary.
- `एक ब्राउज़र एक्सटेंशन बनाओ` (hi) — verb-final; the pre-verb fallback takes
  the last segment, which ends with the artifact surface.
- `创建一个浏览器扩展` (zh) — no spaces; CJK boundary characters cut inside a
  token, and the artifact surface ends the resulting segment.
- `Scaffold a Node.js service` — a dot *inside* a token does not cut a
  segment: ASCII punctuation only cuts at a token's first or last character.

## Before and after

### The code-review probe

Prompt: `Find the bug: def average(xs): return sum(xs) / len(xs) - 1`

- **Before**: `agent_suggestion`, the whole sentence echoed as the command.
- **After**: not claimed by the terminal frame — the remainder introduces the
  task after a colon and carries the marker word `the`. The prompt proceeds
  through the routing chain.

### The itinerary probe

Prompt: `Make a 3-day itinerary for a first visit to Rome.`

- **Before**: `agent_suggestion`, same echo.
- **After**: not claimed as a terminal command (marker words `a`, `for`, `to`)
  and not claimed as a software project; the prompt proceeds through the
  routing chain.

### The regex probe

Prompt: `Write a regular expression that matches a US ZIP code with an
optional 4-digit extension.`

- **Before**: `software_project_plan` for an `extension`.
- **After**: not claimed by the software-project frame — the head of what is
  written is "regular expression"; the incidental "extension" modifies
  "ZIP code". The fronted variant ("For a US ZIP code with an optional
  4-digit extension, write a regular expression.") is refused by the same
  rule: the verb is not sentence-final, so the pre-verb fallback cannot
  rescue the incidental noun.

## Tests

`rust/tests/unit/issue_1175_routing.rs`, all hermetic (no network; every case
goes through the public `FormalAiEngine.answer`):

1. `reported_probes_stop_misrouting` — the three reported prompts.
2. `natural_language_starting_with_a_shell_word_is_not_a_command` — five prose
   prompts starting with shell-token words (`Which search engine do you prefer
   and why?`, `Head of the department asked me to schedule a review.`,
   `Touch base with the team before Friday.`, `Cat owners know that cats sleep
   most of the day.`, `File a complaint about the noise, please.`).
3. `argument_shaped_prompts_after_a_shell_token_stay_terminal_commands` — the
   seven command shapes above, asserting both the `agent_suggestion` intent
   and that the answer names the command.
4. `incidental_artifact_words_do_not_claim_the_software_project_frame` — the
   fronted-PP variant of the regex probe, a study plan "and their extensions
   into graphs", a birdhouse build, a "dashboard launch date" flyer, and the
   Russian regex probe («Напиши регулярное выражение … с необязательным
   4-значным расширением.»).
5. `artifact_headed_requests_still_claim_the_software_project_frame` — en, ru,
   hi (subject-object-verb), zh (unspaced).

Every probe was verified to occur in no file under `data/seed/` (the held-out
rule — containing a seed surface *word* is fine; the probes are sentences,
not cue phrases), and no seed shell-intent cue matches any negative probe, so
the semantic shell fallback cannot re-claim what the guard released. Command:
`RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1175_routing`.

## Honest boundaries

- The JavaScript software-project twin (`formalizeSoftwareProjectRequest` in
  `js/worker/formal_ai_worker_11.js`, with `detectSoftwareArtifact` in
  `formal_ai_worker_10.js`) still scans for an artifact surface anywhere in
  the prompt; it was outside this change's file ownership and needs the same
  object-phrase guard.
- The `ts/` tree is generated from `js/` by
  `formal-ai translate --from js --to ts --write` and byte-for-byte gated in
  CI; regenerating it needs a release build and is left to the pull-request
  integrator after all JavaScript edits of this batch merge.
- Gerund authoring verbs ("solve this by *building* a web app") are not
  lexicalised as `software_authoring_action` surfaces — a seed-data gap, not
  a guard gap; such a request is not claimed either way.
- `software_project_claims` receives the normalised prompt (punctuation
  stripped, token boundaries only); the richer boundary analysis runs in
  `from_prompt` on the lowercased raw prompt. The claims gate is the coarser
  of the two by design — the solver chain calls the handler regardless.
