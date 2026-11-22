# Production release-note selection

The release CLI discarded failures while listing archives and reading changelog
files. An invalid UTF-8 CHANGELOG.md, a directory at that path, or an unreadable
archive could become a successful generic release note or select an older copy.
Its regex parser also selected version headings inside fenced examples and
terminated real notes at example headings. Existing unit tests exercised a
separate copied regex, so passing them did not verify the publishing parser.

Reproduce the parser error with a changelog containing a fenced Markdown example
with `## [1.2.3]` before the real version heading, or `## [0.0.0]` inside an example
in the desired version's notes. Reproduce the I/O error by writing byte `ff` to
CHANGELOG.md, or creating a directory at that file path. The retained original
source shows the discarded errors and unqualified heading regex. An executed
before-fix native failure has not yet been observed.

The CLI and original five changelog cases now share scripts/release-changelog.rs.
Its scanner follows the Unlicense Rust template's fenced-section approach at
5f1e7a96acbf7a93b567195443cf6e5158fbbf1f, scripts/release-content.rs, while retaining
Formal AI's trimmed notes and generic fallback for genuinely absent optional
sources or a missing/empty section. Existing newest-first archive precedence is
retained. Enumerated archive files are required; failed reads cannot redirect
selection. A main-changelog match still avoids reading unused archives.

Closing fences accept only ASCII spaces and tabs after the delimiter, as required
by the [CommonMark fenced-code rules](https://spec.commonmark.org/0.31.2/#fenced-code-blocks).
Fresh source at template HEAD 260ae6f826d6bff0cbfe2280098c287a4c09b3f4
retains Unicode trim, permitting a nonbreaking space to close a fence; the
new physical native parser fixture covers that difference without claiming an
executed upstream reproduction.

Four new native cases cover fake headings, delimiter identity and length,
indented and unclosed fences, CRLF, exact version identity, Unicode closing-fence
whitespace and physical missing/unreadable main and archive sources. The focused
workflow_contracts target includes the original changelog module and its shared
parser tests. The maintained JavaScript artifact suite verifies this production
and native-target registration and passes all 19 top-level cases. Rust formatting
passes without a local Rust build. Native execution remains pending hosted CI.

Supplied edits and procedures earn zero independent-authorship credit. This
practice repair does not qualify the remaining template practices or complete
issue #1580. PR #1581 remains draft and unmerged.

Retained preimages, formatting, admission and touched-gate streams are under
evidence/checks/production-changelog-repair. The fresh template source identity
is a source comparison, not an executed upstream reproduction.

Retained preimages, formatting, admission and touched-gate streams are under
evidence/checks/production-changelog-repair. The fresh template source identity
is a source comparison, not an executed upstream reproduction.
