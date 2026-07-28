// Parity harness for issue #839: renders a fixture through the browser mirror
// (`src/web/app/issue-report.js`) so the Rust test can compare its own output
// byte for byte. Reads the fixture as JSON on stdin, writes the rendered
// artifacts as JSON on stdout.
//
// Usage: node tests/support/render-issue-report.mjs < fixture.json

import { readFileSync } from "node:fs";
import {
  issueTitle,
  renderReportBody,
  truncateRecords,
} from "../../src/web/app/issue-report.js";

const fixture = JSON.parse(readFileSync(0, "utf8"));
const result = {};

if (fixture.body) {
  result.body = renderReportBody(fixture.body);
}
if (fixture.title) {
  result.title = issueTitle(fixture.title.turns, fixture.title.settings);
}
if (fixture.truncate) {
  const { text, max_bytes: maxBytes, omitted_label: omittedLabel } = fixture.truncate;
  result.truncate = truncateRecords(text, maxBytes, omittedLabel);
}

process.stdout.write(JSON.stringify(result));
