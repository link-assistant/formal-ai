# Timeline, requirements, and root-cause analysis

## Timeline (UTC)

1. 2026-07-18 19:47:37 — issue 776 was filed automatically from an agentic session after the Russian source-first prompt fell through to `unknown`.
2. 2026-07-18 19:47:50–19:47:53 — GitHub applied the `bug` label and Bug issue type.
3. 2026-07-18 19:49:29 — the reporter clarified that request syntax must be widened, translation must use source → meta language → target, and translation quality should use reasoned internet knowledge.
4. 2026-07-19 05:10:44 — the prepared branch received its `.gitkeep` bootstrap commit.
5. 2026-07-19 05:10:53 — draft PR 794 was created with a placeholder description.
6. 2026-07-19 05:10:56–05:12:13 — Actions run 29674453764 completed. Detect Changes and Version Modification Check passed; substantive jobs were skipped because no relevant files had changed.
7. 2026-07-19 — local CLI reproduction returned the exact `unknown` response recorded in the issue.
8. 2026-07-19 — the first browser regression reached a separate main-thread defect: `система` was misread as a theme command and returned `Done. Theme is now auto.` before the worker ran.

## Complete requirements

| ID | Requirement | Evidence/acceptance |
|---|---|---|
| R776-1 | Recognize the reported source-first command, where the source precedes `translate to <target>`. | Exact CLI, Rust integration, and browser E2E cases return translation rather than `unknown`. |
| R776-2 | Widen request recognition generally, not by matching the reported sentence in routing code. | Word-order frames live in the multilingual meaning seed; Rust and JS project `Slot::Suffix`/`suffix` forms from the role. |
| R776-3 | Extract only the source, excluding dash, command, and target. | Focused parser regression asserts the exact extracted Russian proposition. |
| R776-4 | Formalize source language → one language-neutral meaning → target language. No direct language-pair implementation. | A single compositional meaning owns en/ru/hi/zh surfaces; every directed pair and reverse leg has the same meaning id. |
| R776-5 | Translate the reported proposition idiomatically and preserve its claim and source formatting. | Expected English is `any formal system is either incomplete or inconsistent` because the source is lowercase and has no terminal punctuation. |
| R776-6 | Use researched domain knowledge without silently changing the user's sentence. | Research distinguishes literal translation from the stricter Gödel theorem qualifiers; the translated surface preserves the input and is classified as a compositional concept. |
| R776-7 | Keep native Rust and browser-worker behavior in parity. | Mirrored routing, suffix extraction, punctuation cleanup, semantic lookup, integration test, and Playwright test. |
| R776-8 | Cover every supported language and round-trip quality. | en, ru, hi, and zh proposition surfaces are tested across all 12 directed cross-language pairs and reverse legs. |
| R776-9 | Preserve diagnostics and avoid fabricated output for unknown phrases. | Existing opt-in `FORMAL_AI_TRANSLATION_DEBUG=1` remains default-off; existing explicit gap behavior remains unchanged. |
| R776-10 | Ensure ordinary translated prose is not intercepted as an interface preference command. | The exact browser prompt reaches the worker; the existing `Switch to dark theme` behavior remains covered. |

## Root causes

### RC1 — position-specific routing

`try_translation` in Rust and `tryTranslation` in the worker treated English and Russian as obligatorily clause-initial. They used `starts_with`/`startsWith` for translation-action forms. The issue's English action is postpositive, so the handler returned `None` before target detection, source inference, or translation could run.

### RC2 — position-specific source extraction

The unquoted extractor projected only circumfix frames such as `translate … to `. It had no strategy for seed `Slot::Suffix` forms, so it could not recover a source located before the command. The same omission existed in the JavaScript worker.

### RC3 — the proposition had no shared semantic node

Even after routing, Wiktionary/Wikidata's lexical pipeline cannot reliably translate an arbitrary seven-word proposition as a whole. The fallback only composes Russian words when every token is seeded, and naïve token substitution would not supply English copular syntax. The phrase therefore needed one proposition meaning with language-specific renderers, which is precisely the existing meta-language architecture used for fixed compositional phrases.

### RC4 — a second parser would have remained stale

`translation::formalization::parse_translation_object` separately assumed clause-initial English/Russian. Reusing the structural unquoted extractor there prevents response routing and meta-formalization from disagreeing.

### RC5 — long Unicode cache keys could panic

The multilingual round-trip regression exposed a latent cache defect: `sanitize_segment` truncated a UTF-8 `String` at byte 96 without first moving to a character boundary. A long Cyrillic/Devanagari/Han query could therefore panic before returning a translation gap. Truncation now moves backward to the nearest valid boundary, with a direct Unicode regression.

### RC6 — substring-based theme recognition intercepted Russian prose

The web app recognizes interface commands before dispatching a prompt to the worker. Its theme check used unrestricted substring matches: `тема` matched inside `система`, while `систем` then selected the `auto` value. The exact issue prompt therefore returned a theme confirmation in the browser even after the translation engine was corrected. Theme-object recognition now requires Unicode word boundaries (with a separate CJK form), retaining explicit commands such as `Switch to dark theme` without treating embedded morphemes as commands.

## Solution and alternatives

The selected solution adds source-first forms to the seed role, derives suffix extraction from slot metadata in both engines, gates action-anywhere routing on a recognized target plus successful structural extraction, reuses that extractor in formalization, adds one multilingual proposition meaning, and prevents the browser command layer from matching the Russian theme term inside ordinary words.

Rejected alternatives:

- Searching for `translate` anywhere without a structural match would create false positives for prose about translation.
- Hardcoding the Russian sentence in Rust/JavaScript would violate the seed-driven language boundary and duplicate pair-specific behavior.
- Word-for-word Russian composition cannot generate the required English copula or robustly preserve the proposition's semantic unit.
- Adding a general parser dependency for this one new slot order is unnecessary because `WordForm::Slot` already models suffix frames.
