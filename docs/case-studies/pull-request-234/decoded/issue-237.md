# Unknown prompt: Привет, расскажи о себе.

## Environment

- **Version**: 0.104.0
- **URL**: https://link-assistant.github.io/formal-ai/
- **Worker**: wasm worker
- **Mode**: manual
- **Status**: Manual mode
- **Diagnostics**: off
- **Timestamp**: 2026-05-23T21:33:52.906Z

## User Context

- **UI languages**: *en-US*
- **Theme**: auto (dark)
- **UI**: 1712x923 viewport, 1728x1117 @2x screen, Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Safari/605.1.15 browser, MacIntel platform
- **Locale**: en-US (Asia/Calcutta)
- **Guess probability**: 80%
- **Temperature**: 0.7
- **Follow-up probability**: 75%
- **Location**: inferred from time zone / locale only

## Reproduction of dialog

Legend: `U` = user, `A` = agent.

```
U: Привет, расскажи о себе.
A (intent: unknown, reported): Это для меня новое. Я пока не могу ответить на это по локальным правилам Links Notation. Чтобы посмотреть текущие правила, отправьте `List behavior rules`, затем `Show behavior rule unknown`. Чтобы научить этот диалог ответу, отправьте: When I say `ваш запрос`, answer `ваш ответ`. Для постоянного факта или правила экспортируйте память или нажмите Report issue, чтобы разработчики добавили встроенную возможность.
U: Привет
A: Привет! Что вас интересует?
U: Расскажи о себе
A: Себе, Леннокс: Леннокс Лесли Вонгама Себе — южноафриканский политик времён апартеида, президент бантустана Сискей в 1981—1990. Установил в Сискее диктаторский режим, проводил крайне правый политический курс. Свергнут в результате переворота.

   Source: [https://ru.wikipedia.org/wiki/Себе%2C_Леннокс](https://ru.wikipedia.org/wiki/%D0%A1%D0%B5%D0%B1%D0%B5%2C_%D0%9B%D0%B5%D0%BD%D0%BD%D0%BE%D0%BA%D1%81) (wikipedia).
```

## Description

<!-- Please describe what looked wrong or incomplete. -->

## Attach full memory (optional)

Click **Export memory** in the topbar to save `formal-ai-memory.lino`, then attach it as a [GitHub Gist](https://gist.github.com) or wrap it in a `.zip` first. Redact sensitive content before uploading. See the [upload-memory guide](https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md) for the full walkthrough.
