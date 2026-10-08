## Environment

- **Version**: 0.86.0
- **URL**: https://link-assistant.github.io/formal-ai/
- **Worker**: wasm worker
- **Mode**: manual
- **Status**: Manual mode
- **Diagnostics**: off
- **Timestamp**: 2026-05-21T12:27:07.742Z

## User Context

- **UI languages**: *en-US*
- **Theme**: auto (dark)
- **UI**: 1712x913 viewport, 1728x1117 @2x screen, Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Safari/605.1.15 browser, MacIntel platform
- **Locale**: en-US (Asia/Calcutta)
- **Guess probability**: 80%
- **Temperature**: 0.7
- **Follow-up probability**: 75%
- **Location**: inferred from time zone / locale only

## Reproduction of dialog

Legend: `U` = user, `A` = agent.

```
U: Переведи "как у тебя дела?" на английский.
A (intent: translate_ru_to_en, reported): meaning: meaning_2cfc55c914d57d9e
   surface (ru): как у тебя дела?
   surface (en): How are you?
```

## Description

The output is too robotic, we need to make it feel like natural conversation. Also we should prefer the original formatting of the translated message as well. For example if user started question from lower case latter, we should also start the translation with lower case later. So we should assume the flow, that with lowest possible probability leads for the user to introduce corrections.

We need to make sure to update our requirements and docs, to make sure in the future we will follow these guidelines. And implement fix for that case and add ability to translate not only single hardcoded `meaning_2cfc55c914d57d9e `, but all of them, meaning we need to have general logic for translation assisted by wikipedia/wikidata/wiktionary API.

So we should use formalize from source language to semantic meta language (in links, links notation), and after that just deformalize it to the target language. That should actually produce the most efficient flow of translation, as everything goes through single semantic meta language.

We should apply it all to all related places in the codebase and docs.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in a single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.
