// Qualify the narrow local workflow-expression fixture against GitHub's engine.
// Supply an integrity-verified @actions/expressions package; never install into
// the repository. This executes the expression library, not an Actions runner.
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {execFileSync} from 'node:child_process';
import {Script} from 'node:vm';
import YAML from 'yaml';
const [packagePath, output] = process.argv.slice(2);
assert(packagePath && output, 'provide package directory and report path');
const base = resolve(packagePath);
const metadata = JSON.parse(readFileSync(join(base, 'package.json'), 'utf8'));
assert.equal(metadata.name, '@actions/expressions');
const {Lexer, Parser, Evaluator, data} = await import(pathToFileURL(join(base, 'dist/index.js')));
const before = execFileSync('git', ['show', 'aa84dfc247891313dc5dc20847950e91c9528064:.github/workflows/links.yml'], {encoding:'utf8'});
const after = readFileSync('.github/workflows/links.yml', 'utf8');
const expression = source => YAML.parse(source).jobs['link-checker'].steps.find(
  step => step.name === 'Fail if broken links were found').if
  .replace(/^\s*\$\{\{\s*|\s*\}\}\s*$/gu, '');
const rows = [];
for (const [revision, source] of [['before', before], ['after', after]]) {
  const code = expression(source);
  for (const exit of [undefined, null, '', '0', '1', '2', '127', 'unexpected', ' ', '00']) {
    for (const recovered of ['', 'false', 'true']) {
      for (const cancelled of [false, true]) {
        const plain = {steps:{lychee:{outputs:exit === undefined ? {} : {exit_code:exit}},
          recheck:{outputs:{all_recovered:recovered}}}};
        const functions = new Map([['cancelled', {name:'cancelled', minArgs:0, maxArgs:0,
          call:() => new data.BooleanData(cancelled)}]]);
        const tokens = new Lexer(code).lex().tokens;
        const parsed = new Parser(tokens, ['steps'], [...functions.values()]).parse();
        const selected = new Evaluator(parsed, JSON.parse(JSON.stringify(plain), data.reviver), functions).evaluate();
        const actual = selected.value;
        const local = new Script(code).runInNewContext({...plain,cancelled:() => cancelled}, {timeout:100});
        if (typeof exit === 'string') assert.equal(actual, local, JSON.stringify({revision,exit,recovered,cancelled}));
        const expected = !cancelled && exit !== '0' && !(exit === '2' && recovered === 'true');
        if (revision === 'after') assert.equal(actual, expected);
        rows.push({revision,exit:exit === undefined ? '<absent-property>' : exit,recovered,cancelled,selected:actual,expected});
      }
    }
  }
}
const gaps = rows.filter(row => row.revision === 'before' && row.selected !== row.expected);
assert.equal(gaps.length, 15);
assert(rows.filter(row => row.revision === 'after').every(row => row.selected === row.expected));
const report = {engine:{name:metadata.name,version:metadata.version,gitHead:metadata.gitHead},
  comparisons:rows.length, negativeControls:gaps.length, rows,
  actualActionsRunnerExecuted:false, liveLinkHealthVerified:false, independentAuthorshipCredit:0, fullIssueComplete:false};
writeFileSync(output, JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({...report,rows:undefined}));
