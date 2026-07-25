process.stdin.setRawMode?.(true);
process.stdin.resume();
process.stdout.write('\u001b[2J\u001b[HChoose reports');

let selected = '';
const initialDimensions = `${process.stdout.columns}x${process.stdout.rows}`;
for await (const chunk of process.stdin) {
  for (const input of chunk.toString()) {
    if (/[123]/u.test(input)) selected += input;
    if (input === '\r') {
      process.stdout.write(
        `\u001b[2J\u001b[HChoose reports\nSubmitted: ${selected}\nwaiting-resize`,
      );
      const resizeWatcher = setInterval(() => {
        const dimensions = `${process.stdout.columns}x${process.stdout.rows}`;
        if (dimensions === initialDimensions) return;
        clearInterval(resizeWatcher);
        process.stdout.write(
          `\u001b[2J\u001b[HChoose reports\nSubmitted: ${selected}\nresized:${dimensions}`,
        );
        setTimeout(() => process.exit(0), 20);
      }, 5);
    }
  }
}
