# Agent CLI run: compiling Kotlin from the command line (R1163-15)

Producer: the JavaScript server of this branch (`node js/server/main.mjs
--agent-mode`, version 0.352.1, worktree at `73ff52f71` plus this pull
request's uncommitted changes) driven by `@link-assistant/agent` 0.26.0 with
`--model formal-ai/formal-ai`, the same `opencode.json` provider block as
`docs/case-studies/issue-1138/self-use/run_self_use_batch.sh`. Run started
2026-10-07T08:25:37Z in a fresh temporary workspace; the Agent CLI exited 0
after four model steps. No native binary was built for this run (the branch
is drafted without a local `cargo` build), so this is the JavaScript engine's
behaviour, not the Rust engine's.

Prompt: `How do I compile a Kotlin program from the command line?`

## What happened

| Step | Tool call issued by Formal AI | Result |
| --- | --- | --- |
| 1 | `websearch` `{"query": "i compile a kotlin program from the command line"}` | 16998 characters; the first hit is `https://kotlinlang.org/docs/command-line.html` and its highlights contain `kotlinc hello.kt -include-runtime -d hello.jar`. |
| 2 | `webfetch` `{"format": "text", "url": "https://kotlinlang.org/docs/command-line.html"}` | 4665 characters of page text, which also contain the `kotlinc … -include-runtime -d …` command. |
| 3 | `webfetch` `{"format": "text", "url": "https://kotlinlang.org/docs/compiler-reference.html"}` | 20585 characters. |
| 4 | final answer (`stop`) | quoted below. |

Final answer, verbatim:

```text
Kotlin command-line compiler | Kotlin DocumentationKotlin  HelpKotlin command-line compilerEvery Kotlin release ships with a standalone version of the compiler. Compile the application using the Kotlin compiler: To compile a Kotlin/Native application, use the Kotlin/Native compiler.Compile a libraryIf you're developing a library to be used by other Kotlin applications, you can build the .

web_research_source_label: https://kotlinlang.org/docs/command-line.html

Kotlin compiler options | Kotlin DocumentationKotlin  HelpKotlin compiler optionsEach release of Kotlin includes compilers for the supported targets: JVM, JavaScript, and native binaries for supported platforms.These compilers are used by:The IDE, when you click the Compile or Run button for your Kotlin project.Gradle, when you call gradle build in a console or in the IDE.Maven, when you call mvn compile or mvn test-compile in a console or in the IDE.You can also run Kotlin compilers manually from the command line as described in the Working with command-line compiler tutorial.Compiler optionsKotlin compilers have a number of options for tailoring the compiling process. -kotlin-home pathSpecify a custom path to the Kotlin compiler used for the discovery of runtime libraries. The command-line tool for Kotlin/Native compilation is kotlinc-native.In addition to the common options, Kotlin/Native compiler has the options listed below.

web_research_source_label: https://kotlinlang.org/docs/compiler-reference.html
```

- Raw Agent CLI log (not published, 85529 bytes): SHA-256
  `20ea743e95639a00a6596d0df6d44ec38d4912ea7d72b6060ceb319765d65714`.
- Final answer above: SHA-256
  `f944fe33ebe616077bef30f8f73bd83eab8eb8c5bf5d6c99c75b186afb4a70e1`.

## Findings

1. Retrieval works end to end: the engine's own tool calls found and fetched
   the right page, and the command is in the fetched bytes.
2. The answer does **not** contain the command. The Agent CLI path answers by
   picking prose sentences out of `webfetch` plain text (the
   `web_research_source_label` composition); the page formalizer of this issue
   (`formalizePage` / `formalize_page`, `command_mentioning`) is not on that
   path, so the sentence "Compile the application using the Kotlin compiler:"
   survives while the code block it introduces is dropped. The search query
   also lost its leading "how do" to "i".
3. The same branch's formalizer, run on the same URL fetched live a minute
   earlier (`../../self-use/kotlinlang-compile/formalizer-run.json`), does
   resolve the paragraph mentioning "Compile" to exactly
   `kotlinc hello.kt -include-runtime -d hello.jar` and "Run" to
   `java -jar hello.jar`, with SHA-256
   `0847a47edc7136be89ddb103aed1178c4eb95a0ba03bdfe396f1b207ab806e8e` — the
   bytes committed as
   `rust/tests/fixtures/coding-discovery/captured/issue-1163/command-line.html`.

The gap this run documents is routing, not formalization: the agentic
research composition should formalize `webfetch` results (request
`format: "html"`) and answer a how-to question from `command_mentioning`
instead of prose sentence selection. That is follow-up work; this README is
the evidence, not a claim that the agent path answers correctly.
