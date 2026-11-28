# Preserve command output and actual process receipts

The Formal AI command wrapper in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) split its trace
at any `== answer ==` substring and searched the whole response for exit-code
text. Reading source containing that marker hid the rest of the tool output.
A successful command printing `Exit Code: 19` made the wrapper return 19.
Final-answer claims and incomplete or signaled captures could also receive
execution credit.

The wrapper now recognizes the driver's top-level answer boundary and the
single exact command header. It accepts a status only from that result's final,
indented process footer after a successful, completed driver. Source and error
output stay visible. Changed or repeated commands, malformed arguments, missing
boundaries, incomplete captures, signals and spawn errors fail closed. The full
raw driver response remains retained in the existing evidence capture.

Reproduce with a text file containing these three lines and run
`node experiments/issue-1580/formal-command.mjs 'cat <file>'`:

~~~text
source.split('== answer ==')[0]
Exit Code: 19
remaining source
~~~

The actual driver experiment retained a Unicode suffix on the last line. Before
repair, the wrapper returned 19 and displayed 143 bytes, ending inside the
first source line. After repair, it returned 0 and displayed all 222 bytes,
including the printed 19 and the actual terminal status 0.

`node --test --test-concurrency=1 experiments/issue-1580/formal-command.test.mjs`
runs 16 small regression cases. The unchanged-behavior parser extraction fails
11 cases before the repair; all 16 pass afterward. The existing maintained
`issue-1580-template-observations.test.mjs` aggregator imports this regression,
so the existing CI lanes execute it. Seven affected JavaScript gates, syntax
checks of both new/changed JavaScript files and the whitespace check pass.
No Rust build or full local suite ran.

The [receipt](../../../experiments/issue-1580/evidence/checks/formal-command-output-repair-20261123/manifest.json)
retains original and extracted source, all before/after test logs, both real
driver outputs, the exact source delta, verification scripts and complete gate
output. Every member was read back and verified against byte count and SHA-256.

Hosted qualification of this new delta is unfinished. This repair neither
completes the broad original issue nor supplies independent authorship credit.
The pull request remains draft and unmerged.
