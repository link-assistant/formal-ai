## Environment

- **Version**: 0.183.0 (wasm)
- **URL**: https://link-assistant.github.io/formal-ai/
- **Timestamp**: 2026-06-11T08:34:41.204Z

## User Context

- **UI languages**: *en-US*
- **UI**: 1712x923 viewport, 1728x1117 @2x screen, Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Safari/605.1.15 browser, MacIntel platform
- **Locale**: en-US (Asia/Calcutta)

## Reproduction of dialog

Legend: `U` = user, `A` = agent.

````
... omitted 1 earlier message ...
A: Вот код на JavaScript, который сортирует числа 3, 5, 6, 7, 8 по возрастанию:

   ```javascript
   const numbers = [3, 5, 6, 7, 8];
   const sorted = [...numbers].sort((a, b) => a - b);
   console.log(sorted.join(", "));
   ```

   Результат: 3, 5, 6, 7, 8
U: Отсортируй 4, 3, 1, 17, 8, 9, 15
A (intent: unknown): Я тебя не понял. Я пока не могу ответить на это по локальным правилам связей. Чтобы посмотреть текущие правила, отправьте `Покажи правила поведения`, затем `Покажи правило unknown`. Чтобы научить этот диалог ответу, отправьте: Когда я скажу `ваш запрос`, ответь `ваш ответ`. Если после этих проверок всё ещё нужен общий seed-факт или правило связей в формате Links Notation, сообщите о недостающем правиле с диагностической трассировкой или экспортируйте память, чтобы сохранить правило этого диалога.
U: Покажи правила
A (intent: unknown, reported): Я пока не знаю, как ответить на это. Я пока не могу ответить на это по локальным правилам связей. Чтобы посмотреть текущие правила, отправьте `Покажи правила поведения`, затем `Покажи правило unknown`. Чтобы научить этот диалог ответу, отправьте: Когда я скажу `ваш запрос`, ответь `ваш ответ`. Если после этих проверок всё ещё нужен общий seed-факт или правило связей в формате Links Notation, сообщите о недостающем правиле с диагностической трассировкой или экспортируйте память, чтобы сохранить правило этого диалога.
````

We need configuration of our AI system much more user friendly, so everything can be done fully at chat, with high quantity of actual variations supported, that all should be formalized correctly in meta language, and be reasoned about.

We need to download all logs and data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), in which we will reconstruct timeline/sequence of events, list of each and all requirements from the issue, find root causes of the each problem, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration.

If issue related to any other repository/project, where we can report issues on GitHub, please do so. Each issue must contain reproducible examples, workarounds and suggestions for fix the issue in code. Also double check to fully apply requirements to entire codebase, so if we have issue in multiple places, it should be fixed in all them.

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.
