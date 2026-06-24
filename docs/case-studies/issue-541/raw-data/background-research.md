# Background Research: UI/UX Issues for "formal-ai" (Electron + no-build React)

> Compiled via online research (WebSearch/WebFetch) on 2026-06-20 to support the
> issue-541 case study. Every finding carries a source URL.

## 1. Electron persistent storage & data migration

- **Per-OS user-data location.** `app.getPath('userData')` is the idiomatic place to store app data. It defaults to the `appData` directory + the app name and resolves to: **macOS** `~/Library/Application Support/<AppName>`, **Windows** `%APPDATA%\<AppName>`, **Linux** `$XDG_CONFIG_HOME/<AppName>` (or `~/.config/<AppName>` if `XDG_CONFIG_HOME` is unset). The OS-designated directory is preferred because auto-updates may move/delete app source files and writing into the app bundle invalidates the code signature. Electron docs also warn to create a subdirectory rather than write directly into `userData`, since Chromium uses sibling dirs like `Cache`, `GPUCache`, and `Local Storage`. Sources: https://www.electronjs.org/docs/latest/api/app , https://medium.com/cameron-nokes/how-to-store-user-data-in-electron-3ba6bf66bc1e

- **Where IndexedDB lives / how it gets wiped.** A renderer's persistent storage lives under the session's partition. When a `<webview>`/window `partition` starts with `persist:`, the session is persisted under `app.getPath('userData')/Partitions/<name>`; without the prefix it is in-memory only (so changing or dropping the partition string effectively orphans/discards prior IndexedDB data). The default session stores IndexedDB inside `userData` (e.g. an `IndexedDB` LevelDB dir). Sources: https://www.electronjs.org/docs/latest/api/session , https://www.electronjs.org/docs/latest/api/app

- **IndexedDB loss across updates (real, reported bug).** IndexedDB data has been reported deleted after Electron upgrades — e.g. updating Electron 23.1.4 -> 23.2.4 wiped IndexedDB even though it was an upgrade, not a downgrade. Source: https://github.com/electron/electron/issues/38616

- **Chromium wipes data on version *downgrade* by design.** Chromium does not allow storage-format downgrades; if an app built on Electron v9 (newer Chromium) is later rebuilt on v8 (older Chromium), opening it erases IndexedDB contents. Relevant because rollback/downgrade of an Electron app can silently destroy local user data. Source: https://github.com/electron/electron/issues/24882

- **Versioned schema migration pattern.** The canonical approach for local user data is `electron-store` (sindresorhus), which persists JSON in `userData` and supports a `migrations` map of `{'version': handler}` (semver ranges allowed) run automatically when the app version increases, plus a JSON `schema` option for validation. Maintainers flag the migration feature as having known bugs, so migrations should be tested. Sources: https://github.com/sindresorhus/electron-store/blob/main/readme.md , https://github.com/sindresorhus/electron-store/issues/108

## 2. Chakra UI feasibility for a no-build React app

- **A build step is effectively required.** Chakra's official install path targets framework + bundler setups (Vite, Next.js); all guidance is npm-based with a `<Provider>` wrapper and JSX. The docs make **no mention of CDN or `<script>`-tag (UMD) usage** — it ships ESM and assumes a React JSX build pipeline. Sources: https://chakra-ui.com/docs/get-started/installation , https://chakra-ui.com/docs/get-started/frameworks/vite

- **Peer dependencies are CSS-in-JS heavy.** Chakra is built on Emotion. v3 installs `@chakra-ui/react @emotion/react`; v2 additionally requires `@emotion/styled` and `framer-motion`. These runtime peers (Emotion + framer-motion) are themselves ESM packages that normally need bundling. Sources: https://v2.chakra-ui.com/getting-started , https://chakra-ui.com/docs/get-started/installation

