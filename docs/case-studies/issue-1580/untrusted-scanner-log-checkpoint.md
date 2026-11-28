# Contributor-controlled scanner diagnostics

The current Go template keeps untrusted log text behind a command-processing
boundary. Formal AI already escapes its machine annotations, but its human
diagnostics printed filenames and prose directly. A filename containing
`##[error]` creates two fake errors after a valid size warning. A hardcoded
language filename containing `##[add-mask]` creates an unintended mask command.
These are real scanner subprocess reproductions, not inferred source findings.

Use a shared deterministic encoder in both language twins. Ordinary diagnostics
stay byte-identical. Unsafe values become reversible JSON strings with escaped
colon/hash delimiters and control characters, so neither Actions command
protocol can interpret them. Intentional machine annotations keep their normal
property/data escaping. Apply the same boundary to associative-terminology
human diagnostics and read errors. This selected sink repair does not establish
repository-wide log isolation.

The four focused cases pass, including exact data round trips and independent
command-parser controls. The original size reproducer fails before repair;
its two initially missing helper tests are not additional before-behavior
reproductions. The added hardcoded-language reproducer separately fails before
its repair. The maintained owner now passes all 195 observations, and the
existing script-twin JavaScript checks pass. The source scan still has 60
warnings and no violations. Rust formatting passes; no native compilation or
native execution is claimed. The two Rust helper tests and exact twin output
still require hosted qualification.

Move the existing file-size Rust test module into an explicitly addressed
child to preserve source headroom. An exact inverse reconstructs every byte
before extraction; existing test function names and parent test namespace stay
the same. No assertions or fixtures are removed.

The complete retained observations, preimages, source identity and conservation
proof are under experiments/issue-1580/evidence/checks/untrusted-scanner-logs/.
The Actions runner's own parser confirms the leading-marker rule for V2 and
the anywhere-in-line search for its legacy protocol:
https://github.com/actions/runner/blob/7e211dc69c347b78fc398e630e74facb11a13235/src/Runner.Common/ActionCommand.cs

The Go comparison also records the now-published acquisition recovery adoption;
its historical decision is retained separately. Every supplied implementation
and conclusion receives zero independent Formal AI credit. Other template,
live/budget, source warning and independent reasoning criteria remain open.
The PR remains draft and unmerged.

PR: https://github.com/link-assistant/formal-ai/pull/1581
