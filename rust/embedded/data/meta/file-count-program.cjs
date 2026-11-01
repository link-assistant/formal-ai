// Bounded immediate regular-file count; names are never printed or evaluated.
const fs = require('node:fs');
const directory = fs.opendirSync(process.argv[1]);
let visited = 0;
let count = 0;
try {
  for (;;) {
    const entry = directory.readSync();
    if (entry === null) break;
    if (++visited > 4096) throw Error('FileCountLimit');
    if (entry.isFile()) count++;
  }
} finally {
  directory.closeSync();
}
console.log(count);
