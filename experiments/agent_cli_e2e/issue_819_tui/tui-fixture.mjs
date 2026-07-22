const pause = () => new Promise((resolve) => setTimeout(resolve, 15));

process.stdout.write('\u001b[2J\u001b[HUser: Find archive folder on my desktop');
await pause();
process.stdout.write('\u001b[2J\u001b[HUser: Find archive folder on my desktop');
await pause();
process.stdout.write(
  '\u001b[2J\u001b[HUser: Find archive folder on my desktop\nTool: find "$HOME/Desktop" -type d',
);
await pause();
process.stdout.write(
  '\u001b[2J\u001b[HUser: Find archive folder on my desktop\nTool: find "$HOME/Desktop" -type d\nResult: /tmp/Desktop/archive',
);
