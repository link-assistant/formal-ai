# The progressive JPEG method, JavaScript first, and automated translation

Source: the architect's 2026-10-08 instructions while continuing
[PR #1188](https://github.com/link-assistant/formal-ai/pull/1188). Every quote is copied byte for byte from
[`docs/case-studies/pull-request-1188/user-messages.md`](../case-studies/pull-request-1188/user-messages.md),
which `node experiments/formal_ai_subagent/collect-user-messages.mjs --write` regenerates from the session history.
The requirements these words became are rows R1188-U28 to R1188-U30 of
[`docs/requirements/issue-1188-user-requirements.md`](../requirements/issue-1188-user-requirements.md); this note keeps the words.

2026-10-08 (15:41 UTC), message 145 of `user-messages.md`:

> Can we apply progressive jpeg princile https://www.artlebedev.ru/kovodstvo/sections/167 - as one of methods to attack each problem and as method of development of current pull request? Also we should try to use more of Formal AI via JavaScript code, make sure all JavaScript requirements will pass first and so on, make sure we have fully automated translation between JavaScript/TypeScript/Rust and MetaLanguage, so we can do less translation of code manually and more of it automatted, and even if relative meta logic and meta language are still not fully done, we can make temporary workarounds here, so later we will improve RML and ML as they are being worked in parallel at the moment.

2026-10-08 (15:41 UTC), message 146 of `user-messages.md`:

> It also must be recorded in our documents.

## The referenced principle

[§ 167 "Метод прогрессивного джипега"](https://www.artlebedev.ru/kovodstvo/sections/167/) (Artemy Lebedev, 26 November 2010): "В любую секунду любой проект готов на 100%, хотя проработанность может быть и на 4%" ("at any second, any project is 100% ready, although it may be only 4% worked out"). A project "can be worked out down to the pixel, or left at the stage of a conceptual sketch", and "time is spent only on the needed degree of doneness, while the overall look of the steak is always clear".

## What changed

- The progressive JPEG method is how every problem and this pull request are attacked: the whole is always complete at some resolution, and each pass raises the resolution of everything, lowest first, instead of finishing one part while the rest is missing ([`docs/progressive-delivery.md`](../progressive-delivery.md), R1188-U28).
- More of the work goes through Formal AI from its JavaScript source, and the JavaScript requirements pass first; the other roots follow (R1188-U29).
- Translation between JavaScript, TypeScript, Rust and the meta language is automated, so less code is translated by hand; where relative meta logic (RML) and the meta language (ML) cannot translate yet, a temporary workaround in this repository does it, recorded so it is retired as RML and ML improve in parallel (R1188-U30).
