# Issue #836 Case Study: The Reconsideration Warning

Issue [#836](https://github.com/link-assistant/formal-ai/issues/836).
Case study written on the `qa-reasoning-coding-bulk-fixes` branch.

## What this is

A user-protection feature, not a policing one: when Formal AI classifies an
incoming request as likely illegal, it warns clearly and early — what
appears illegal, which category fired, that jurisdictions vary, and that
the classification is not legal advice — and then leaves the decision to
the user. A narrow refuse set (child safety, credible serious violence) is
refused; everything else warns and informs.

## Requirements (from the issue body)

1. Warn clearly and early, stating what/why/where plus the not-legal-advice
   caveat.
2. Let the user reconsider (rephrase, clarify a legitimate purpose, or
   withdraw).
3. Distinguish illegal from sensitive-but-legal: educational,
   defensive-security, journalistic, academic framings are not treated as
   crime.
4. Hard refusal only for a narrow set; warn + inform by default.
5. Explainable (category + rationale in the warning), jurisdiction-aware
   (never a universal verdict), non-paternalistic tone, low
   false-positive bias for legitimate use, disclaimer everywhere,
   auditable and testable.

## Solution

- **Catalogue as seed data** — `data/seed/legality-patterns.lino`: ten
  `pattern` records (two refuse categories, eight warn categories spanning
  property crime, stolen goods, fraud, controlled substances, weapons,
  copyright piracy, malware deployment, impersonation) each with English
  and Russian trigger phrases, a readable reason, and a
  jurisdiction-variance note; plus `exemption` records naming the four
  sensitive-but-legal framings (education, security research, journalism,
  academic study). No phrase or category lives in Rust.
- **Advisory module** — `rust/src/legality_warning.rs`: `assess(prompt,
  normalized)` returns the matched pattern, disposition, and exemption
  framing; `handle_legality_warning(...)` renders the localized template
  from `data/seed/multilingual-responses-legality.lino` (en/ru/hi/zh/es)
  and logs `legality:category`, `legality:disposition`,
  `legality:framing` for audit. A warn-category request with a matched
  legitimate framing returns `None` — the caller answers normally with a
  `warn_suppressed_by_legitimate_framing` log event.
- **Templates** carry the disclaimer, the reconsideration invitation, and
  the category with its reason; the refusal variant names the category and
  stays short.

## Known components surveyed

- The issue asks to reuse the #835 file-legality taxonomy: the category
  slugs here are request-level counterparts (`copyright`, `weapons`,
  `controlled_substances`); a shared taxonomy record can merge them when
  #835's file-side catalogue lands here.
- Upstream content classifiers (OpenAI moderation categories, Perspective
  API) are model-based; the repo's formal-first doctrine keeps this
  catalogue deterministic and auditable, so they were noted and not
  adopted.

## Integration site left for main

`handle_legality_warning` is not yet in the dispatch chain — the natural
rank is ahead of the specialized handlers (the warning must fire "before
acting"); wiring `rust/src/solver_dispatch.rs` + `lib.rs` (`pub mod
legality_warning;`) + registering the two seeds is the main session's
integration step, recorded here so it is not lost.

## Verification

- Automated: nine tests in
  `rust/tests/unit/issue_836_legality_warning.rs` (CI) — warn content,
  refuse set, framing suppression, framing-proof refusal, ordinary-request
  pass-through, Russian twin, audit logging.
- Manual: replay each category phrase in the web UI and read the tone.

## Residuals

- The pattern reasons/jurisdiction notes are single-language catalogue
  notation; localized reasons per language are a seed follow-up (the
  templates themselves are already five-language).
- The category vocabulary can grow; it is a data edit, no Rust change.
