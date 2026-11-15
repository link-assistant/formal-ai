"""Rewrite `JSON.parse(<expr>.answer)` in a test into `recorded(<expr>)`.

Issue #1560: a work-item session answers with a localized report; its record
is kept on the final result. `recorded` asserts the answer is not a JSON object
and returns the record. Usage: python3 migrate-answer-parse.py FILE...
"""
import re
import sys

HELPER = (
    "function recorded(plan) {\n"
    "  assert.equal(parsesAsJsonObject(plan.answer), false, 'a work-item answer is a report, not a JSON object');\n"
    "  const record = plan.record ?? workItemRecordOf(plan);\n"
    "  assert.ok(record, 'a work-item final keeps its record');\n"
    "  return record;\n"
    "}\n"
)


def rewrite(source):
    out, at, count = [], 0, 0
    while True:
        start = source.find('JSON.parse(', at)
        if start < 0:
            break
        depth, index = 0, start + len('JSON.parse')
        while index < len(source):
            if source[index] == '(':
                depth += 1
            elif source[index] == ')':
                depth -= 1
                if depth == 0:
                    break
            index += 1
        inner = source[start + len('JSON.parse('):index]
        if inner.endswith('.answer'):
            out.append(source[at:start] + 'recorded(' + inner[:-len('.answer')] + ')')
            count += 1
        else:
            out.append(source[at:index + 1])
        at = index + 1
    out.append(source[at:])
    return ''.join(out), count


def main():
    for path in sys.argv[1:]:
        source = open(path, encoding='utf8').read()
        text, count = rewrite(source)
        if count == 0:
            print(path, 0)
            continue
        depth = path.count('/')
        prefix = '../' * depth
        imports = list(re.finditer(r'^import .*;$', text, re.M))
        anchor = imports[-1].end() if imports else 0
        line = "\nimport { workItemRecordOf, parsesAsJsonObject } from '" + prefix + "js/agentic/work_item_report.mjs';\n"
        if 'function recorded(' not in text:
            text = text[:anchor] + line + HELPER + text[anchor:].lstrip('\n')
        open(path, 'w', encoding='utf8').write(text)
        print(path, count)


main()
