export const promptAfterFor = (tool) =>
  ({
    opencode: 'Ask anything...',
    claude: '$',
    // The footer appears while the model is loading; Enter is ignored then.
    // Match the selected model in the banner, allowing its ANSI formatting.
    codex: /model:(?:[ \t]|\u001b\[[0-?]*[ -/]*[@-~])*formal-ai(?=[ \t\u001b│]|$)/u,
  })[tool];

export const startupInteractionsFor = (tool) =>
  ({
    claude: [{ after: 'Enter y/n:', text: 'y', key: 'ENTER' }],
    codex: [{ after: 'Press enter to continue', key: 'ENTER' }],
  })[tool] ?? [];

export const promptInteractionsFor = (tool, prompt) => {
  const after = promptAfterFor(tool);
  // Codex 0.147 treats Enter within 120 ms of a text burst as pasted text.
  // Require its rendered draft, then quiet output, inside the capture cap.
  if (tool !== 'codex') return [{ after, text: prompt, key: 'ENTER' }];
  const words = prompt.trim().split(/\s+/u).map(word => word.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&'));
  const draft = new RegExp(words.join('(?:\\s|\\u001b\\[[0-?]*[ -/]*[@-~])+'), 'u');
  return [{ after, text: prompt }, { after: draft, idleMilliseconds: 200, key: 'ENTER' }];
};
