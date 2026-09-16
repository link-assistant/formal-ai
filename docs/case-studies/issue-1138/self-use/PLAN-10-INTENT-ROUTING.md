# Plan 10 — the frontier queue, its paraphrases, and Spanish

**Binary.** `formal-ai 0.350.0`, built from `74875c1b9b6e36bee9b942343ba295541fdb6997`.
**Captured.** 2026-09-16. Agent CLI 0.26.0 in agent mode, and `formal-ai chat`
for the non-agent-mode half. 48 prompts, 61 runs.

Three sets:

1. the **seven reported frontier prompts** of #1087 (#720 #721 #722 #724 #869
   #1063 #447), verbatim from plan 10's table;
2. one **held-out paraphrase per intent per language** from plan 10's
   capability-routing sets — 7 intents × 5 languages;
3. six **Spanish routing variations** of the #745 matrix. Spanish is absent from
   `tests/unit/issue_745.rs` and from all seven capability cue lists in
   `data/seed/agentic-tool-capabilities.lino` (280 phrases, four languages, zero
   Spanish), so these are held out by construction.

## The seven reported prompts

| issue | prompt | 2026-07 report | observed now |
| --- | --- | --- | --- |
| #720 | `последние новости` | `unknown` | **routes** — searched, opened `news.mail.ru` and `ria.ru`, then produced an empty conversation-summary scaffold. No headlines, no dates. |
| #721 | `我不明白` | `unknown` | **answered** — from the literal now in `data/seed/intent-routing.lino:400-402` |
| #722 | `Привет, напиши мне эссе по квантовой механике` | `unknown` | **unchanged in kind** — sent to `websearch`, fetched a Pittsburgh course page, no composition |
| #724 | `Скажи что то на Китайском` | `unknown` | **unchanged** — sent to `websearch`; the language object is dropped |
| #869 | `Назначь мне встречу с Александром на 20:00 по Грузии` | `unknown` | **solved** — a complete `VCALENDAR` event at 20:00 `Asia/Tbilisi` |
| #1063 | `Какого размера средний корень яблони?` | `unknown` | **unchanged** — sent to `websearch`, fetched a cyberleninka paper on apple-root geometry, connection failed, gave up |
| #447 | `интерфейс ужасен.` | `unknown` | **unchanged** — sent to `websearch`, fetched two Habr articles. There is still no act for a complaint about the assistant's own surface. |

Two of seven now work. Both work as **literals** — see below.

### A query-construction defect the transcripts expose

In agent mode the search query is built by stripping a leading interrogative,
and the stripping is by substring rather than by token:

| prompt | query actually searched |
| --- | --- |
| `Какого размера средний корень яблони?` | `ого размера средний корень яблони` |
| `What's happening in the world right now?` | `happening in the world right now` |
| `How far down do oak roots usually reach?` | `far down do oak roots usually reach` |

`Какого` became `ого`: the three characters `Как` were removed from the middle of
a word. No unit test is added for this here, because the query is built on the
agent-mode tool-call path and `solver::solve` does not expose it — the defect is
recorded with its transcripts (`frontier_1063_measurement/ru/answer.txt`,
`class_news/en/answer.txt`) rather than claimed as covered.

## The two that work, work as literals

**#721.** `我不明白` is answered. Every held-out paraphrase of the same act is
sent to `websearch`, which fetches dictionary pages *about the idiom*:

| lang | paraphrase | route |
| --- | --- | --- |
| en | `That went over my head.` | websearch → `dictionary.cambridge.org/us/dictionary/english/over-head` |
| ru | `Это мне непонятно.` | **correct** (also seeded) |
| hi | `यह मेरे सिर के ऊपर से चला गया।` | websearch → `storiespub.com/sira-ke-upar-se-guzar-jana-…` |
| zh | `这个我没跟上。` | websearch → `dictionary.cambridge.org/…/跟不上` |
| es | `Eso se me escapó.` | websearch → `ldoceonline.com/es-LA/dictionary/spanish-english/se-me-le-escapo` |

