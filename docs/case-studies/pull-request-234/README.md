# Pull Request 234 Case Study

This folder preserves the evidence used to expand PR 234 from issue #146 into
the full self-awareness cluster requested in the PR discussion.

## Evidence Collected

- Raw issue payloads: `raw/issues/issue-<number>.json`
- Raw issue comments: `raw/issues/issue-<number>-comments.json`
- Raw PR payload and comments: `raw/pr/`
- Decoded generated issue bodies: `decoded/issue-<number>.md`

The decoded files came from the repository decoder script,
`scripts/decode-github-issue-url.rs`, so the report URL bodies are preserved as
reviewable Markdown next to the raw GitHub API data.

## Timeline

- 2026-05-19: issues #137, #139, #141, #142, #146, #147, #148, and #155 were
  reported from the browser demo. Most reports used `wasm worker`, Russian UI
  context, and versions 0.68.0 or 0.70.0.
- 2026-05-23: issue #237 reproduced the same self-introduction gap on version
  0.104.0 and additionally showed `Расскажи о себе` being routed to a
  Wikipedia biography for `Себе, Леннокс`.
- 2026-05-23: PR 234 initially fixed the direct issue #146 known-facts prompt
  and some LLM/OpenAI follow-ups.
- 2026-05-24: PR comment
  `raw/pr/pr-234-conversation-comments.json` requested a comprehensive pass
  over issues #137, #139, #141, #142, #146, #147, #148, #155, and #237, with
  raw evidence, a case study, prompt variations next to expected answers, and
  configurable self-awareness such as assistant-name status.

## Requirements Matrix

| Issue | Reported prompt or dialog cue | Required behavior | Implemented coverage |
| --- | --- | --- | --- |
| #137 | `Привет, расскажи о себе.` | Treat combined greeting plus self-introduction as identity, not unknown. | Early self-introduction recognizer returns localized `identity`. |
| #139 | `Что тебе вообще известно?` plus prior `Приветы, расскажи о себе` and `В чём идея твоей разработки?` | Explain known fact sources and answer self-awareness/project-purpose variants. | Known-facts recognizer handles general knowledge phrasing; project-purpose phrasing routes to `meta_explanation`; self-intro variants route to `identity`. |
| #141 | `Расскажи что тебе известно об окружающем мире` | Explain available fact classes and the limits of local memory versus internet lookup. | Known-facts recognizer no longer requires the literal word `факт`. |
| #142 | `Какая у тебя модель окружающего мира?` | Explain the system model/world model instead of falling through. | Architecture recognizer includes world-model phrasing. |
| #146 | `какие факты ты знаешь?` and comments about internet, memory, and system facts | Describe local seed facts, internet lookup, conversation memory, and self facts. | `known_facts` response includes Links Notation evidence for local seed, internet, memory, self, and assistant-name setting status. |
| #147 | `Ты LLM?` | Answer in Russian that this demo is not an LLM runtime and explain the architecture. | Mixed Cyrillic/Latin self-awareness prompts now force Russian response language. |
| #148 | OpenAI API, local rules, internet link follow-up | Explain OpenAI-compatible API shapes, deterministic solver, local rules, memory, and web lookup. | Existing architecture handling kept and mirrored across Rust and browser paths. |
| #155 | `какой принцип работы у тебя` | Explain the working principle. | Architecture recognizer includes `принцип работы` phrasing. |
| #237 | `Расскажи о себе` routed to Wikipedia `Себе, Леннокс` | Self-introduction must beat article/person lookup. | Browser worker handles self-introduction before the Wikipedia article question tool. |

## Root Causes

The failures were not one bug. They were a routing coverage cluster:

1. Russian self-introduction phrases such as `расскажи о себе` were absent from
   the identity route and from the early browser behavior-rule pass.
2. The browser worker attempted Wikipedia article lookup before it had claimed
   self-introduction, so `себе` could be misread as a named entity.
3. Known-facts recognition required explicit `facts` / `факт` wording, missing
   natural prompts like `что тебе вообще известно`.
4. Architecture recognition did not include `модель окружающего мира`,
   `принцип работы`, or project-purpose phrasing.
5. Language detection treated very short mixed-script prompts such as `Ты LLM?`
   as English because the Latin token dominated the character count.
6. Self-facts described model/rules/memory but did not expose assistant-name
   configuration status, even though the web app already supports that setting.

## Solution Plan

The chosen fix keeps the existing architecture: update seed routing, then mirror
the same predicates in Rust, the web worker, and the React-only local fallback.

- Add table-driven unit tests in `tests/unit/specification/issue_146.rs` where
  each reported prompt and nearby variation sits next to the expected intent and
  answer fragments.
- Route self-introduction before self-facts, known-facts, runtime behavior
  rules, and browser Wikipedia article lookup.
- Broaden known-facts predicates to general knowledge/world prompts in English,
  Russian, Hindi, and Chinese.
- Broaden architecture predicates for world model, working principle, and
  project-purpose prompts.
- Add local language overrides for self-awareness prompts that mix Cyrillic or
  other supported scripts with Latin technical tokens.
- Expose assistant-name status in `self_facts` and add `rule_assistant_name` to
  the behavior-rule catalog.

## External Research

- GitHub's REST API treats pull requests as issue-backed objects and exposes
  issue comments separately from pull request review comments, which matches the
  three raw PR comment files saved under `raw/pr/`.
  Source: https://docs.github.com/en/rest/issues/issues and
  https://docs.github.com/en/rest/pulls/comments
- OpenAI's Chat Completions API is message based and returns assistant messages
  with the `assistant` role. That supports the repository wording that
  formal-ai exposes OpenAI-compatible API shapes while this demo's answers are
  generated by its deterministic symbolic solver rather than neural inference.
  Source: https://platform.openai.com/docs/api-reference/chat/create

## Verification Strategy

The minimum verification for this cluster is the focused Rust unit spec:

```sh
cargo test issue_146 --test unit
```

The broader PR checks should also include formatting, JavaScript syntax checks,
the full Rust unit suite, file-size/changelog guards, and CI after pushing.

No external upstream issue is required: the observed failures are local routing
and fallback-ordering defects in this repository.