- **Conclusion for a no-build (hyperscript/CDN) app.** *(Inference)* Because Chakra depends on JSX compilation and bundled ESM peers (Emotion CSS-in-JS, framer-motion) and publishes no CDN/UMD build, it is not a practical fit for a no-build, hyperscript Electron+React app. The lighter-weight, bundler-free alternative is plain **CSS custom properties as design tokens** (see §3): no JS runtime, no Emotion, themeable in one place. Supporting docs: https://chakra-ui.com/docs/get-started/installation

## 3. CSS theming best practice (design tokens / custom properties for light/dark)

- **Three-tier token layering.** Separate **global primitives** (`--color-blue-600: #0052CC`), **semantic tokens** (`--color-bg-surface`, `--color-text-primary` mapped to primitives), and optional **component tokens**. Components consume *only* semantic tokens, so switching themes never requires touching components and no element is missed. Source: https://penpot.app/blog/the-developers-guide-to-design-tokens-and-css-variables/

- **Define once, override per theme.** Declare semantic tokens on `:root`, then re-declare only those tokens inside a `@media (prefers-color-scheme: dark)` block (or a `[data-theme="dark"]` selector). Because all elements reference the same variables, a single override flips the entire UI. Source: https://www.scale.at/blog/css-custom-properties

- **`color-scheme` adapts the UA chrome.** Per MDN, `color-scheme` tells the browser which schemes an element supports; the UA then matches the canvas surface, scrollbars, and form controls to the active scheme. `:root { color-scheme: light dark; }` prevents mismatched native controls/scrollbars in dark mode. Source: https://developer.mozilla.org/en-US/docs/Web/CSS/color-scheme

- **`prefers-color-scheme` for everything else.** MDN: component authors must use the `prefers-color-scheme` media feature to style the rest of the elements; the newer `light-dark()` color function is a compact alternative. Source: https://developer.mozilla.org/en-US/docs/Web/CSS/color-scheme

## 4. Chat "thinking"/reasoning UX animation timing

- **Nielsen response-time limits (1993).** **0.1 s** = feels instantaneous, no special feedback needed. **1.0 s** = the limit for the user's flow of thought to stay uninterrupted (delay noticed). **10 s** = the limit for keeping attention on the dialogue; above this, show a percent-done indicator. Source: https://www.nngroup.com/articles/response-times-3-important-limits/

- **Streaming is the baseline for AI chat.** Token-by-token streaming is expected; waiting for completion "feels broken." Show a visible streaming indicator plus a stop affordance. Source: https://dev.to/greedy_reader/ai-chat-ui-best-practices-designing-better-llm-interfaces-18jj

- **Concrete "thinking" affordances.** Combine an animated icon, a dynamic/changing text label, and a counter; keeping a short readable reasoning snippet visible gives the user something to read while the model thinks. Sources: https://fuselabcreative.com/chatbot-interface-design-guide/ , https://www.digestibleux.com/p/how-ai-models-show-their-reasoning

## 5. Reasoning-step humanization

- **Show a human-readable summary, not raw reasoning.** OpenAI returns a natural-language reasoning *summary* (`summary: "auto"/"detailed"`) rather than raw tokens — establishing the pattern of summarized chain-of-thought for display. Source: https://developers.openai.com/api/docs/guides/reasoning

- **Default to concise + progressively disclose.** "More transparency != better UX": excessive detail causes overload. Show high-level reasoning by default as bullets/collapsible sections, with an affordance to expand the full trace. ChatGPT keeps reasoning short/collapsed; Claude hides it and uses bullets; DeepSeek's unstructured auto-scrolling dump is cited as overwhelming. Source: https://www.digestibleux.com/p/how-ai-models-show-their-reasoning

- **Layered progressive disclosure.** Surface reasoning in tiers (confidence -> expandable explanation -> full trace), each hidden by default. Source: https://www.aiuxdesign.guide/patterns/progressive-disclosure

- **Faithfulness caveat.** Anthropic research finds displayed chain-of-thought may be a plausible post-hoc rationalization rather than the true internal reasoning, so humanized steps should be framed as an explanation. Source: https://assets.anthropic.com/m/71876fabef0f0ed4/original/reasoning_models_paper.pdf
