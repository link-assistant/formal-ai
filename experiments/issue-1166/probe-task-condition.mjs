// Probe: the task acceptance condition reader over conditional output clauses.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { conditionalStatement, conditionTable, taskLiterals } from '../../js/agentic/task_condition.mjs';
await installNodeHost(new WorkerHost());
const clauses = process.argv.slice(2).length ? process.argv.slice(2) : [
  'When x is greater than y, print "big", otherwise print "small".',
  'Print "big" if y is less than x, else print "small".',
  'Print "big" when x > y; otherwise print "small".',
  'Print "yes" when a is less than b and b is less than c.',
  'Print "yes" when a is less than b or b is less than c.',
  'Print "yes" unless a is less than b.',
  'Print "yes" when a is not less than b.',
  'Print "yes" when a is less than b and c.',
  'Print "yes" when a is roughly bigger than b.',
  'Если x больше y, выведи "big", иначе выведи "small".',
  'Si x es mayor que y, imprime "big", de lo contrario imprime "small".',
  '如果 x 大于 y，打印 "big"，否则打印 "small"。',
  'When the program runs, it prints `Ready`.',
  'Print "a\\"b" now.',
  'Print `C:\\temp\\ünï "q"` twice.',
];
for (const clause of clauses) {
  const statement = conditionalStatement(clause);
  const table = statement && !statement.gap && !statement.continues ? conditionTable([statement]) : null;
  console.log(JSON.stringify(clause), '=>', JSON.stringify(statement), table ? JSON.stringify(table.rows.map((row) => row.args.join(',') + ':' + row.lines.join('|')).join(' ')) : '', JSON.stringify(taskLiterals(clause).map((l) => [l.text, l.escaped])));
}
