# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: issue-282.spec.js >> Issue #282 Rust/WASM worker parity >> English unknown prompts use the native Rust stable-id opener
- Location: tests/issue-282.spec.js:65:5

# Error details

```
Error: expect(locator).toHaveCount(expected) failed

Locator:  locator('[data-testid="chat-message"]')
Expected: 2
Received: 1
Timeout:  20000ms

Call log:
  - Expect "toHaveCount" with timeout 20000ms
  - waiting for locator('[data-testid="chat-message"]')
    44 × locator resolved to 1 element
       - unexpected value "1"

```

# Page snapshot

```yaml
- main [ref=e2]:
  - generic [ref=e3]:
    - button "Collapse side panel" [pressed] [ref=e4] [cursor=pointer]:
      - generic [ref=e5]: ◀
    - generic [ref=e6]:
      - generic [ref=e7]: FA
      - strong [ref=e8]: formal-ai
      - generic [ref=e9]: vdev
    - generic [ref=e10]:
      - status [ref=e11]: Manual mode
      - status [ref=e12]: "Mode: Chat"
      - generic [ref=e13]: wasm worker
      - link "Source code" [ref=e14] [cursor=pointer]:
        - /url: https://github.com/link-assistant/formal-ai
        - img [ref=e16]
      - link "Download" [ref=e20] [cursor=pointer]:
        - /url: download/
        - img [ref=e22]
      - link "Report issue" [ref=e24] [cursor=pointer]:
        - /url: https://github.com/link-assistant/formal-ai/issues/new?title=Issue+with+dialog%3A+blorfblarf&body=%23%23+Environment%0A%0A-+**Version**%3A+dev+%28wasm%29%0A-+**URL**%3A+http%3A%2F%2Flocalhost%3A3456%2Fapp%2F%0A-+**Diagnostics**%3A+on%0A-+**Timestamp**%3A+2026-07-19T07%3A23%3A52.316Z%0A%0A%23%23+User+Context%0A%0A-+**UI+languages**%3A+*en-US*%0A-+**UI**%3A+1280x720+viewport%2C+1280x720+%401x+screen%2C+Mozilla%2F5.0+%28Windows+NT+10.0%3B+Win64%3B+x64%29+AppleWebKit%2F537.36+%28KHTML%2C+like+Gecko%29+Chrome%2F148.0.7778.96+Safari%2F537.36+browser%2C+Windows+platform%0A-+**Locale**%3A+en-US+%28UTC%29%0A%0A%23%23+Reproduction+of+dialog%0A%0ALegend%3A+%60U%60+%3D+user%2C+%60A%60+%3D+agent.%0A%0A%60%60%60%0AU%3A+blorfblarf%0A%60%60%60%0A%0A%23%23+Description%0A%0A%3C%21--+Please+describe+what+looked+wrong+or+incomplete.+--%3E%0A%0A%23%23+Attach+full+memory+%28optional%29%0A%0AClick+**Export+memory**+to+save+%60formal-ai-memory.lino%60%2C+redact+it%2C+and+attach+it+%28as+a+%60.zip%60+if+needed%29.+See+the+%5Bupload-memory+guide%5D%28https%3A%2F%2Fgithub.com%2Flink-assistant%2Fformal-ai%2Fblob%2Fmain%2Fdocs%2Fupload-memory.md%29.%0A&labels=bug
        - img [ref=e26]
      - button "Export memory" [ref=e32] [cursor=pointer]:
        - img [ref=e34]
      - button "Import memory" [ref=e38] [cursor=pointer]:
        - img [ref=e40]
      - button "Reset memory" [ref=e44] [cursor=pointer]:
        - img [ref=e46]
      - button "Diagnostics on" [pressed] [ref=e51] [cursor=pointer]:
        - img [ref=e53]
      - radiogroup "Operating mode - choose Chat, Agent, or Full Auto." [ref=e55]:
        - radio "Chat" [checked] [ref=e56] [cursor=pointer]:
          - img [ref=e58]
        - radio "Agent" [ref=e60] [cursor=pointer]:
          - img [ref=e62]
        - radio "Full Auto" [ref=e64] [cursor=pointer]:
          - img [ref=e66]
      - button "Demo" [ref=e68] [cursor=pointer]:
        - img [ref=e70]
  - generic [ref=e76]:
    - complementary [ref=e77]:
      - generic [ref=e79] [cursor=pointer]:
        - button "Menu" [ref=e80]:
          - generic [ref=e81]: ▶
          - heading "Menu" [level=2] [ref=e82]
        - button "Expand only this section" [ref=e83]:
          - img [ref=e85]
      - generic [ref=e94]:
        - generic [ref=e95] [cursor=pointer]:
          - button "Conversations" [expanded] [ref=e96]:
            - generic [ref=e97]: ▼
            - heading "Conversations" [level=2] [ref=e98]
          - button "Expand only this section" [ref=e99]:
            - img [ref=e101]
        - generic [ref=e111]:
          - button "+ New conversation" [ref=e112] [cursor=pointer]
          - generic [ref=e113]:
            - checkbox "Show deleted conversations" [ref=e114]
            - generic [ref=e115]: Show deleted conversations
          - paragraph [ref=e116]: Start a new conversation.
      - generic [ref=e117]:
        - generic [ref=e118] [cursor=pointer]:
          - button "Settings" [expanded] [ref=e119]:
            - generic [ref=e120]: ▼
            - heading "Settings" [level=2] [ref=e121]
          - button "Expand only this section" [ref=e122]:
            - img [ref=e124]
        - generic [ref=e134]:
          - generic [ref=e135]:
            - generic [ref=e136]:
              - generic [ref=e137]: Reset settings
              - button "Reset all" [ref=e138] [cursor=pointer]
            - list [ref=e139]:
              - listitem [ref=e140]:
                - generic [ref=e141]: Greeting variations
                - button "Reset" [ref=e142] [cursor=pointer]
          - generic [ref=e143]:
            - generic [ref=e144]: Ambiguity
            - generic [ref=e145]:
              - generic [ref=e146]: More questions
              - generic [ref=e147]: More guessing
            - slider "Ambiguity" [ref=e148]: "0.8"
            - status [ref=e149]: 80%
          - generic [ref=e150]:
            - generic [ref=e151]: Follow-up initiative
            - generic [ref=e152]:
              - generic [ref=e153]: User initiative
              - generic [ref=e154]: Assistant proposes
            - slider "Follow-up initiative" [ref=e155]: "0.75"
            - status [ref=e156]: 75%
          - generic [ref=e157]:
            - generic [ref=e158]: Temperature
            - generic [ref=e159]:
              - generic [ref=e160]: Deterministic
              - generic [ref=e161]: Varied
            - slider "Temperature" [ref=e162]: "0.7"
            - status [ref=e163]: "0.70"
          - generic [ref=e164]:
            - checkbox "Greeting variations" [ref=e165]
            - generic [ref=e166]: Greeting variations
          - generic [ref=e167]:
            - generic [ref=e168]: Definition fusion
            - combobox "Definition fusion" [ref=e169]:
              - option "Explicit only" [selected]
              - option "Auto for definitions"
          - generic [ref=e170]:
            - generic [ref=e171]: Program composition
            - combobox "Program composition" [ref=e172]:
              - option "Composed (projected)" [selected]
              - option "Documented (full)"
          - generic [ref=e173]:
            - generic [ref=e174]: Thinking detail
            - combobox "Thinking detail" [ref=e175]:
              - option "Brief"
              - option "Standard" [selected]
              - option "Detailed"
          - generic [ref=e176]:
            - generic [ref=e177]: Minimum thinking animation
            - generic [ref=e178]:
              - generic [ref=e179]: Immediate
              - generic [ref=e180]: Relaxed
            - slider "Minimum thinking animation" [ref=e181]: "2000"
            - status [ref=e182]: 2.0s
          - generic [ref=e183]:
            - generic [ref=e184]:
              - checkbox "Experimental OCR" [ref=e185]
              - generic [ref=e186]: Experimental OCR
            - paragraph [ref=e187]: "Downloads about 6 MB on first use: OCR wrapper, worker, WebAssembly core, and English traineddata."
          - generic [ref=e188]:
            - paragraph [ref=e189]: External trusted services
            - paragraph [ref=e190]: Choose which trusted services the assistant may consult for procedural how-to answers, project lookups, and document originality / fact-check verification. Each one is on by default; turn off any you would rather not contact.
            - generic [ref=e191]:
              - checkbox "wikiHow procedures" [checked] [ref=e192]
              - generic [ref=e193]: wikiHow procedures
            - generic [ref=e194]:
              - checkbox "Stack Exchange answers" [checked] [ref=e195]
              - generic [ref=e196]: Stack Exchange answers
            - generic [ref=e197]:
              - checkbox "Wikibooks, Wikiversity & Wikivoyage" [checked] [ref=e198]
              - generic [ref=e199]: Wikibooks, Wikiversity & Wikivoyage
            - generic [ref=e200]:
              - checkbox "GitHub project docs" [checked] [ref=e201]
              - generic [ref=e202]: GitHub project docs
          - generic [ref=e203]:
            - generic [ref=e204]: Language
            - combobox "Language" [ref=e205]:
              - option "Auto" [selected]
              - option "English"
              - option "Русский"
              - option "中文"
              - option "हिन्दी"
          - generic [ref=e206]:
            - generic [ref=e207]: Response language
            - combobox "Response language" [ref=e208]:
              - option "Last message language" [selected]
              - option "Preferred language"
              - option "UI language"
          - generic [ref=e209]:
            - generic [ref=e210]: Theme
            - combobox "Theme" [ref=e211]:
              - option "Auto" [selected]
              - option "Light"
              - option "Dark"
          - generic [ref=e212]:
            - generic [ref=e213]: UI skin
            - combobox "UI skin" [ref=e214]:
              - option "Flat" [selected]
              - option "Glass"
              - option "Contrast"
          - generic [ref=e215]:
            - generic [ref=e216]: Toolbar icons
            - combobox "Toolbar icons" [ref=e217]:
              - option "Font Awesome" [selected]
              - option "Material Symbols"
              - option "Bootstrap Icons"
              - option "Ionicons"
              - option "Remix Icon"
              - option "Tabler Icons"
              - option "Names"
          - generic [ref=e218]:
            - generic [ref=e219]: Chat style
            - combobox "Chat style" [ref=e220]:
              - option "Cards" [selected]
              - option "Compact"
              - option "Bubbles"
          - generic [ref=e221]:
            - generic [ref=e222]: Input style
            - combobox "Input style" [ref=e223]:
              - option "Flat" [selected]
              - option "Glass soft"
              - option "Glass clear"
              - option "Bubble"
          - generic [ref=e224]:
            - generic [ref=e225]: Input action
            - combobox "Input action" [ref=e226]:
              - option "Attach" [selected]
              - option "Plus"
          - generic [ref=e227]:
            - generic [ref=e228]: Assistant name
            - textbox "Assistant name" [ref=e229]:
              - /placeholder: Not set
          - generic [ref=e230]:
            - generic [ref=e231]: Location
            - textbox "Location" [ref=e232]:
              - /placeholder: City or region
      - generic [ref=e233]:
        - generic [ref=e234] [cursor=pointer]:
          - button "Example prompts" [expanded] [ref=e235]:
            - generic [ref=e236]: ▼
            - heading "Example prompts" [level=2] [ref=e237]
          - button "Expand only this section" [ref=e238]:
            - img [ref=e240]
        - generic [ref=e250]:
          - button "Hi" [ref=e251] [cursor=pointer]
          - button "Привет" [ref=e252] [cursor=pointer]
          - button "नमस्ते" [ref=e253] [cursor=pointer]
          - button "你好" [ref=e254] [cursor=pointer]
          - button "Goodbye" [ref=e255] [cursor=pointer]
          - button "До свидания" [ref=e256] [cursor=pointer]
          - button "अलविदा" [ref=e257] [cursor=pointer]
          - button "再见" [ref=e258] [cursor=pointer]
          - button "Who are you?" [ref=e259] [cursor=pointer]
          - button "Кто ты?" [ref=e260] [cursor=pointer]
          - button "तुम कौन हो?" [ref=e261] [cursor=pointer]
          - button "你是谁?" [ref=e262] [cursor=pointer]
          - button "I don't understand" [ref=e263] [cursor=pointer]
          - button "не понял" [ref=e264] [cursor=pointer]
          - button "समझ नहीं आया" [ref=e265] [cursor=pointer]
          - button "我不明白" [ref=e266] [cursor=pointer]
          - button "What can you do?" [ref=e267] [cursor=pointer]
          - button "Что ты умеешь?" [ref=e268] [cursor=pointer]
          - button "List behavior rules" [ref=e269] [cursor=pointer]
          - button "List all facts you know about yourself" [ref=e270] [cursor=pointer]
          - button "Write me hello world program in Rust" [ref=e271] [cursor=pointer]
          - button "Create a hello world example in Python" [ref=e272] [cursor=pointer]
          - button "Write hello world in JavaScript" [ref=e273] [cursor=pointer]
          - button "Write hello world in TypeScript" [ref=e274] [cursor=pointer]
          - button "Show hello world in Go" [ref=e275] [cursor=pointer]
          - button "Show hello world in C" [ref=e276] [cursor=pointer]
          - button "What is 2 + 2?" [ref=e277] [cursor=pointer]
          - button "Сколько будет два плюс два?" [ref=e278] [cursor=pointer]
          - button "What is Rust?" [ref=e279] [cursor=pointer]
          - button "Who is Donald Trump?" [ref=e280] [cursor=pointer]
          - button "Кто такой Илон Маск?" [ref=e281] [cursor=pointer]
          - button "Что такое Википедия?" [ref=e282] [cursor=pointer]
          - button "विकिपीडिया क्या है?" [ref=e283] [cursor=pointer]
          - button "维基百科是什么?" [ref=e284] [cursor=pointer]
          - button "What is IIR in machine learning?" [ref=e285] [cursor=pointer]
          - button "Summarize this conversation" [ref=e286] [cursor=pointer]
          - button "Brainstorm 5 small tools for link notation." [ref=e287] [cursor=pointer]
          - button "Who wrote The Lord of the Rings?" [ref=e288] [cursor=pointer]
          - button "столица россии" [ref=e289] [cursor=pointer]
          - button "जापान की राजधानी क्या है?" [ref=e290] [cursor=pointer]
          - button "日本的首都是什么?" [ref=e291] [cursor=pointer]
          - button "Navigate to github.com" [ref=e292] [cursor=pointer]
          - button "Сделай запрос к google.com" [ref=e293] [cursor=pointer]
          - button "Search the web for Nikola Tesla" [ref=e294] [cursor=pointer]
          - button "What features make it different from C?" [ref=e295] [cursor=pointer]
          - button "Pretend you are Albert Einstein and explain relativity to a teenager." [ref=e296] [cursor=pointer]
          - button "Купи слона" [ref=e297] [cursor=pointer]
          - button "When did I ask about Rust?" [ref=e298] [cursor=pointer]
          - button "Find Wikipedia in another conversation" [ref=e299] [cursor=pointer]
          - button "Export memory" [ref=e300] [cursor=pointer]
          - button "Import memory" [ref=e301] [cursor=pointer]
      - generic [ref=e302]:
        - generic [ref=e303] [cursor=pointer]:
          - button "Tools" [expanded] [ref=e304]:
            - generic [ref=e305]: ▼
            - heading "Tools" [level=2] [ref=e306]
          - button "Expand only this section" [ref=e307]:
            - img [ref=e309]
        - list [ref=e320]:
          - listitem [ref=e321]:
            - generic [ref=e322]:
              - strong [ref=e323]: http_fetch
              - generic [ref=e324]: thinking
            - paragraph [ref=e325]: Issue an HTTP GET request from the current environment and return body, status, and headers; when CORS blocks the response, check frame-policy metadata before using an iframe preview.
          - listitem [ref=e326]:
            - generic [ref=e327]:
              - strong [ref=e328]: url_navigate
              - generic [ref=e329]: thinking
            - paragraph [ref=e330]: Normalize a requested URL, check CORS-readable frame-policy metadata, and return either an iframe preview or a direct external HTTPS link.
          - listitem [ref=e331]:
            - generic [ref=e332]:
              - strong [ref=e333]: web_search
              - generic [ref=e334]: thinking
            - paragraph [ref=e335]: Search the open web through the configured search provider and return ranked result links.
          - listitem [ref=e336]:
            - generic [ref=e337]:
              - strong [ref=e338]: wikipedia_lookup
              - generic [ref=e339]: thinking
            - paragraph [ref=e340]: Fetch a structured Wikipedia REST summary for the given title in the detected language, falling back to English.
          - listitem [ref=e341]:
            - generic [ref=e342]:
              - strong [ref=e343]: calculator
              - generic [ref=e344]: thinking
            - paragraph [ref=e345]: Evaluate calculator-parsable math, unit, currency, percentage, and datetime expressions through link-calculator.
          - listitem [ref=e346]:
            - generic [ref=e347]:
              - strong [ref=e348]: eval_js
              - generic [ref=e349]: agent
            - paragraph [ref=e350]: Evaluate a JavaScript snippet inside the browser worker sandbox with no DOM and no network.
          - listitem [ref=e351]:
            - generic [ref=e352]:
              - strong [ref=e353]: read_local_file
              - generic [ref=e354]: thinking
            - paragraph [ref=e355]: Read a file the user selected through the browser file picker without touching the host filesystem.
          - listitem [ref=e356]:
            - generic [ref=e357]:
              - strong [ref=e358]: append_memory
              - generic [ref=e359]: agent
            - paragraph [ref=e360]: Append a single event to the AI append-only memory log in Links Notation.
          - listitem [ref=e361]:
            - generic [ref=e362]:
              - strong [ref=e363]: export_memory
              - generic [ref=e364]: thinking
            - paragraph [ref=e365]: Serialize the full memory bundle as Links Notation text for download or migration.
          - listitem [ref=e366]:
            - generic [ref=e367]:
              - strong [ref=e368]: import_memory
              - generic [ref=e369]: thinking
            - paragraph [ref=e370]: Import a demo_memory log or full formal_ai_bundle from a user-selected .lino file.
          - listitem [ref=e371]:
            - generic [ref=e372]:
              - strong [ref=e373]: conversation_recall
              - generic [ref=e374]: thinking
            - paragraph [ref=e375]: Search the append-only memory log for prior mentions and group matches by conversation.
          - listitem [ref=e376]:
            - generic [ref=e377]:
              - strong [ref=e378]: concept_lookup
              - generic [ref=e379]: thinking
            - paragraph [ref=e380]: Resolve a seeded concept, alias, or Wikidata-backed entity to a grounded summary.
          - listitem [ref=e381]:
            - generic [ref=e382]:
              - strong [ref=e383]: write_program
              - generic [ref=e384]: thinking
            - paragraph [ref=e385]: Render seeded programs from language and task parameters.
          - listitem [ref=e386]:
            - generic [ref=e387]:
              - strong [ref=e388]: intent_routing
              - generic [ref=e389]: thinking
            - paragraph [ref=e390]: Route normalized prompts through seeded multilingual keywords, phrases, tokens, and combos.
          - listitem [ref=e391]:
            - generic [ref=e392]:
              - strong [ref=e393]: fact_lookup
              - generic [ref=e394]: thinking
            - paragraph [ref=e395]: Answer seeded factual questions with localized text and Wikidata evidence anchors.
          - listitem [ref=e396]:
            - generic [ref=e397]:
              - strong [ref=e398]: summarize_conversation
              - generic [ref=e399]: thinking
            - paragraph [ref=e400]: Summarize the current conversation from recorded user and assistant turns.
          - listitem [ref=e401]:
            - generic [ref=e402]:
              - strong [ref=e403]: brainstorm
              - generic [ref=e404]: thinking
            - paragraph [ref=e405]: Generate seeded idea lists and names for common brainstorming prompts.
          - listitem [ref=e406]:
            - generic [ref=e407]:
              - strong [ref=e408]: coreference
              - generic [ref=e409]: thinking
            - paragraph [ref=e410]: Resolve follow-up pronouns against previous conversation turns before answering.
          - listitem [ref=e411]:
            - generic [ref=e412]:
              - strong [ref=e413]: roleplay
              - generic [ref=e414]: thinking
            - paragraph [ref=e415]: Respond from grounded persona templates while preserving deterministic symbolic behavior.
      - generic [ref=e416]:
        - generic [ref=e417] [cursor=pointer]:
          - button "Trace" [expanded] [ref=e418]:
            - generic [ref=e419]: ▼
            - heading "Trace" [level=2] [ref=e420]
          - button "Expand only this section" [ref=e421]:
            - img [ref=e423]
        - generic [ref=e433]:
          - generic [ref=e434]:
            - term [ref=e435]: Model
            - definition [ref=e436]: formal-ai
          - generic [ref=e437]:
            - term [ref=e438]: Mode
            - definition [ref=e439]: Manual mode
          - generic [ref=e440]:
            - term [ref=e441]: Intent
            - definition [ref=e442]: none
          - generic [ref=e443]:
            - term [ref=e444]: Data
            - definition [ref=e445]: data/source-index.lino
          - generic [ref=e446]:
            - term [ref=e447]: Seed files
            - definition [ref=e448]: seed/agent-info.lino, seed/interface-capabilities.lino, seed/multilingual-responses.lino, seed/concepts.lino, seed/concept-contexts.lino, seed/facts.lino, seed/projects.lino, seed/brainstorm-seeds.lino, seed/personas.lino, seed/summary-topics.lino, seed/coreference.lino, seed/tools.lino, seed/language-detection.lino, seed/prompt-patterns.lino, seed/intent-routing.lino, seed/operation-vocabulary.lino, seed/numeric-list-operations.lino, seed/coding-idioms.lino, seed/terminal-commands.lino, seed/shell-intents.lino, seed/program-plan-rules.lino, seed/market-price-references.lino, seed/meanings.lino, seed/meanings-behavior-rules.lino, seed/meanings-calculator.lino, seed/meanings-calendar.lino, seed/meanings-coding-catalog.lino, seed/meanings-conversation.lino, seed/meanings-definition-merge.lino, seed/meanings-docs.lino, seed/meanings-facts.lino, seed/meanings-feature-capability.lino, seed/meanings-file-write.lino, seed/meanings-file-edit.lino, seed/meanings-agent-actions.lino, seed/meanings-finance.lino, seed/meanings-how.lino, seed/meanings-intent.lino, seed/meanings-lexical-meta.lino, seed/meanings-links-root.lino, seed/meanings-meta.lino, seed/meanings-ontology.lino, seed/meanings-playwright.lino, seed/meanings-policy.lino, seed/meanings-program-synthesis.lino, seed/meanings-proof.lino, seed/meanings-research-table.lino, seed/meanings-semantic-meta.lino, seed/meanings-skill-compiler.lino, seed/meanings-software-project.lino, seed/meanings-summary.lino, seed/meanings-tool-access.lino, seed/meanings-translation.lino, seed/meanings-units.lino, seed/meanings-web-followup.lino, seed/meanings-web-navigation.lino, seed/meanings-web-research.lino, seed/meanings-web-search-query.lino, seed/meanings-web-search.lino, seed/meanings-wikidata.lino, seed/greetings.lino, seed/identity.lino, seed/hello-world-programs.lino, seed/self-improvement-loop.lino, seed/demo-dialogs.lino, seed/environments.lino
          - generic [ref=e449]:
            - term [ref=e450]: Tools loaded
            - definition [ref=e451]: "19"
          - generic [ref=e452]:
            - term [ref=e453]: Concepts loaded
            - definition [ref=e454]: "24"
    - separator "Resize the side panel." [ref=e455]
    - generic [ref=e456]:
      - generic [ref=e457]:
        - article [ref=e458]:
          - generic [ref=e459]: "Y"
          - generic [ref=e460]:
            - generic [ref=e461]:
              - strong [ref=e462]: You
              - time [ref=e463]: 07:23 AM
              - button "Copy the whole message as Markdown" [ref=e464] [cursor=pointer]:
                - generic [ref=e465]: Copy as Markdown
            - paragraph [ref=e467]: blorfblarf
        - article [ref=e468]:
          - generic [ref=e469]: FA
          - region "Thinking" [ref=e471]:
            - generic [ref=e472]:
              - strong [ref=e473]: Thinking
              - button "Expand" [ref=e475] [cursor=pointer]
            - generic [ref=e476]:
              - paragraph [ref=e477]: Composing the answer in natural language.
              - paragraph [ref=e478]: Working through the request.
      - generic [ref=e480]:
        - button "Composer menu" [ref=e481] [cursor=pointer]:
          - img [ref=e483]
        - textbox "Message formal-ai" [ref=e486]
        - button "Sending..." [disabled] [ref=e487]:
          - generic [ref=e489]: Sending...
```

