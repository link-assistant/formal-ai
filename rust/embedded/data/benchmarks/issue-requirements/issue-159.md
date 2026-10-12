## Environment

- **Version**: 0.70.0
- **URL**: https://link-assistant.github.io/formal-ai/
- **User Agent**: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36
- **Worker**: wasm worker
- **Mode**: manual
- **Status**: Ручной режим
- **Diagnostics**: off
- **Timestamp**: 2026-05-19T18:01:37.007Z

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
- **Locale**: ru
- **Time Zone**: Asia/Yekaterinburg
- **Color Scheme**: light
- **Preferred Location**: not set
- **Guess Probability**: 80%
- **Temperature**: 0.7
- **Viewport**: 1360x641
- **Screen**: 1360x768 @1x
- **Platform**: Windows
- **Online**: yes
- **Location Inference**: time zone / locale only; exact geolocation was not requested

## Dialog

Legend: `U` = user, `A` = agent.

```
U: Что ты можешь
A: Я formal-ai — детерминированный символьный ИИ. Вот что я умею:
   
   - **Приветствия**: отвечаю на «Привет», «Здравствуйте» и т.п.
   - **Hello World**: генерирую программы на Rust, Python, JavaScript, Go, C и других языках.
   - **Поиск понятий**: объясняю термины — попробуйте «Что такое Википедия?»
   - **Арифметика**: вычисляю выражения — например, «Сколько будет 2 + 2?»
   - **Перевод**: перевожу фразы между языками.
   - **Память**: помню контекст разговора в рамках сессии.
   
   Я работаю на основе локальных символьных правил, без нейросетевого инференса.
U: Что такое Википедия?
A: Wikipedia (encyclopedia): Wikipedia is a free, multilingual online encyclopedia written and maintained by a community of volunteer contributors through a model of open collaboration.
   
   Source: https://en.wikipedia.org/wiki/Wikipedia (wikipedia).
U: Что такое Hive Mind?
A (intent: wikipedia_lookup, reported): LOIC: LOIC — программа с открытым исходным кодом, предназначенная для осуществления DoS-атак, написанная на языке программирования C#. Первоначально разработана Praetox Technologies, но позже была распространена, как общественное достояние. Название — отсылка к играм серии Command & Conquer.
   
   Source: [https://ru.wikipedia.org/wiki/LOIC](https://ru.wikipedia.org/wiki/LOIC) (wikipedia).
   
   Ближайшее совпадение по поиску Wikipedia: «LOIC». Если это не то, уточните запрос.
```

## Reproduction Steps

1. Open https://link-assistant.github.io/formal-ai/
2. Send the prompt "Что такое Hive Mind?"
3. Click the report link on the dialog message

## Description

<!-- Please describe what looked wrong or incomplete. -->

## Attach full memory (optional)

