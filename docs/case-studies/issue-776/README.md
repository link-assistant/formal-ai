# Issue 776: source-first translation through a meta language

Issue 776 exposed two consecutive gaps: `source - translate to target` was not a recognized translation word order, and the reported proposition had no shared semantic representation from which target-language text could be rendered.

The implementation projects suffix command frames from the same seed role already used for prefix/circumfix forms, applies the strategy in Rust and the browser worker, and represents the proposition once with en/ru/hi/zh lexemes. All translations therefore follow source surface → language-neutral meaning → target surface.

The complete GitHub snapshots, CI logs, reproduction outputs, research, timeline, requirement matrix, root-cause analysis, and alternatives are in [`dev/log/issues/776/pulls/794`](../../../dev/log/issues/776/pulls/794/README.md).

## Verification contract

- Exact reported prompt routes to `translate_ru_to_en`.
- Extraction excludes the separator and command.
- Every cross-language directed pair among en/ru/hi/zh resolves to the expected surface.
- Every reverse leg resolves to the same meaning id and original surface.
- The browser worker produces the same result as native Rust.