# Test source

```ts
  1  | // @ts-check
  2  | const { test, expect } = require('@playwright/test');
  3  | 
  4  | const wasmParityCases = [
  5  |   {
  6  |     language: 'en',
  7  |     name: 'English',
  8  |     prompt: 'blorfblarf',
  9  |     expected: "I'm not sure how to respond to that yet.",
  10 |   },
  11 |   {
  12 |     language: 'ru',
  13 |     name: 'Russian',
  14 |     prompt: 'неведомослово',
  15 |     expected: 'Я ещё не научился отвечать на это.',
  16 |     forbidden: 'Я тебя не понял.',
  17 |   },
  18 |   {
  19 |     language: 'hi',
  20 |     name: 'Hindi',
  21 |     prompt: 'अज्ञातशब्द',
  22 |     expected: 'मैं समझ नहीं पाया।',
  23 |   },
  24 |   {
  25 |     language: 'zh',
  26 |     name: 'Chinese',
  27 |     prompt: '未知词',
  28 |     expected: '我不太明白你说的意思。',
  29 |   },
  30 | ];
  31 | 
  32 | async function sendPrompt(page, text) {
  33 |   const input = page.locator('[data-testid="chat-composer-input"]');
  34 |   await expect(input).toBeEnabled({ timeout: 5_000 });
  35 |   await input.fill(text);
  36 | 
  37 |   const messages = page.locator('[data-testid="chat-message"]');
  38 |   const initialCount = await messages.count();
  39 |   await page.locator('[data-testid="chat-composer-submit"]').click();
> 40 |   await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });
     |                          ^ Error: expect(locator).toHaveCount(expected) failed
  41 |   const lastMessage = messages.last();
  42 |   await expect(lastMessage).toHaveClass(/assistant/);
  43 |   const body = lastMessage.locator('.markdown-body');
  44 |   await expect(body).toBeVisible();
  45 |   return body;
  46 | }
  47 | 
  48 | test.describe('Issue #282 Rust/WASM worker parity', () => {
  49 |   test.beforeEach(async ({ page }) => {
  50 |     await page.addInitScript(() => {
  51 |       window.localStorage.setItem(
  52 |         'formal-ai.preferences.v1',
  53 |         'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
  54 |       );
  55 |     });
  56 |     await page.goto('./');
  57 |     await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  58 |     await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  59 |     await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
  60 |       timeout: 5_000,
  61 |     });
  62 |   });
  63 | 
  64 |   for (const { language, name, prompt, expected, forbidden } of wasmParityCases) {
  65 |     test(`${name} unknown prompts use the native Rust stable-id opener`, async ({ page }) => {
  66 |       await expect(page.locator('.status')).toContainText('wasm worker');
  67 | 
  68 |       const reply = await sendPrompt(page, prompt);
  69 |       await expect(reply, `${language} opener should match native Rust`).toContainText(expected);
  70 |       if (forbidden) {
  71 |         await expect(reply).not.toContainText(forbidden);
  72 |       }
  73 |     });
  74 |   }
  75 | });
  76 | 
```