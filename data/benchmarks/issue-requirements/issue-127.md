## Environment

- **Version**: 0.64.0
- **URL**: https://link-assistant.github.io/formal-ai/
- **User Agent**: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Safari/605.1.15
- **Worker**: wasm worker
- **Mode**: manual
- **Status**: Manual mode
- **Diagnostics**: off
- **Timestamp**: 2026-05-19T11:50:33.009Z

## User Context

- **UI Language**: en
- **UI Language Preference**: auto
- **Theme Preference**: auto
- **UI Skin**: flat
- **Chat Style**: cards
- **Composer Style**: flat
- **Composer Action**: attach
- **Browser Language**: en-US
- **Browser Languages**: en-US
- **Locale**: en-US
- **Time Zone**: Asia/Calcutta
- **Color Scheme**: dark
- **Preferred Location**: not set
- **Guess Probability**: 80%
- **Temperature**: 0.7
- **Viewport**: 1712x919
- **Screen**: 1728x1117 @2x
- **Platform**: MacIntel
- **Online**: yes
- **Location Inference**: time zone / locale only; exact geolocation was not requested

## Dialog

Legend: `U` = user, `A` = agent.

```
U: Какова столица Японии?
A: Столица Японии — Токио.
U: Столица России?
A (intent: unknown): Я пока не могу ответить на это по локальным правилам Links Notation. Добавьте факт или правило в Links Notation и повторите запрос.
U: Какова столица России?
A (intent: unknown): Я пока не могу ответить на это по локальным правилам Links Notation. Добавьте факт или правило в Links Notation и повторите запрос.
U: столица россии
A (intent: unknown, reported): Я пока не могу ответить на это по локальным правилам Links Notation. Добавьте факт или правило в Links Notation и повторите запрос.
```

## Reproduction Steps

1. Open https://link-assistant.github.io/formal-ai/
2. Send the prompt "столица россии"
3. Click the report link on the dialog message

## Description

<!-- Please describe what looked wrong or incomplete. -->

## Attach full memory (optional)

Click **Export memory** in the topbar to save `formal-ai-memory.lino`, then attach it as a [GitHub Gist](https://gist.github.com) or wrap it in a `.zip` first. Redact sensitive content before uploading. See the [upload-memory guide](https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md) for the full walkthrough.