Plan 10 named this shape before the run: *the fix for a missing phrasing is
another phrasing*. The run confirms it in four languages.

**#869.** The reported prompt makes a real calendar event. The verb `назначь`,
which plan 10 recorded as absent from `calendar_schedule_action`, has since been
added. None of the five held-out paraphrases reaches the route — all five are
sent to `websearch`, which obligingly fetches `time.is/Tbilisi` and
`timeanddate.com/worldclock/georgia/tbilisi`, and then produces no event.

## The speak-in-language object is dropped

All five `demonstrate` paraphrases go to `websearch`, and the language the prompt
names does not survive the trip:

| prompt | searched for |
| --- | --- |
| `Скажи что то на Китайском` | `Скажи что то` |
| `Give me a line in Hindi.` | `Give me a line` |
| `Дай фразу на хинди.` | `Дай фразу` |

`Hindi` / `хинди` occurs nowhere in the reply. The forced-response-language
machinery exists (`SolverConfig::forced_response_language`); only a follow-up
turn can set it.

## Spanish routing — the #745 matrix with the missing row

Agent CLI, agent mode, throwaway workspace containing only `opencode.json`:

| prompt | required | observed |
| --- | --- | --- |
| `Lee https://example.com y dime qué dice.` | `web_fetch` of the URL **in the prompt** | websearch → two LinkedIn posts and SpanishDict. The URL was never fetched. |
| `Muéstrame el contenido de 1.txt` | `read_file` | websearch → a Linux `cat` tutorial and two online .txt viewers |
| `Escribe hola en 1.txt` | `write_file` | websearch → StackOverflow-es, 403, gave up. **No file written.** |
| `Lista los archivos de este directorio` | `ls` | **correct** — ran `ls`, reported `opencode.json` |
| `Busca en la web qué es la notación de enlaces` | `web_search` | correct (trivially) |
| `Busca en el código RouteIntent` | `grep_search` | websearch → a random GitHub file, 404 |

Four of six misroute. The URL case is the one to keep: the prompt contains
`https://example.com` and the system searched the web for the sentence
*describing* the fetch instead of performing it.

## Local filesystem search — the honest one

The maintainer's pinned prompts (#1133), agent mode:

| lang | route | answer |
| --- | --- | --- |
| en | **filesystem** | `hive-mind-control-center was not found after exact, substring, and nearby-name checks within ${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop}. No wider location was searched.` |
| ru | **filesystem** | the same, localized |
| hi | websearch | opened `tutorialpandit.com/windows-desktop/` |
| zh | websearch | opened `github.com/hivemind-os/hivemind` |
| es | websearch | opened `spideroak.support/…/La-carpeta-Hive` |

The English and Russian answers are the best behaviour seen anywhere in this
wave: the right route, a real search, a negative result stated plainly, and the
scope of the search declared so the reader knows what was *not* looked at. Two
languages of five reach it.

## Outcome summary

| set | runs | correct | misrouted | class |
| --- | --- | --- | --- | --- |
| reported frontier prompts | 7 | 2 | 5 | 2 solved as literals |
| held-out class paraphrases | 35 | 3 | 32 | `memorized_literal` / `memorized_verb` |
| Spanish #745 variations | 6 | 2 | 4 | `wrong_route` |

## Tests this produced

`tests/unit/issue_1138_self_use_intent_routing.rs`, all red:

- `a_statement_of_non_understanding_routes_by_act_not_by_memorized_phrase` — 4/5
- `a_speak_in_language_request_keeps_the_language_it_names` — 5/5, both halves
- `a_scheduling_request_reaches_the_calendar_whatever_verb_it_uses` — 5/5 (the reported prompt passes as a control)
- `spanish_routing_variations_reach_their_capability` — 3/5 in chat mode, 4/6 in agent mode
- `a_local_location_never_becomes_a_web_search` — 3/5 in chat mode, 3/5 in agent mode
