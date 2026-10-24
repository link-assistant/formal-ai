import {readFileSync} from 'node:fs';

const frames = JSON.parse(readFileSync(new URL('./fixtures/codex-startup.json', import.meta.url)));
process.stdin.setRawMode?.(true);
process.stdin.resume();
let state = 'confirmation', input = '', lastCharacterAt = 0;
const deadline = setTimeout(() => process.exit(2), 4000);
process.stdout.write('Press enter to continue');
process.stdin.on('data', chunk => {
  if (state === 'confirmation' && chunk.toString() === '\r') {
    state = 'loading';
    process.stdout.write(frames.loadingFrame);
    setTimeout(() => {state = 'ready'; process.stdout.write(frames.readyFrame);}, 250);
    return;
  }
  if (state !== 'ready') {
    process.stderr.write('Prompt arrived while model was loading\n');
    clearTimeout(deadline); process.exit(1);
  }
  for (const character of chunk.toString()) {
    if (character === '\r') {
      if (Date.now() - lastCharacterAt <= frames.pasteEnterSuppressionMilliseconds) {
        process.stderr.write('Enter arrived inside the paste-burst window\n');
        clearTimeout(deadline); process.exit(1);
      }
      process.stdout.write('\r\nSubmitted: ' + input + '\r\n');
      clearTimeout(deadline); setTimeout(() => process.exit(0), 50);
    } else {input += character; lastCharacterAt = Date.now();}
  }
  process.stdout.write('\r\nDraft: ' + input + '\r\n');
});
