# Local doc consistency findings (2026-07-14)

## Stale / inconsistent items found by direct reading

1. **ARCHITECTURE.md §17 References**: says "REQUIREMENTS.md — issue-by-issue implementation matrix (R1 ... R305)" — matrix now reaches R444 (+ R499-1..8 block). Stale range.
2. **ROADMAP.md headings claim "current PR" for merged PRs**:
   - "Issue #408 Text And Code Editing - current PR" (PR #416 merged 2026-06-12)
   - "Issue #538 Detailed Meanings and Words - current PR" (PR #601 merged 2026-07-02)
   - "Issue #526 Translation Quality - current PR" (PR #635 merged 2026-07-04)
   All merged → wording stale; should say "merged".
3. **ROADMAP.md has no rows/sections for the latest big issues**: #558 auto-learning (PR #637), #540 dreaming (PR #645), #649 world models (PR #675), #499 learn-from-source (PR #641), #482 Nemotron (PR #639), #559 general meta algorithm (PR #560), #680 capability router. Vision-pillar table (26 pillars) predates these; no pillar for world models, dreaming/self-improvement, agentic CLI control, auto-learning.
4. **ROADMAP.md does not mention the open E35–E55 planning batch** (issues #656–#674 from #651 "most critical missing features"). The roadmap ends with "no vision-planning epic remains open" (for #244) which is misleading now that E37–E55 are open.
5. **VISION.md "Current Direction"** ends at issues #408/#526-era; doesn't reflect: meta method registry/self-improvement (#559), dreaming (#540), auto-learning/self-healing (#558), world models (#649), agentic-coding mode / controlling external agent CLIs (#468, #680, #681, #682), learn-from-source (#499).
6. **VISION.md Product Shape** doesn't list the VS Code extension though README does.
7. **docs/meta-algorithm.md** documents 6 recipes (procedural how-to, agentic coding, response-language, document verification, market-price, dreaming) — check against data/meta/*.lino for others (self-healing #558? issue-680 capability router?).
8. **GOALS.md** has no goals about: controlling external AI agent CLIs, world models, self-modification/dreaming, online learning, predicting next user requests, parallel multi-solution attempts & comparison.
9. **NON-GOALS.md** — vision says no neural inference, but open issue #483 proposes "experimental fallback for formalization using small models" — potential contradiction to reconcile (needs explicit boundary statement).
10. **USER-JOURNEYS.md** future journeys F1–F6 now have dedicated epic issues (#672 F1-F5 UI follow-ups? actually E52=#672 is issue-541 UI follow-ups F1–F5; E49=#669 F3 cloud sync; E50=#670 F5 WebVM; E48=#668 F6 packages; E42=#662 F4 search) — journeys doc should link them.

## Open issues snapshot (34)
687 bug: reporting issue via chat fails; 686 associative knowledge networks learning (contexts, world models, formal systems); 682 OpenAI content:null 400 (qwen); 681 agentic CLI read-instead-of-write; 674 E55 compile arbitrary NL programs; 673 E54 workspace AST census; 672 E53 issue-541 UI follow-ups; 671 E52 multi-CLI agentic e2e matrix in CI; 670 E51 WebVM; 669 E50 cloud memory sync; 668 E49 shareable packages; 667 E48 step debugger; 666 E47 marketplace publish; 665 E46 PWA+npm; 664 E45 terminology cleanup; 663 E44 retire SPECIALIZED_HANDLERS; 662 E43 budget-driven random/evolutionary search; 661 E42 probability-weighted formalization + contradiction warnings; 660 E41 bulk semantics importer; 659 E40 CI lint hardcoded strings; 658 E39 absorb JS worker into WASM; 657 E38 self-hosting metric; 656 E37 benchmark-gated promotion protocol; 651 create issues for critical missing features; 557 embedded buttons UI; 534 repo disk space; 531 patterns inference (1D/2D); 491 principle of least action; 483 small-model fallback for formalization; 453 moonshot tasks; 447 dialog UI terrible (ru).

## PR merge status checked
#416, #601, #635, #637, #639, #641, #645, #675 all MERGED.
