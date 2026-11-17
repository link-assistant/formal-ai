// Run through the maintained Formal AI tool host. Supplied commands earn no authorship credit.
import fs from 'node:fs';
import {spawn} from 'node:child_process';
const [mode, filename, ...command] = process.argv.slice(2);
if (mode === 'start') {
  fs.rmSync(filename + '.exit', {force: true});
  const descriptor = fs.openSync(filename + '.log', 'w');
  const child = spawn(process.execPath, [import.meta.filename, 'run', filename, ...command], {
    detached: true, stdio: ['ignore', descriptor, descriptor],
  });
  fs.writeFileSync(filename + '.pid', String(child.pid));
  child.unref();
  console.log(JSON.stringify({pid: child.pid, log: filename + '.log', exit: filename + '.exit'}));
} else if (mode === 'run') {
  const child = spawn(command[0], command.slice(1), {stdio: 'inherit'});
  child.on('error', error => {fs.writeFileSync(filename + '.exit', '1'); console.error(error.message);});
  child.on('exit', (code, signal) => {
    fs.writeFileSync(filename + '.exit', String(code ?? 1));
    if (signal) console.error('Terminated by ' + signal);
  });
} else throw new Error('Expected start or run');
