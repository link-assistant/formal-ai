# Reproduction of the dialog (issue #340)

Legend: `U` = user, `A` = agent.

Captured from the GitHub Pages WASM worker, version `0.149.0`, manual mode,
UI language `ru`, locale `ru-RU` (`Asia/Yekaterinburg`), timestamp
`2026-05-29T18:44:07.939Z`.

```
U: Write a Rust program that:
   1. Makes an HTTP GET request to a URL
   2. Parses the JSON response
   3. Calculates statistics (mean, median) from the data
   4. Outputs the results

   Include error handling and comments.
A (intent: write_program_unsupported, reported): I can route
   `write_program(language, task)`, but I do not have a template for language
   `rust` and task `missing`. Supported languages: rust, python, javascript,
   typescript, go, c, cpp, java, csharp, ruby. Supported tasks: hello_world,
   count_to_three, list_files, list_files_arg.
```

## What the user expected

A complete, idiomatic Rust program covering the four numbered requirements
(HTTP GET, JSON parsing, mean/median statistics, output) with error handling
and comments — i.e. the kind of multi-step program a competent programmer would
write, not a single-line "hello world" template.

## What actually happened

The request was routed to the `write_program` intent (program noun `program` +
imperative verb `write`), and the requested **language** (`rust`) was extracted
correctly, but the **task** could not be resolved to any catalog alias, so the
formalizer produced `UnsupportedWriteProgram { task: None, language: Some("rust") }`
and the engine rendered the "I do not have a template for language `rust` and
task `missing`" dead-end. The four explicit sub-requirements in the prompt were
never inspected.
