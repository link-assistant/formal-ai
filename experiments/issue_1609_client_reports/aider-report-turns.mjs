// The scripted Aider report conversation: each turn waits for a line of Aider's
// output and answers it. Kept apart from aider-report.mjs so the PR-lane test
// can hold these matchers to the server's real menu text (issue #1639).
//
// The report menus print the question on one line and every numbered option on
// the next, separated by two spaces ("1. Harness log — …  2. Server log — …"),
// since 8711d61f2 kept them within clients' concise output limits. A choice is
// therefore matched anywhere on a line, not only at its start.
export const aiderReportTurns = Object.freeze([
  [/Hi[!,]/u, 'Report issue'],
  [/(?:^|\s)3\. GitHub issue\b/u, '3'],
  [/(?:^|\s)3\. Server log\b/u, '1'],
  [/^formal-ai report create-issue --session '[^']+' --source both /u, null],
  [/^https:\/\/github\.com\/link-assistant\/formal-ai\/issues\/9999$/u, 'n\n/exit'],
]);

