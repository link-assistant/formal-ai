# lindera docs.rs build failure — reproduction

Reproduces the upstream defect behind requirement R3 of
[issue #736](https://github.com/link-assistant/formal-ai/issues/736): docs.rs
reports *All builds failed* for `formal-ai`.

Run `./run.sh`. See [the case study](../../docs/case-studies/issue-736/README.md#43-defect-3--docs-generation-shows-as-failing-is-docsrs-not-github-actions)
for the full analysis, and the upstream report linked from there.

The first build needs to download the jieba dictionary and takes a few minutes.
