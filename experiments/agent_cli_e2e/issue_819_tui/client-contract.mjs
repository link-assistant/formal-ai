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
