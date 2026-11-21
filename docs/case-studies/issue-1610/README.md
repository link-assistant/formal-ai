# OpenCode issue report with missing server history

This is the OpenCode sub-issue of [issue #1609](../issue-1609/README.md). It is tracked as [issue #1610](https://github.com/link-assistant/formal-ai/issues/1610) in [PR #1611](https://github.com/link-assistant/formal-ai/pull/1611).

On 2026-11-21 at 15:39:00.766 UTC, Formal AI 0.356.0 on macOS filed a report for OpenCode session `ses_e013e4ac4ffeZ5CsfCO3RVX0Yi`. The issue body recorded 10 harness messages, zero server messages and 24,735 exported bytes. The [issue snapshot](raw-data/issue.json) and the [issue comments](raw-data/issue-comments.json) retain the original report and publication receipt.

## Sequence and causes

1. OpenCode sent a separate title request. Formal AI treated a list introduction as a required output literal. Its refusal became the terminal session title. [Original title and greeting screenshot](screenshots/opencode-greetings-before.png).
2. `Привет` and `Hello` reached the correct handlers. The harness log shows their intended answers in the reasoning records. A clause split out of OpenCode’s response guidance replaced both answers with output-constraint errors. The Linux v0.356.0 release produced the same failure family with OpenCode 1.19.5. [Released-client output](raw-data/opencode-0.356.0.log).
3. The report question offered multiple destinations. Subsequent shell steps displayed the same generic narration for harness export, server export, learning, and GitHub submission. [Original report actions screenshot](screenshots/opencode-actions-before.png).
4. The harness exported real messages. The server lookup and learning action could not find exchanges under its native `ses_…` identifier. The HTTP capture and the harness were using different identifiers. The server Gist recorded that error and the merged document had only harness turns. [Harness export](raw-data/harness.lino), [server export](raw-data/server.lino), [merged export](raw-data/merged.lino).
5. The GitHub CLI did create issue #1610. The final output projection returned another constraint error in place of the URL. [Original confirmation screenshot](screenshots/opencode-confirmation-before.png), [filed issue](https://github.com/link-assistant/formal-ai/issues/1610).

## Verification map

| Requirement | Implementation | Regression |
| --- | --- | --- |
| Useful title and greetings | Apply title formatting to the separate client request and classify known conversational instructions as style guidance. | English and Russian messages plus the OpenCode title request in the HTTP replay. |
| A different, concise explanation for each command | Reuse the seeded localized responses for client export, server export, learning and publication. | Native report tests cover five languages. The terminal path exercises OpenCode’s original multiple-selection flow. |
| Server and harness files refer to the active conversation | Read the native OpenCode session header and write canonical conversation records under that ID. Retain the records for wrapper-managed servers. | Verify that the scripted export uses the ID from the wire and that the issue body and server attachment include server messages. |
| Return a trustworthy final confirmation | Finish only after observing matching tool outputs. A successful receipt must include an issue URL under the Formal AI repository. | Test a successful receipt and an error response containing a URL-shaped string. Check the stock terminal confirmation with a local GitHub fixture. |
| Prevent recurrence on deployment | Run the selected binary and stock client tests as a required package verification step in each release path. | The release workflow test checks both publication graphs. |

The [stock OpenCode localhost run](raw-data/opencode-1.19.5-sandbox-report.log) displayed both greetings, narrated the four actions separately, and finished with the fixture issue URL. The server used the patched JavaScript code and the command boundary used recording fixtures. The release regression uses the selected native binary to verify actual harness and server attachments before publication. [Original confirmation](screenshots/opencode-confirmation-before.png), [localhost confirmation](screenshots/opencode-confirmation-after.png).

The original Gists are mirrored byte-for-byte. The [original attempt archive](raw-data/attempt-44920308698.tar.gz) preserves the separate failed Hive Mind draft and its identity receipt. [Original harness Gist](https://gist.github.com/konard/32bf71914c4af11c052eb5c634d9557b), [original server Gist](https://gist.github.com/konard/5ca673789e9f546f9803789e958cef67), [original merged Gist](https://gist.github.com/konard/8310741a4b47a9a8d25ebd93404bf584).
