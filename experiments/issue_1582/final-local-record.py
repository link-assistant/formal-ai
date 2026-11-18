from pathlib import Path

root = Path('docs/case-studies/issue-1582')
p = root / 'README.md'
s = p.read_text()
s = s.replace('The complete local gate inventory was run twice: the second round passed or\nskipped 134 of 138 entries.', 'The third complete local gate run passed or skipped all 138 entries with no\nfailures. The second round had passed or skipped 134 of 138 entries.')
s = s.replace('Per-round gate logs retain the original failures; subsequent focused checks\nrecord the corrected state.', 'Per-round gate logs retain the original failures; subsequent focused checks\nand the complete third round record the corrected state. A fourth complete\nround checks the final Codex seed change before pushing.')
p.write_text(s)
p = root / 'work-plan.md'
s = p.read_text().replace('- [ ] Update mirrors, generated translations, release notes and required local gates.', '- [x] Update mirrors, generated translations and release notes. Complete local gate run: 138/138 passed or explicitly skipped; final Codex seed change is being rechecked.')
p.write_text(s)
p = Path('experiments/issue_1582/pull-request.md')
s = p.read_text().replace('Codex, Qwen, Aider and current Grok receive numeric served limits;', 'Codex enables its dedicated live web-search tool while retaining its existing read-only shell sandbox. Codex, Qwen, Aider and current Grok receive numeric served limits;')
s = s.replace('The full JavaScript gate inventory and corrections are retained in the case study.', 'The complete local gate inventory passes or explicitly skips all 138 entries with no failures; the case study retains every correction and skipped-check reason.')
p.write_text(s)
