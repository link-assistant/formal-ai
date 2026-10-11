Parent: #651

## Motivation and evidence

A full audit of all 329 closed issues and 317 merged PRs (2026-07-14) confirmed the maintainer's suspicion that many requirements were silently dropped at closure. Of 183 closed issues ≤ #350, only ~15% show clear delivery evidence; of 146 closed issues > #350, ~68 were partially addressed with silent scope-narrowing as the dominant failure mode. This issue is the **tracked regression backlog**: re-verify each flagged item against current `main`, fix what is still broken, and update `REQUIREMENTS.md` rows from "Implemented" to honest statuses where the audit contradicts them. (This also completes the closure-audit ask of issue #123, which itself went unmet.)

## Requirements

1. Re-verify each checklist item below against current `main`; classify: `works-now` (add the missing regression test), `still-broken` (fix here or split a focused sub-issue), `superseded` (name the superseding issue/PR), `blocked-upstream` (link the upstream issue, file it if never filed).
2. Update `REQUIREMENTS.md` and `ROADMAP.md` statuses to match the verified reality (requirement-level done/partial/not-done tracking per #651).
3. Every `still-broken` conversational item gets a pinned failing-then-passing test in the specification suites, in all four languages where applicable.

### Checklist (from the audit; issue refs in parentheses)

**Chat behavior:**
- [ ] Conversation-history recall "О чём мы разговаривали?" (#14, #27, #37)
- [ ] Russian identity/capabilities "Кто ты?", "Что ещё ты умеешь?" (#16, #29, #49, #65, #66, #190, #272)
- [ ] Multi-statement messages answered per statement, incl. many-question splits (#93, #137, #445)
- [ ] Context-qualified questions ("iir в ml") (#20, #31)
- [ ] Typo-tolerant understanding + clarification (#69, #82, #343); fuzzy matching across the whole formalization path (PR #197 narrowing)
- [ ] "Что такое антирежим?" / "ложная тотальность?" definition class (#286, #288)
- [ ] "What's in my folder" prompt variants closed with no fix (#543, #544, #545)
- [ ] Ambiguous-modification prompts ask one clarifying question instead of `unknown` (#359)
- [ ] Free-time small talk: multiple deterministic answer variants, not one canned reply (#402)
- [ ] Assistant name set/read via chat in all languages (#156, #284); attribution fact (#157)

**Localization:**
- [ ] #292's four asks: localized rules listing, answer-in-question-language guarantee, CI language-parity checks (extend those from PRs #229/#231), markdown fix
- [ ] Thinking-step localization on non-UI surfaces (CLI/API/Telegram render English only) (#488)
- [ ] Thinking-UX spec: collapsed animation ~1.5 visible steps, thinking on top (#488 / PR #489)

**Knowledge/reasoning:**
- [ ] Formal-proof translation to other programming languages (#403)
- [ ] ≥ 50 equation-type examples verified (#406)
- [ ] Calculations combined with other instructions, general composition (#407)
- [ ] Word problems beyond the train-meeting normalizer (#460)
- [ ] Films-in-release-order class without the stale hardcoded Spider-Man seed (#462; seed stale after 2023)
- [ ] Pronoun resolution to closest contextual match via meta algorithm (#465)
- [ ] "How to X" multi-source synthesis + 7-day service-availability cache (#444)
- [ ] Summarization: the 2-random-files-until-stable iterative validation + 80% quality bar; markdown recursive embedded grammars claim (#563)
- [ ] Interior/plain-capitalized entity reasoning covering the entire class (Claude, Tesla, Wikipedia) (#571)

**Platform/process:**
- [ ] Calendar export in all common formats + Apple/Google/Microsoft insertion flows (#404, #506, #507, #508)
- [ ] OCR optional bundle with settings gate (tesseract.js) + image-attachment transcription failing test (#205, #493)
- [ ] e2e suite against the deployed GitHub Pages URL (#1)
- [ ] CI/CD file-by-file comparison vs the four pipeline templates + upstream filings; desktop-template fixes left "ready-to-file" (#4, #24, #72, #84, #121, #347, #442, #479)
- [ ] Coverage measurement + the "double tests toward ~100%" ask made concrete: publish coverage, set a ratchet (#449, #451 / PRs #450, #452)
- [ ] gemini headless `-p` advertises no tools — wrapper limitation documented but never fixed/upstreamed (#620 comment on PR #623)
- [ ] macOS auto-update blocked on signing/notarization — track the production path (#548)
- [ ] link-foundation/start + command-stream adoption or upstream feature issues (#546, #8, #195)
- [ ] web-search/web-capture used as real components, not deferred adapters (#410)
- [ ] Iframe embeddability pre-check + external-link buttons (#71, #125, #169)

## Acceptance criteria

- Every checklist row has a verdict with evidence links (test name, commit, superseding issue, or upstream URL) recorded in the case study.
- `REQUIREMENTS.md` contains no row the audit proved false that still claims "Implemented".
- All `still-broken` conversational rows have failing-then-passing pinned tests merged.

## Dependencies

- Blocks the honesty of #657 (E38) and the roadmap requirement-status tracking (#651).
- Related: every issue named above; the audit reports live in the maintainers' session records and should be attached to the case study.

## Process

Collect data to `docs/case-studies/issue-{id}` (the audit tables, per-row verification evidence); split focused sub-issues only where a row is a large feature; otherwise fix in the single PR series of this issue.

