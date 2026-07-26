process.stdin.setRawMode?.(true);
process.stdin.resume();
process.stdout.write('\u001b[2J\u001b[HChoose reports');

let selected = '';
for await (const chunk of process.stdin) {
  for (const input of chunk.toString()) {
    if (/[123]/u.test(input)) selected += input;
    if (input === '\r') {
      process.stdout.write(
        `\u001b[2J\u001b[HChoose reports\nSubmitted: ${selected}`,
      );
      process.exit(0);
    }
  }
}
