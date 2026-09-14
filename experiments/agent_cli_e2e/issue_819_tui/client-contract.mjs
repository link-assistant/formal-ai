export const promptAfterFor = (tool) =>
  ({
    opencode: 'Ask anything...',
    claude: '$',
    codex: 'formal-ai default',
  })[tool];

export const startupInteractionsFor = (tool) =>
  ({
    claude: [{ after: 'Enter y/n:', text: 'y', key: 'ENTER' }],
    codex: [{ after: 'Press enter to continue', key: 'ENTER' }],
  })[tool] ?? [];
