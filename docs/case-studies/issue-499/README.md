# Issue 499 Case Study: Learn From This Data Source

Status: **delivered and driven by the Agent CLI + Formal AI**, not deferred.

Issue [#499](https://github.com/link-assistant/formal-ai/issues/499) reported that
a user teaching the engine where to find current knowledge — the Russian directive
`Обратясь сюда ты узнаешь актуальные темы https://trends.google.com/trending?hl=ru&&geo=US`
("turn here and you'll learn the current topics <Google Trends URL>") — was
answered with `intent: unknown`. The maintainer then clarified the ambition: the
engine should have automated tools that convert a source like Google Trends into
data collection for the auto-learning process, in every supported language.

The implemented slice makes that directive a first-class, language-agnostic
**learn-from-source** intent. The same natural-language teaching directive now
drives *two* entry points through one seed-declared registry:

- in chat, it is recognized as `learn_from_source` and acknowledged in the
  prompt's language, reporting the auto-learning coverage split; and
- through the **Agent CLI**, the very same directive drives the human-gated
  Google Trends learning recipe that writes the committed learning report.

Nothing is auto-adopted: the frontier of prompts the engine cannot yet resolve
flows into the issue-#558 human-gated self-improvement loop for triage.

## Source material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/499>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/641>
- Raw GitHub and Google Trends data: [`raw-data/`](raw-data/)
- Requirements decomposition: [`requirements.md`](requirements.md)
- Per-requirement solution plan: [`solution-plans.md`](solution-plans.md)
- Online research notes: [`raw-data/online-research.md`](raw-data/online-research.md)
- Committed learn-from-source Agent CLI session:
  [`agent-cli-session-learn-from-source.json`](agent-cli-session-learn-from-source.json)
- Captured end-to-end Agent CLI run (real external CLI ↔ live server):
  [`agent-cli-e2e-run-learn-from-source.log`](agent-cli-e2e-run-learn-from-source.log)
- Seed-declared learnable-source registry:
  [`../../../data/seed/learning-sources.lino`](../../../data/seed/learning-sources.lino)
- Generated learning-frontier artifact:
  [`../../../data/meta/google-trends-learning.lino`](../../../data/meta/google-trends-learning.lino)

## 1. Collected Data

- Issue snapshot: `raw-data/issue-499.json`.
- Issue comments (incl. the maintainer's auto-learning directive):
  `raw-data/issue-499-comments.json`.
- Prepared PR snapshot: `raw-data/pr-641.json`.
- PR conversation, review-comment, and review snapshots:
  `raw-data/pr-641-conversation-comments.json`,
  `raw-data/pr-641-review-comments.json`, and `raw-data/pr-641-reviews.json`.
- Related merged work: `raw-data/recent-related-merged-prs.json`.
- Google Trends RSS snapshot and headers: `raw-data/google-trends-us-rss.xml`
  and `raw-data/google-trends-us-rss-headers.txt`, collected from
  `https://trends.google.com/trending/rss?geo=US` on `2026-07-08T20:29:32Z`.
- Before-state manual runs showing the reported failure:
  `raw-data/manual-formal-ai-grok-4-5-before.log` and
  `raw-data/manual-formal-ai-blue-jays-before.log`.
- Online research notes: `raw-data/online-research.md`.
- The committed Agent CLI session and captured E2E run record the exact
  `write_file`, verification command, and final answer produced when the reported
  directive drives the learning recipe.

No issue screenshots were present, so there were no image attachments to
download or verify.

## 2. Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R499-1 | Preserve issue data, PR data, online research, requirements, solution plans, and the Google Trends source snapshot under `docs/case-studies/issue-499`. | This directory stores all raw GitHub snapshots, the raw Trends RSS XML and headers, this README, requirements, solution plans, research notes, and the Agent CLI transcript/session. |
| R499-2 | The reported directive must no longer return `intent: unknown`. | `try_learn_from_source` routes it to `intent "learn_from_source"` with high confidence; `tests/unit/issue_499_learn_from_source.rs` pins the exact reported prompt. |
| R499-3 | Recognition must be language-agnostic, proven with different wording per language. | Detection reads the seed `learning_sources` registry (language-agnostic directive cues + native-language keywords) so `en`, `ru`, `hi`, and `zh` directives all route; the acknowledgement is localized to the prompt's language. |
| R499-4 | Routing must be data-driven, never a hardcoded URL or phrase. | Production code branches only on a source's declared `capability` slug (`LearningSources::match_directive`); a directive needs both a learning cue and a declared source, and a new source is a seed edit. |
| R499-5 | The directive must feed the auto-learning process rather than answer in isolation. | Both entry points route the Google Trends learning frontier through the human-gated issue-#558 self-improvement loop; of 80 trending prompts, 20 are already routed and 60 land on the frontier — nothing is auto-adopted. |
| R499-6 | Drive the *same* teaching directive through Formal AI's own Agent CLI. | `is_google_trends_learning_task` recognizes the seed-driven directive, so `run_agentic_task(<reported prompt>)` walks `write_file -> run_command -> final` and writes `data/meta/google-trends-learning.lino`. |
| R499-7 | Pin the Agent CLI session byte-for-byte and prove it live. | `tests/unit/issue_499_learn_from_source.rs` compares a fresh run with `agent-cli-session-learn-from-source.json`; `.github/workflows/release.yml` drives the directive through the real external Agent CLI in CI. |
| R499-8 | Keep the documentation and implementation contract test-covered. | `tests/unit/docs_requirements_issue_499.rs` protects this case-study evidence, and the issue-specific tests protect routing, localization, recipe driving, and artifact freshness. |

## 3. Root Cause

Formal AI could answer a user-supplied prompt, but it had no notion of a user
*teaching* it where to keep learning from. The reported directive names a data
source (Google Trends) and asks the engine to learn current topics from it, but
no intent captured "learn from this source": the URL fell through navigation and
research routing, and the prompt landed on the `unknown` opener.

The maintainer's clarification widened the gap: the fix is not a one-off answer
but an automated bridge from a live source into the auto-learning process, in
every supported language. That requires:

- a language-agnostic way to recognize a "learn from this source" directive;
- a data-driven registry of learnable sources so new sources are seed edits;
- a route from the directive into the existing human-gated learning loop; and
- the *same* directive driving the Agent CLI recipe, not a separate operator
  phrase, so the user's own words are the automation trigger.

## 4. Implemented Design

`data/seed/learning-sources.lino` is the new registry. It declares each learnable
`source` (its `capability`, `host`, and the native-language `keyword`s that name
it) plus the shared, language-agnostic `directive` cues that mark a "learn from
this source" request. Routing reads this data instead of branching on a URL, so a
new source is a data edit.

`src/seed.rs::LearningSources::match_directive` is the single source of truth. A
directive matches only when the lowercased prompt carries **both** a directive cue
(e.g. `learn from`, `узнаешь`, `यहाँ से सीख`, `在这里了解`) **and** a reference to a
declared source (its host or one of its keywords). Both entry points call it:

- `src/solver_handlers/mod.rs::try_learn_from_source` — the chat handler. It
  matches the directive, looks up the source's `capability`, renders the
  auto-learning summary for that capability, localizes the acknowledgement to the
  prompt's language, and answers with `intent "learn_from_source"`.
- `src/agentic_coding/google_trends_learning.rs::is_google_trends_learning_task`
  — the Agent CLI recipe router. It now recognizes the same seed-driven directive
  (for the `google_trends_learning` capability) in addition to the operator-worded
  learning-frontier task, so the reported prompt drives the recipe that writes
  `data/meta/google-trends-learning.lino`.

`src/google_trends_learning.rs::trending_learning_report` is the auto-learning
loop the directive feeds. It re-answers every Google Trends catalog prompt, splits
them into the ones the engine already routes (20) and the **learning frontier**
(60), and hands that frontier to the human-gated issue-#558 learner. Trending
searches are open-domain questions, not program-plan modifiers, so the learner
honestly adopts nothing — the value is the auditable frontier and the proof the
gap flows into the gated loop rather than off a cliff.

## 5. Prior Art And Existing Components

| Component | Relevance | Decision |
| --- | --- | --- |
| Issue #498 Google Trends catalog + learning recipe | Already turns Trends into multilingual prompts and routes the frontier through the gated loop. | Reused directly: the #499 directive routes into the *same* `trending_learning_report` and Agent CLI recipe rather than a parallel path. |
| Issue #558 human-gated self-improvement loop | Provides the proposal-only learner that adopts nothing without review. | Reused as the destination for the learning frontier the directive surfaces. |
| Existing seed-driven routing (intents, responses) | The engine already routes intents from data, not code branches. | Mirrored with a new `learning-sources.lino` registry so the directive is data-driven and multilingual. |
| Google Trends RSS feed / API alpha / pytrends | Candidate live-source ingestion paths. | RSS snapshot kept as the deterministic source; API alpha and pytrends recorded as future prior art, not CI dependencies. |
| Issue #527 Agent CLI session pinning | Established byte-for-byte generated artifacts driven by the in-repo CLI. | Mirrored: the reported directive's Agent CLI session is pinned and driven live in CI. |

## 6. Verification

A reproducing test was added before the fix:

```sh
cargo test --test unit issue_499 -- --nocapture
```

Before the change, the reported prompt returned `intent: unknown`. After the
change, focused coverage verifies:

- the exact reported prompt routes to `learn_from_source` with high confidence;
- the directive is recognized across `en`, `ru`, `hi`, and `zh` with different
  wording each time, and the acknowledgement is localized to the prompt language;
- routing needs both a directive cue and a seed-declared source (a bare
  navigation request or a sourceless cue does not trigger it);
- the frontier flows into the human-gated loop and nothing is auto-adopted;
- the same reported directive drives the Agent CLI learning recipe to write
  `data/meta/google-trends-learning.lino`;
- a fresh Agent CLI run matches the committed
  `agent-cli-session-learn-from-source.json`; and
- this documentation/evidence contract.

The real external Agent CLI drives the directive end-to-end in CI
(`.github/workflows/release.yml`) against a live `formal-ai serve`, writing the
learning report and asserting the `human_gated` canary — captured in
`agent-cli-e2e-run-learn-from-source.log`.

To regenerate the pinned Agent CLI session after a routing or report change:

```sh
cargo run --example issue_499_dump_agent_cli_session \
  > docs/case-studies/issue-499/agent-cli-session-learn-from-source.json
```
