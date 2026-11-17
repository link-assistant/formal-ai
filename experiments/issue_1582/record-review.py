"""Update the case study with the concrete verification state."""
from pathlib import Path
p=Path('docs/case-studies/issue-1582/README.md')
s=p.read_text().replace('routing made all eight pass. A real HTTP transport regression was then added.', 'routing made all eight pass. Real HTTP transport and resource-derived model\nmetadata regressions brought the issue file to ten passing tests. The logger\nsuites pass sixteen more tests, including lossless conversion of nested logs: 26\nfocused JavaScript tests in total.')
s=s.replace('The work plan records remaining checks without declaring the issue complete.', 'The work plan records remaining checks without declaring the issue complete.\nThe complete local gate inventory was run twice: the second round passed or\nskipped 134 of 138 entries. The four failures were a stale seed metadata audit,\na dependency count that had fallen below its ceiling, test row syntax the\nmultilingual checker could not identify for English/Spanish, and duplicate JSON\nprefix parsing counted by the literal-predicate ratchet. The repair adds proper\nseed metadata, lowers the dependency ceiling, names languages in the test rows,\nand unifies JSON assignment/merge handling without raising a debt ceiling.\nPer-round gate logs retain the original failures; subsequent focused checks\nrecord the corrected state.')
s=s.replace('The bootstrap installed only the maintained generator\'s Babel parser/traversal', 'Two full-gate attempts through the driver hit its 60-second tool bound; a\nbackground child was also terminated by the driver. The full JavaScript gate\ncommands were therefore executed directly with complete logs retained. This is\na documented execution-path exception, with zero authorship credit. The worker\nidentity bootstrap also needed one direct maintained-generator invocation\nbecause the changed worker could not boot its own stale identity.\nThe bootstrap installed only the maintained generator\'s Babel parser/traversal')
s+='''

The supplied experiment recipes are investigation history. Some are one-time
transformations and intentionally fail on a second application; they are not a
replacement setup command. Use the maintained generators and focused commands
above to verify the current tree. Temporary dependency/tool checkouts stay
outside committed source and no local native compilation was attempted.
'''
p.write_text(s)
p=Path('docs/case-studies/issue-1582/work-plan.md')
s=p.read_text()
for old in ['Enable read-only network retrieval by default, respecting explicit offline/deny controls.', 'Answer internet access and permission questions with actionable harness controls in the supported languages.', 'Send routine startup/exit diagnostics to files; opt-in terminal logging and nested formatting.', 'Add regression tests, reproduce failures before the changes, and run focused checks afterward.', 'Complete timeline, root causes, solution research, upstream reporting where applicable, evidence index and limitations.']:
 s=s.replace('- [ ] '+old,'- [x] '+old)
s=s.replace('- [ ] Configure actual resource-derived context limits for every supported client; fix Claude\'s unknown-model fallback.', '- [ ] Configure actual resource-derived context limits for every supported client. Claude and configurable integrations are implemented; **Gemini input context is blocked upstream** (google-gemini/gemini-cli#30333). Native adapter tests still need execution on an adequately provisioned runner.')
s=s.replace('- [ ] Archive every available current-branch CI run and explain inaccessible original local logs.', '- [x] Archive all 33 available historical branch runs and log-fetch attempts; explain inaccessible original Mac logs. Current repair-head CI remains below.')
p.write_text(s)
# A nested checkout used only by generator verification is scratch state.
p=Path('.git/info/exclude')
s=p.read_text()
if '/.meta-language/' not in s:p.write_text(s+'\n/.meta-language/\n')
# Preserve fresh feedback and metadata for comparison to the initial snapshots.
import shutil
root=Path('docs/case-studies/issue-1582/raw-data')
for name in ['pr-current','runs-current','issue-comments-current','pr-comments-current','inline-current','reviews-current']:
 source=Path('/tmp')/('issue-1582-'+name+'.json')
 if source.exists():shutil.copyfile(source,root/(name+'.json'))