Click **Export memory** in the topbar to save `formal-ai-memory.lino`, then attach it as a [GitHub Gist](https://gist.github.com) or wrap it in a `.zip` first. Redact sensitive content before uploading. See the [upload-memory guide](https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md) for the full walkthrough.

https://github.com/link-assistant/formal-ai/issues/new?title=Unknown+prompt%253A+%25D0%25BD%25D0%25B5+%25D0%25B2%25D0%25B5%25D1%2580%25D0%25BD%25D0%25BE+%25D1%258D%25D1%2582%25D0%25BE+%25D0%25B2%25D0%25BE%25D1%2582+%25D1%2587%25D1%2582%25D0%25BE+https%253A%252F%252Fgithub.com%252Flink-assistant%252Fhive-mind&body=%2523%2523+Environment%250A%250A-+**Version**%253A+0.70.0%250A-+**URL**%253A+https%253A%252F%252Flink-assistant.github.io%252Fformal-ai%252F%250A-+**User+Agent**%253A+Mozilla%252F5.0+%2528Windows+NT+10.0%253B+Win64%253B+x64%2529+AppleWebKit%252F537.36+%2528KHTML%252C+like+Gecko%2529+Chrome%252F148.0.0.0+Safari%252F537.36%250A-+**Worker**%253A+wasm+worker%250A-+**Mode**%253A+manual%250A-+**Status**%253A+%25D0%25A0%25D1%2583%25D1%2587%25D0%25BD%25D0%25BE%25D0%25B9+%25D1%2580%25D0%25B5%25D0%25B6%25D0%25B8%25D0%25BC%250A-+**Diagnostics**%253A+off%250A-+**Timestamp**%253A+2026-05-19T17%253A58%253A01.109Z%250A%250A%2523%2523+User+Context%250A%250A-+**UI+Language**%253A+ru%250A-+**UI+Language+Preference**%253A+auto%250A-+**Theme+Preference**%253A+auto%250A-+**UI+Skin**%253A+flat%250A-+**Chat+Style**%253A+cards%250A-+**Composer+Style**%253A+flat%250A-+**Composer+Action**%253A+attach%250A-+**Browser+Language**%253A+ru-RU%250A-+**Browser+Languages**%253A+ru-RU%252C+ru%252C+en-US%252C+en%250A-+**Locale**%253A+ru%250A-+**Time+Zone**%253A+Asia%252FYekaterinburg%250A-+**Color+Scheme**%253A+light%250A-+**Preferred+Location**%253A+not+set%250A-+**Guess+Probability**%253A+80%2525%250A-+**Temperature**%253A+0.7%250A-+**Viewport**%253A+1360x641%250A-+**Screen**%253A+1360x768+%25401x%250A-+**Platform**%253A+Windows%250A-+**Online**%253A+yes%250A-+**Location+Inference**%253A+time+zone+%252F+locale+only%253B+exact+geolocation+was+not+requested%250A%250A%2523%2523+Dialog%250A%250ALegend%253A+%2560U%2560+%253D+user%252C+%2560A%2560+%253D+agent.%250A%250A%2560%2560%2560%250AU%253A+%25D1%2587%25D1%2582%25D0%25BE+%25D1%2582%25D1%258B+%25D1%2583%25D0%25BC%25D0%25B5%25D0%25B5%25D1%2588%25D1%258C%253F%250AA%253A+%25D0%25AF+formal-ai+%25E2%2580%2594+%25D0%25B4%25D0%25B5%25D1%2582%25D0%25B5%25D1%2580%25D0%25BC%25D0%25B8%25D0%25BD%25D0%25B8%25D1%2580%25D0%25BE%25D0%25B2%25D0%25B0%25D0%25BD%25D0%25BD%25D1%258B%25D0%25B9+%25D1%2581%25D0%25B8%25D0%25BC%25D0%25B2%25D0%25BE%25D0%25BB%25D1%258C%25D0%25BD%25D1%258B%25D0%25B9+%25D0%2598%25D0%2598.+%25D0%2592%25D0%25BE%25D1%2582+%25D1%2587%25D1%2582%25D0%25BE+%25D1%258F+%25D1%2583%25D0%25BC%25D0%25B5%25D1%258E%253A%250A+++%250A+++-+**%25D0%259F%25D1%2580%25D0%25B8%25D0%25B2%25D0%25B5%25D1%2582%25D1%2581%25D1%2582%25D0%25B2%25D0%25B8%25D1%258F**%253A+%25D0%25BE%25D1%2582%25D0%25B2%25D0%25B5%25D1%2587%25D0%25B0%25D1%258E+%25D0%25BD%25D0%25B0+%25C2%25AB%25D0%259F%25D1%2580%25D0%25B8%25D0%25B2%25D0%25B5%25D1%2582%25C2%25BB%252C+%25C2%25AB%25D0%2597%25D0%25B4%25D1%2580%25D0%25B0%25D0%25B2%25D1%2581%25D1%2582%25D0%25B2%25D1%2583%25D0%25B9%25D1%2582%25D0%25B5%25C2%25BB+%25D0%25B8+%25D1%2582.%25D0%25BF.%250A+++-+**Hello+World**%253A+%25D0%25B3%25D0%25B5%25D0%25BD%25D0%25B5%25D1%2580%25D0%25B8%25D1%2580%25D1%2583%25D1%258E+%25D0%25BF%25D1%2580%25D0%25BE%25D0%25B3%25D1%2580%25D0%25B0%25D0%25BC%25D0%25BC%25D1%258B+%25D0%25BD%25D0%25B0+Rust%252C+Python%252C+JavaScript%252C+Go%252C+C+%25D0%25B8+%25D0%25B4%25D1%2580%25D1%2583%25D0%25B3%25D0%25B8%25D1%2585+%25D1%258F%25D0%25B7%25D1%258B%25D0%25BA%25D0%25B0%25D1%2585.%250A+++-+**%25D0%259F%25D0%25BE%25D0%25B8%25D1%2581%25D0%25BA+%25D0%25BF%25D0%25BE%25D0%25BD%25D1%258F%25D1%2582%25D0%25B8%25D0%25B9**%253A+%25D0%25BE%25D0%25B1%25D1%258A%25D1%258F%25D1%2581%25D0%25BD%25D1%258F%25D1%258E+%25D1%2582%25D0%25B5%25D1%2580%25D0%25BC%25D0%25B8%25D0%25BD%25D1%258B+%25E2%2580%2594+%25D0%25BF%25D0%25BE%25D0%25BF%25D1%2580%25D0%25BE%25D0%25B1%25D1%2583%25D0%25B9%25D1%2582%25D0%25B5+%25C2%25AB%25D0%25A7%25D1%2582%25D0%25BE+%25D1%2582%25D0%25B0%25D0%25BA%25D0%25BE%25D0%25B5+%25D0%2592%25D0%25B8%25D0%25BA%25D0%25B8%25D0%25BF%25D0%25B5%25D0%25B4%25D0%25B8%25D1%258F%253F%25C2%25BB%250A+++-+**%25D0%2590%25D1%2580%25D0%25B8%25D1%2584%25D0%25BC%25D0%25B5%25D1%2582%25D0%25B8%25D0%25BA%25D0%25B0**%253A+%25D0%25B2%25D1%258B%25D1%2587%25D0%25B8%25D1%2581%25D0%25BB%25D1%258F%25D1%258E+%25D0%25B2%25D1%258B%25D1%2580%25D0%25B0%25D0%25B6%25D0%25B5%25D0%25BD%25D0%25B8%25D1%258F+%25E2%2580%2594+%25D0%25BD%25D0%25B0%25D0%25BF%25D1%2580%25D0%25B8%25D0%25BC%25D0%25B5%25D1%2580%252C+%25C2%25AB%25D0%25A1%25D0%25BA%25D0%25BE%25D0%25BB%25D1%258C%25D0%25BA%25D0%25BE+%25D0%25B1%25D1%2583%25D0%25B4%25D0%25B5%25D1%2582+2+%252B+2%253F%25C2%25BB%250A+++-+**%25D0%259F%25D0%25B5%25D1%2580%25D0%25B5%25D0%25B2%25D0%25BE%25D0%25B4**%253A+%25D0%25BF%25D0%25B5%25D1%2580%25D0%25B5%25D0%25B2%25D0%25BE%25D0%25B6%25D1%2583+%25D1%2584%25D1%2580%25D0%25B0%25D0%25B7%25D1%258B+%25D0%25BC%25D0%25B5%25D0%25B6%25D0%25B4%25D1%2583+%25D1%258F%25D0%25B7%25D1%258B%25D0%25BA%25D0%25B0%25D0%25BC%25D0%25B8.%250A+++-+**%25D0%259F%25D0%25B0%25D0%25BC%25D1%258F%25D1%2582%25D1%258C**%253A+%25D0%25BF%25D0%25BE%25D0%25BC%25D0%25BD%25D1%258E+%25D0%25BA%25D0%25BE%25D0%25BD%25D1%2582%25D0%25B5%25D0%25BA%25D1%2581%25D1%2582+%25D1%2580%25D0%25B0%25D0%25B7%25D0%25B3%25D0%25BE%25D0%25B2%25D0%25BE%25D1%2580%25D0%25B0+%25D0%25B2+%25D1%2580%25D0%25B0%25D0%25BC%25D0%25BA%25D0%25B0%25D1%2585+%25D1%2581%25D0%25B5%25D1%2581%25D1%2581%25D0%25B8%25D0%25B8.%250A+++%250A+++%25D0%25AF+%25D1%2580%25D0%25B0%25D0%25B1%25D0%25BE%25D1%2582%25D0%25B0%25D1%258E+%25D0%25BD%25D0%25B0+%25D0%25BE%25D1%2581%25D0%25BD%25D0%25BE%25D0%25B2%25D0%25B5+%25D0%25BB%25D0%25BE%25D0%25BA%25D0%25B0%25D0%25BB%25D1%258C%25D0%25BD%25D1%258B%25D1%2585+%25D1%2581%25D0%25B8%25D0%25BC%25D0%25B2%25D0%25BE%25D0%25BB%25D1%258C%25D0%25BD%25D1%258B%25D1%2585+%25D0%25BF%25D1%2580%25D0%25B0%25D0%25B2%25D0%25B8%25D0%25BB%252C+%25D0%25B1%25D0%25B5%25D0%25B7+%25D0%25BD%25D0%25B5%25D0%25B9%25D1%2580%25D0%25BE%25D1%2581%25D0%25B5%25D1%2582%25D0%25B5%25D0%25B2%25D0%25BE%25D0%25B3%25D0%25BE+%25D0%25B8%25D0%25BD%25D1%2584%25D0%25B5%25D1%2580%25D0%25B5%25D0%25BD%25D1%2581%25D0%25B0.%25

<img width="1280" height="722" alt="Image" src="https://github.com/user-attachments/assets/5e84d08d-fcb9-4261-83ff-b9b2a86d06a3" />

