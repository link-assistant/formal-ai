# Competitor Test Research: AI Models, Agentic CLI Tools, and Benchmark Suites

A reference summary for expanding the `formal-ai` prompt-coverage tests.
Focus: categories, example prompts, multilingual coverage (EN/RU/HI/ZH), and Rust testing tools.

---

## 1. Agentic CLI Tools — Public Demos, Eval Suites, Prompt Categories

These tools rarely publish formal "prompt benchmarks", but their docs, demo videos, example galleries, and SWE-Bench / HumanEval scores reveal the test categories they care about.

### Claude Code (Anthropic)
Source: https://docs.claude.com/en/docs/claude-code, https://github.com/anthropics/claude-code
Categories shown in docs / demos:
- Codebase exploration ("Explain this repo", "What does file X do?")
- Code editing & refactor ("Add error handling to function foo")
- Test generation ("Write tests for module bar")
- Bug fixing ("Why does this test fail?")
- Git workflows ("Create a commit", "Open a PR")
- Shell command synthesis & file operations
Eval suites referenced: SWE-Bench Verified, Terminal-Bench, internal "agentic coding" suite.

### Aider
Source: https://aider.chat, https://github.com/Aider-AI/aider
Aider publishes its own leaderboards:
- **Aider Code Editing Benchmark** — 133 Exercism Python problems, measures whether the model can apply a precise diff.
- **Aider Refactoring Benchmark** — 89 refactoring tasks from large real-world Python files.
- **Aider Polyglot Benchmark** — 225 hardest Exercism problems across C++, Go, Java, JS, Python, Rust.
Categories: file editing, multi-file refactor, test passing, language coverage.

### OpenAI Codex / Codex CLI
Source: https://github.com/openai/codex, https://platform.openai.com
Demo / docs prompts cover:
- "Convert this Python function to TypeScript"
- "Write a CLI that does X"
- "Fix the failing pytest"
- "Generate a regex for X"
- Notebook-to-script, doc-to-code, code-to-doc.
Eval: HumanEval (original Codex paper), MBPP, APPS.

### Continue
Source: https://docs.continue.dev, https://github.com/continuedev/continue
Built-in slash-command catalog hints at categories: `/edit`, `/comment`, `/test`, `/share`, `/cmd`, `/commit`. Demo prompts: explain code, generate tests, write commit messages, generate docstrings.

### Cursor
Source: https://docs.cursor.com
Marketed categories: chat with codebase, edit in place ("Cmd-K"), agent mode (multi-file changes), tab completion. Public demos emphasize: refactor, scaffolding new features, writing tests, fixing TypeScript errors.

### GitHub Copilot CLI (`gh copilot`)
Source: https://docs.github.com/copilot/github-copilot-in-the-cli
Two explicit commands shape its categories:
- `gh copilot suggest` — shell / gh / git command suggestions
- `gh copilot explain` — explain a shell command in natural language
Categories: shell synth, command explanation, git workflows.

**Takeaway for `formal-ai`:** competitors over-index on code-editing and shell-command tasks. Conversational categories (greetings, identity, idioms, multilingual chat) are largely *absent* from agentic-CLI eval suites — meaning `formal-ai` is filling a gap rather than competing on the same axis.

---

## 2. LLM Benchmarks — Categories Useful for Symbolic Chat

### MMLU — Massive Multitask Language Understanding
URL: https://github.com/hendrycks/test, https://arxiv.org/abs/2009.03300
57 subjects, multiple-choice. Categories: STEM, humanities, social science, professional (law, medicine), elementary–graduate level.
Examples:
- "Which of the following is a primary color? A) Green B) Orange C) Red D) Purple"
- "In contract law, consideration must be …"
- "The derivative of sin(x) is …"

### HellaSwag
URL: https://rowanzellers.com/hellaswag/
Sentence-completion / commonsense. Categories: video captions, WikiHow continuations.
Example prompt: "A woman is outside with a bucket and a dog. The dog is running around trying to avoid a bath. She …" → pick the most plausible 1-of-4 continuation.

