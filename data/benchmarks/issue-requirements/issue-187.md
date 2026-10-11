## Environment

- **Version**: 0.80.0
- **URL**: https://link-assistant.github.io/formal-ai/
- **Worker**: wasm worker
- **Mode**: manual
- **Status**: status.manual
- **Diagnostics**: off
- **Timestamp**: 2026-05-20T13:33:30.916Z

## User Context

- **UI languages**: *ru-RU*, ru, en-US, en
- **Theme**: auto (light)
- **UI**: 3440x1239 viewport, 3440x1440 @1x screen, Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:150.0) Gecko/20100101 Firefox/150.0 browser, Win32 platform
- **Locale**: ru-RU (Europe/Moscow)
- **Guess probability**: 80%
- **Temperature**: 0.7
- **Follow-up probability**: 75%
- **Location**: inferred from time zone / locale only

## Dialog

Legend: `U` = user, `A` = agent.

```
... omitted 27 earlier messages ...
U: Какой сегодня день?
A (intent: unknown, reported): Я пока не могу ответить на это по локальным правилам Links Notation. Добавьте факт или правило в Links Notation и повторите запрос.
```

## Reproduction Steps

1. Open https://link-assistant.github.io/formal-ai/
2. Send the prompt "Какой сегодня день?"
3. Click the report link on the dialog message

## Description

<!-- Please describe what looked wrong or incomplete. -->

## Attach full memory (optional)

Click **Export memory** in the topbar to save `formal-ai-memory.lino`, then attach it as a [GitHub Gist](https://gist.github.com) or wrap it in a `.zip` first. Redact sensitive content before uploading. See the [upload-memory guide](https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md) for the full walkthrough.

