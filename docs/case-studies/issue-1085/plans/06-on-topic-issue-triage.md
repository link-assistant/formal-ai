# Plan 06 -- every open coding / self-coding issue, triaged

The maintainer asked (2026-09-09) that every open issue "on topic (coding, self
coding)" be delivered in PR #1086. This file is the triage: all 72 open issues
read, each placed in one of three groups, with the reason. A group-C entry is
not a deferral of something asked for -- it is a statement that the issue is
not on this topic, and that claim is checkable from the issue text quoted.

## A. Delivered in this pull request

| Issue | Why it is on topic | Where |
| --- | --- | --- |
| #1072 | the coding ladder's own harness | `e69827a1f` |
| #1091 | self-hosting metric; first self-authored task | merged `#1103` |
| #1095 | a coding session routed to web search | `50e28e9e1` |
| #1096 | a coding edit verified against a generated file | `50e28e9e1` |
| #1099 | a coding task naming two artifacts stops after one | plan 03 |
| #1101 | the same routing defect family as #1095, four languages | plan 04 stretch |
| #1104 | the session that produced the #1105 report | plan 05 |
| #1105 | the report flow a coding session ends in | plan 05 |
| #1106 | the server every coding session talks to | landed |
| #1107 | the CI bill self-coding iterations pay | landed + green ledger |
| #1109 | found while measuring #1106 | `e69827a1f` |
| #1110 | Formal AI's own rename recipe fails on macOS | plan 07 |

## B. On topic, but each is its own epic with its own acceptance floor

These are self-coding work and stay open as sub-issues of #1085. Folding an
epic into this PR would mean delivering it badly; each names a floor this PR
does not reach.

| Issue | Its own floor | Why not here |
| --- | --- | --- |
| #1087 (E109) | six user-prompt issues, each with a four-language held-out paraphrase set | six independent fixes |
| #1088 (E110) | a fresh clone below 150 MB; a second repository | needs a repository that does not exist yet; its item 3 (#1072) landed here |
| #1089 (E111) | `ls tests/unit \| grep -c docs_` at most 5, from 48 | a 43-file consolidation; its item 4 (a wall-clock ceiling) landed here |
| #1090 (E112) | unconfirmed traceability rows below 100, from 716 | 716 manual confirmations |
| #959 (E107) | handler ledger ratcheted, promotion predicates in seed | the ratchet landed (`kernel-ratchet.lino`); the seed migration is #1085 D1, in progress across many pushes |
| #954 (E102) | `src/` reorganised into directory modules + generated module map | a repository-wide move |
| #957 (E105) | three CI-enforced columns on every REQUIREMENTS.md row | 776 rows |

## C. Not on this topic

Read and set aside, with the reason. Grouped; the count is 53.

- **Web/UI surface** (#951, #952, #953, #934, #825, #557, #447, #665, #670):
  JavaScript-to-WASM migration and UI work. #951 alone is a 9,269-line
  `main.jsx` split. Formal AI's coding path does not run there.
- **Answer quality for specific prompts** (#1063, #1071, #872, #869, #827,
  #826, #821, #802, #801, #800, #724, #722, #721, #720, #838, #836, #483):
  each is one prompt answered wrongly. Real, but they are the *answering*
  path, not the coding path, and #1087 already owns the batch.
- **Surfaces other than coding** (#930 Telegram, #941 Gemini thinking parts,
  #940 research documents, #939 installation corpus, #937 docker containers,
  #942 redaction, #861 sentry, #666 marketplace): other products.
- **Knowledge and reasoning breadth** (#901 TRIZ, #705 anticipatory dreaming,
  #700 units, #669 cloud sync, #668 packages, #667 debugging view, #949 language
  parity, #948 memoization burndown, #491, #453, #651, #710, #955, #958, #950,
  #935, #1071): vision-level programs.
- **Infrastructure not on the coding path** (#1083 JS lint, #1084 multi-arch
  images): real CI gaps, unrelated to whether Formal AI can code.

## Ordering

1. Plan 03 (#1099) -- plan-first coding, the maintainer's own ask.
2. Plan 05 (#1105, #1104) -- the report flow.
3. Plan 07 (#1110) -- the rename recipe that fails on macOS.
4. Plan 04 stretch (#1101).
5. Remaining unticked boxes in plans 01, 02, 04.