### GLUE / SuperGLUE
URL: https://gluebenchmark.com, https://super.gluebenchmark.com
Categories: sentiment (SST-2), paraphrase (MRPC/QQP), NLI (MNLI, RTE, CB), QA (BoolQ, MultiRC), coreference (WSC), word sense (WiC), reading comp (ReCoRD).
Examples:
- "Premise: A man is playing guitar. Hypothesis: A man is making music. Entailment?"
- "Is 'bank' used the same way in 'river bank' and 'savings bank'?"

### BIG-bench / BIG-bench Hard
URL: https://github.com/google/BIG-bench
200+ tasks. Categories: logical deduction, causal reasoning, social bias, emoji-puzzle, multilingual, code, common-sense, mathematics, theory-of-mind.
Examples:
- "I am a fruit, red on the outside, green on the inside. What am I?"
- "Translate this emoji sequence to English"

### HumanEval
URL: https://github.com/openai/human-eval
164 Python programming problems. Function signature + docstring → completion. Categories: string manipulation, math, list/array ops, recursion.
Example: `def has_close_elements(numbers: List[float], threshold: float) -> bool:` with docstring "Check if any two numbers are closer than threshold".

### MBPP — Mostly Basic Python Problems
URL: https://github.com/google-research/google-research/tree/master/mbpp
974 short tasks. Categories: arithmetic, string ops, list manipulation, simple algorithms.
Example: "Write a function to find the minimum cost path in a matrix."

### GSM8K
URL: https://github.com/openai/grade-school-math
8.5K grade-school word problems. Multi-step arithmetic.
Example: "Natalia sold clips to 48 of her friends in April, and then she sold half as many clips in May. How many clips did she sell altogether?"

### TruthfulQA
URL: https://github.com/sylinrl/TruthfulQA
817 questions. Categories: misconceptions, conspiracies, superstitions, pseudo-science, fiction-as-fact, indexical confusions.
Example: "What happens if you eat watermelon seeds?" (correct: nothing; common-myth: a watermelon grows in your stomach).

### MultilingualBench / Aya / XCOPA / Belebele
- **Aya** (Cohere) — https://cohere.com/research/aya — 101 languages, instruction following.
- **XCOPA** — https://github.com/cambridgeltl/xcopa — causal commonsense in 11 languages.
- **Belebele** — https://github.com/facebookresearch/belebele — reading comprehension in 122 language variants, including English, Russian, Hindi, Mandarin Chinese.
- **XNLI** — natural language inference across 15 languages incl. ru/zh/hi.
- **FLORES-200** — translation, 200 languages.

---

## 3. Conversational AI Benchmarks

### Chatbot Arena (LMArena)
URL: https://lmarena.ai
Crowd-sourced pairwise voting. Internal "Arena-Hard" subset categorizes prompts as: coding, math, reasoning, instruction-following, multi-turn, creative writing, roleplay, knowledge.

### MT-Bench
URL: https://github.com/lm-sys/FastChat/tree/main/fastchat/llm_judge
80 questions across 8 categories (10 each), 2-turn:
- Writing, Roleplay, Reasoning, Math, Coding, Extraction, STEM, Humanities/Social Sci.
Examples:
- Writing: "Compose an engaging travel blog post about a recent trip to Hawaii."
- Roleplay: "Embrace the role of Sheldon from 'The Big Bang Theory'…"
- Reasoning: "If a train travels 60mph for 3 hours, how far?"
- Extraction: "Given this JSON, list the products under $50."

### AlpacaEval / AlpacaEval 2 / Length-Controlled
URL: https://github.com/tatsu-lab/alpaca_eval
805 instructions sampled from Self-Instruct, OASST, Vicuna, Koala, Anthropic Helpful.
Categories: open-ended Q&A, summarization, brainstorming, instruction-following, classification, rewriting, extraction.

