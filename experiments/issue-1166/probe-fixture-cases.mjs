// Probe: print specifyTask output/meaning for the acceptance fixture candidates.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const { specifyTask } = await import('../../js/agentic/task_acceptance.mjs');
const tasks = process.argv.slice(2).length ? process.argv.slice(2) : [
  'Write a Python program that prints `Hi`. The output must be `Hi`.',
  'Write a Python program that prints `Hi` twice.',
  'Write a Python program that prints `Hi` and `Hi`.',
  'Write a Python program that prints `C:\\temp\\ünï "q"` twice.',
  'Write a Python program that prints "Привет, 世界!".',
  'Write a Python program that prints "a\\"b".',
  'Write a Python program. When x is greater than y, print "big", otherwise print "small".',
  'Write a Python program. Print "big" if y is less than x, else print "small".',
  'Write a Python program. Print "big" when x > y; otherwise print "small".',
  'Write a Python program. When x is less than y, print "big", otherwise print "small".',
  'Write a Python program. Print "yes" when a is less than b and b is less than c.',
  'Write a Python program. Print "yes" when a is less than b or b is less than c.',
  'Write a Python program. Print "yes" unless a is less than b.',
  'Write a Python program. Print "yes" when a is not less than b.',
  'Write a Python program. Print "yes" when a is roughly bigger than b.',
];
for (const task of tasks) {
  const s = specifyTask(task);
  console.log(JSON.stringify(task));
  console.log('  output', JSON.stringify(s.output));
  console.log('  meaning', s.meaning);
  for (const c of s.clauses) console.log('   ', c.span.join(':'), JSON.stringify(c.checks));
}
