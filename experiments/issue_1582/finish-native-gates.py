from pathlib import Path

p=Path('data/meta/dependency-direction.lino')
s=p.read_text()
assert 'outward-references-ceiling 33' in s
p.write_text(s.replace('outward-references-ceiling 33', 'outward-references-ceiling 25'))
p=Path('data/meta/core-boundary-ledger.lino')
s=p.read_text()
old='source rust/src/solver_handlers/web_requests.rs\n    disposition justified-native\n    baseline_lines 859'
if old not in s:
    import re
    match=re.search(r'(source rust/src/solver_handlers/web_requests.rs\n    disposition \S+\n    baseline_lines )859',s)
    assert match
    s=s[:match.start()]+match.group(1)+'858'+s[match.end():]
else:
    s=s.replace(old,old[:-3]+'858')
for key,old,new in [('source_lines_max',35695,35694),('outside_core_lines_max',18599,18598)]:
    assert f'{key} {old}' in s
    s=s.replace(f'{key} {old}',f'{key} {new}')
s+='  note "The final #1582 native read-policy review removes one obsolete web-search comment line; its source baseline and both sums fall by one. Moving eight remaining read gates off the CLI surface also lowers the outward dependency ceiling to 25."\n'
p.write_text(s)

p=Path('docs/case-studies/issue-1582/README.md')
s=p.read_text().replace('The sixth complete\nround checks this additional repair before pushing.',
    'The sixth round caught the expected reduced dependency ceiling (33 to 25),\nweb-request baseline (859 to 858) and one stale JavaScript-to-Rust projection\nidentity after its comment update. Maintained generation and lower ceilings\nrepair those failures; the seventh complete round checks the final tree.')
s=s.replace('The upstream library interfaces already provide logging codecs, environment',
    'Recent related merged PRs were also reviewed: [#1564](https://github.com/link-assistant/formal-ai/pull/1564)\nkeeps machine work-item evidence in the trace while returning localized prose;\n[#1567](https://github.com/link-assistant/formal-ai/pull/1567) demonstrates why\nproduction native type checks and tests must not be conflated. Their downloaded\nbodies and the related-PR search results are retained with the research.\n\nThe upstream library interfaces already provide logging codecs, environment')
p.write_text(s)
p=Path('docs/case-studies/issue-1582/work-plan.md')
p.write_text(p.read_text().replace('the final native route repair is being rechecked in round six.', 'round six caught lower debt measurements and a stale comment projection; their repairs are being checked in round seven.'))
