# Decoded GitHub Issue URL

- Repository: `link-assistant/formal-ai`
- Path: `/link-assistant/formal-ai/issues/new`
- Labels: `bug`

## Title

Unknown prompt: какие факты ты знаешь?

## Body

## Environment

- **Version**: 0.68.0
- **URL**: https://link-assistant.github.io/formal-ai/
- **User Agent**: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:150.0) Gecko/20100101 Firefox/150.0
- **Worker**: wasm worker
- **Mode**: manual
- **Status**: status.manual
- **Diagnostics**: off
- **Timestamp**: 2026-05-19T15:52:57.597Z

## User Context

- **UI Language**: ru
- **UI Language Preference**: auto
- **Theme Preference**: auto
- **UI Skin**: flat
- **Chat Style**: cards
- **Composer Style**: flat
- **Composer Action**: attach
- **Browser Language**: ru-RU
- **Browser Languages**: ru-RU, ru, en-US, en
- **Locale**: ru-RU
- **Time Zone**: Europe/Moscow
- **Color Scheme**: light
- **Preferred Location**: not set
- **Guess Probability**: 80%
- **Temperature**: 0.7
- **Viewport**: 1486x706
- **Screen**: 1536x864 @1.25x
- **Platform**: Win32
- **Online**: yes
- **Location Inference**: time zone / locale only; exact geolocation was not requested

## Dialog

Legend: `U` = user, `A` = agent.

```
U: Hi
A: Hello — what would you like to explore?
U: Who are you?
A: I am formal-ai, a deterministic symbolic AI prototype that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.
U: Привет
A: Привет! Что вас интересует?
U: 你是谁?
A: 我是 formal-ai —— 一个确定性的符号化 AI 概念验证项目,根据本地的 Links Notation 规则和兼容 OpenAI 的 API 形式作答。本演示不进行任何神经网络推理。
U: Привет. ты кто?
A (intent: unknown): Я пока не знаю символьного правила для этого запроса. Добавьте факт или правило в Links Notation и повторите запрос.
U: Привет. ты кто?
A: Я formal-ai — детерминированный символьный ИИ, который отвечает на основе локальных правил Links Notation и совместимых OpenAI-форматов. В этой демонстрации я не выполняю нейросетевой инференс.
U: Что значит символьный ИИ?
A: Искусственный интеллект: Иску́сственный интелле́кт, также ИИ, искусственный ра́зум, в самом широком смысле — научно-технологическая область, занимающаяся изучением и созданием интеллектуальных сущностей, способных «вычислять, как им действовать эффективно и безопасно в самых разнообразных, в том числе незнакомых им, ситуациях», и решать задачи как минимум уровня человеческого интеллекта и реализованных машинами, в частности компьютерными системами. Это направление исследований в области компьютерных наук, которая разрабатывает и изучает методы и программное обеспечение, позволяющие машинам воспринимать окружающую среду и использовать обучение и интеллект для выполнения действий, которые максимально увеличивают их шансы на достижение поставленных целей. Такие машины можно назвать искусственным интеллектом. В то же время не следует путать понятия ИИ и больших языковых моделей (БЯМ/LLM): если ИИ — это область компьютерных наук, охватывающая решение широкого круга интеллектуальных задач, то БЯМ/LLM — это лишь один из многих инструментов, который, даже развившись в мультимодальные системы с поддержкой изображений, видео и аудио, остаётся, по сути, специализированным вероятностным алгоритмом предсказания последовательностей, а не универсальным интеллектом.

   Source: [https://ru.wikipedia.org/wiki/Искусственный_интеллект](https://ru.wikipedia.org/wiki/Искусственный_интеллект) (wikipedia).

   Ближайшее совпадение по поиску Wikipedia: «Искусственный интеллект». Если это не то, уточните запрос.
U: То есть ты не используешь OpenAI api?  И вся твоя область знаний лежит в локальных правилах - ссылках? По запросу пользователя ты ищешь подходящую ссылку в интернете?
A (intent: unknown): Я пока не могу ответить на это по локальным правилам Links Notation. Добавьте факт или правило в Links Notation и повторите запрос.
U: Ты LLM?
A (intent: unknown): Я пока не могу ответить на это по локальным правилам Links Notation. Добавьте факт или правило в Links Notation и повторите запрос.
U: какие факты ты знаешь?
A (intent: unknown, reported): Я пока не могу ответить на это по локальным правилам Links Notation. Добавьте факт или правило в Links Notation и повторите запрос.
```

## Reproduction Steps

1. Open https://link-assistant.github.io/formal-ai/
2. Send the prompt "какие факты ты знаешь?"
3. Click the report link on the dialog message

## Description

<!-- Please describe what looked wrong or incomplete. -->

## Attach full memory (optional)

Click **Export memory** in the topbar to save `formal-ai-memory.lino`, then attach it as a [GitHub Gist](https://gist.github.com) or wrap it in a `.zip` first. Redact sensitive content before uploading. See the [upload-memory guide](https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md) for the full walkthrough.