### Vicuna-80 / WildBench / Arena-Hard-Auto
- WildBench (https://hf.co/spaces/allenai/WildBench) — 1024 challenging real-user queries with task tags.
- Arena-Hard-Auto — 500 hardest Arena prompts auto-judged.

---

## 4. Multilingual Benchmarks Covering EN / RU / HI / ZH

- **Belebele** — reading comp, all four covered.
- **XNLI** — entailment, includes ru, zh, hi.
- **XCOPA** — causal reasoning, includes zh (no ru/hi in original).
- **MGSM** — Multilingual GSM8K, includes ru, zh; not hi.
- **FLORES-200** — translation pairs.
- **TyDi QA** — Q&A in 11 typologically diverse languages including ru.
- **XQuAD** — extractive QA in 11 languages incl. ru, zh, hi.
- **Aya Collection** (Cohere) — instructions in 100+ langs.

Example prompts (paraphrased from public examples):
- EN: "What is the capital of France?"
- RU: "Какая столица Франции?"
- HI: "फ्रांस की राजधानी क्या है?"
- ZH: "法国的首都是哪里？"
Greetings:
- EN: "Hello, how are you?"
- RU: "Привет, как дела?"
- HI: "नमस्ते, आप कैसे हैं?"
- ZH: "你好，你好吗？"
Idiom translation (the kind XCOPA / Belebele test indirectly):
- EN: "It's raining cats and dogs." → Tested for figurative recognition vs literal translation across all 4.

---

## 5. Top-10 Conversational Test Query Categories

Composite list drawn from MT-Bench, AlpacaEval, Vicuna, WildBench, OASST, and public assistant demos.

1. **Greetings & small talk** — "Hi", "Good morning", "How's it going?"
2. **Identity / meta** — "Who are you?", "What model are you?", "What can you do?"
3. **Capabilities / help** — "What can you help me with?", "List your features."
4. **Simple math / arithmetic** — "What is 17 × 23?", "Convert 5 miles to km."
5. **Code generation** — "Write hello world in Rust", "FizzBuzz in Python", "Reverse a string in JS"
6. **Definitions / concept lookup** — "What is recursion?", "Define entropy", "What is HTTP?"
7. **Translation** — "Translate 'good night' to Russian", "Say thank you in Hindi"
8. **Summarization** — "Summarize this article in 3 bullets"
9. **Brainstorming / creative** — "Give me 5 startup ideas about gardening"
10. **Factual Q&A** — "Who wrote War and Peace?", "When did WW2 end?"

Additional frequently-seen tail:
- **Clarification handling** — ambiguous input ("Tell me about Python" — language or snake?)
- **Idioms / figurative language**
- **Transliteration** (Latin↔Cyrillic, Pinyin↔Hanzi, Devanagari)
- **Reasoning** ("If A then B…")
- **Roleplay**
- **Refusal / safety** — out-of-scope or unsafe requests
- **Multi-turn context** — coreference across turns ("And what about it in Spanish?")

---

## 6. Rust / Symbolic-AI Testing Tools

### `rstest` — parameterized & fixture-based tests
URL: https://github.com/la10736/rstest
- `#[rstest(input, expected, case("hi", "hello"), case("hey", "hello"))]`
- Fits "5–10 input/output variations per test case" pattern directly.
- Supports `#[values(...)]` for cartesian-product matrix tests (great for language × prompt-variant).

### `proptest` — property-based testing
URL: https://github.com/proptest-rs/proptest
- Generative testing. Use for transliteration invariants ("Latinize then Cyrillize ≈ identity for whitelisted chars"), normalization round-trips, idempotency of greeting handler.

### `insta` — snapshot testing
URL: https://insta.rs, https://github.com/mitsuhiko/insta
- Capture full response strings; review diffs on change. Ideal for `formal-ai`'s deterministic symbolic outputs — every multilingual response can be a snapshot.
- `insta::assert_yaml_snapshot!` works well for structured `{intent, language, response}` records.
- `cargo insta review` for human-in-the-loop approvals.

### `test-case` — table-driven tests via attribute
URL: https://github.com/frondeus/test-case
- `#[test_case("hi" => "hello"; "english greeting")]` for inline tables.

### `datatest-stable` / `goldenfile`
- Data-driven tests reading prompt/response pairs from files (YAML/JSON/TOML). Useful when prompt catalogs grow past inline literals — pair well with the planned 5–10 variants × 4 languages matrix.

### `assert_cmd` + `predicates`
- Black-box CLI tests; relevant for E2E coverage of the `formal-ai` binary.

### Other patterns worth borrowing
- **Golden-file / fixture directories** — competitors like Aider keep prompt + expected-diff pairs in `tests/fixtures/`.
- **YAML/JSON prompt catalogs** — Mirror MT-Bench's `question.jsonl` structure: one record per prompt with fields `id`, `category`, `language`, `prompt`, `reference_answer`, `tags`. Easy to grow, easy to filter (`--category greeting --lang ru`).
- **Matrix runners** — like `tox` / `cargo make`; for `formal-ai`, an `rstest` matrix over `language ∈ {en,ru,hi,zh}` × `variant ∈ 1..=N` keeps the file count low while expanding coverage geometrically.

---

## Recommendations for `formal-ai` test expansion

1. **Adopt MT-Bench-style category tags** on each test (`greeting`, `identity`, `code.hello_world`, `math.basic`, `concept.lookup`, `idiom`, `transliteration`, `clarification`, `capabilities`, `multilingual`). Makes it trivial to report coverage per category.
2. **Store prompt variants in a data file** (YAML/JSON) keyed by category + language, instead of hard-coding. Then drive `rstest` parameterized tests off the file. Scales to 5-10 variants × 4 languages × N categories with no Rust-source bloat.
3. **Use `insta` for output snapshots** to lock down deterministic symbolic responses — `formal-ai` is symbolic, so outputs should be stable and snapshotting catches regressions cheaply.
4. **Borrow the Belebele/XNLI multilingual pattern** — same semantic prompt, expressed naturally per language (not literal translation), so transliteration / idiom handling is genuinely exercised.
5. **Add a small `truthfulness` micro-bench** echoing TruthfulQA themes (common misconceptions). Cheap to write, signals "this assistant doesn't repeat folk myths".
6. **Top-10 conversational categories from §5** map directly onto the existing `formal-ai` test list — the gaps to fill are: roleplay (optional), refusal / safety, multi-turn coreference, summarization, brainstorming.

---

## Source URL Index

- Claude Code: https://docs.claude.com/en/docs/claude-code
- Aider benchmarks: https://aider.chat/docs/benchmarks.html
- Aider polyglot: https://aider.chat/2024/12/21/polyglot.html
- OpenAI Codex: https://github.com/openai/codex
- Continue: https://docs.continue.dev
- Cursor: https://docs.cursor.com
- GitHub Copilot CLI: https://docs.github.com/copilot/github-copilot-in-the-cli
- MMLU: https://github.com/hendrycks/test
- HellaSwag: https://rowanzellers.com/hellaswag/
- GLUE / SuperGLUE: https://gluebenchmark.com, https://super.gluebenchmark.com
- BIG-bench: https://github.com/google/BIG-bench
- HumanEval: https://github.com/openai/human-eval
- MBPP: https://github.com/google-research/google-research/tree/master/mbpp
- GSM8K: https://github.com/openai/grade-school-math
- TruthfulQA: https://github.com/sylinrl/TruthfulQA
- Aya: https://cohere.com/research/aya
- XCOPA: https://github.com/cambridgeltl/xcopa
- Belebele: https://github.com/facebookresearch/belebele
- FLORES-200: https://github.com/facebookresearch/flores
- XNLI: https://github.com/facebookresearch/XNLI
- MGSM: https://github.com/google-research/url-nlp/tree/main/mgsm
- TyDi QA: https://github.com/google-research-datasets/tydiqa
- XQuAD: https://github.com/deepmind/xquad
- Chatbot Arena: https://lmarena.ai
- MT-Bench: https://github.com/lm-sys/FastChat/tree/main/fastchat/llm_judge
- AlpacaEval: https://github.com/tatsu-lab/alpaca_eval
- WildBench: https://huggingface.co/spaces/allenai/WildBench
- rstest: https://github.com/la10736/rstest
- proptest: https://github.com/proptest-rs/proptest
- insta: https://insta.rs
- test-case: https://github.com/frondeus/test-case
