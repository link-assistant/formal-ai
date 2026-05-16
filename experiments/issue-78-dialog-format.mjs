// Issue #78: smoke-test the compact `U:`/`A:` dialog block.
// Mirrors `appendDialogBlock` from src/web/app.js (kept in sync by hand —
// the function lives in a React-coupled file that is not Node-importable
// without a bundler).
//
// Usage: node experiments/issue-78-dialog-format.mjs

function pickDialogFence(messages) {
  let fence = "```";
  while (messages.some((message) => String(message.content ?? "").includes(fence))) {
    fence += "`";
  }
  return fence;
}

function appendDialogBlock(lines, messages, effectiveFocus) {
  if (messages.length === 0) {
    lines.push("No messages have been sent yet.");
    return;
  }
  lines.push("Legend: `U` = user, `A` = agent.");
  lines.push("");
  const fence = pickDialogFence(messages);
  lines.push(fence);
  messages.forEach((message) => {
    const prefix = message.role === "user" ? "U" : "A";
    const annotations = [];
    if (message.intent === "unknown") {
      annotations.push(`intent: ${message.intent}`);
      if (effectiveFocus && effectiveFocus.id === message.id) {
        annotations.push("reported");
      }
    }
    const head = annotations.length > 0 ? `${prefix} (${annotations.join(", ")})` : prefix;
    const content = String(message.content ?? "");
    const [first, ...rest] = content.split("\n");
    lines.push(`${head}: ${first}`);
    rest.forEach((row) => lines.push(`   ${row}`));
  });
  lines.push(fence);
}

const cases = [
  {
    name: "empty conversation",
    messages: [],
    focus: null,
  },
  {
    name: "simple greeting",
    messages: [
      { id: "1", role: "user", content: "Hi" },
      { id: "2", role: "assistant", content: "Hi, how may I help you?" },
    ],
    focus: null,
  },
  {
    name: "unknown prompt with focus + intent",
    messages: [
      { id: "1", role: "user", content: "Quxblort fnordwarble plimsy gabble what?" },
      {
        id: "2",
        role: "assistant",
        intent: "unknown",
        content: "I do not have a learned symbolic rule for that prompt yet.",
      },
    ],
    focus: { id: "2" },
  },
  {
    name: "multi-line content + triple backtick",
    messages: [
      { id: "1", role: "user", content: "Write hello world in Rust" },
      {
        id: "2",
        role: "assistant",
        content: "rust hello world example:\n```rust\nfn main() { println!(\"Hello, world!\"); }\n```",
      },
    ],
    focus: null,
  },
  {
    name: "arithmetic dialogue (matches the issue example)",
    messages: [
      { id: "1", role: "user", content: "Hi" },
      { id: "2", role: "assistant", content: "Hello" },
      { id: "3", role: "user", content: "1+2" },
      { id: "4", role: "assistant", content: "3" },
    ],
    focus: null,
  },
];

for (const c of cases) {
  console.log(`\n=== ${c.name} ===`);
  const lines = [];
  appendDialogBlock(lines, c.messages, c.focus);
  console.log(lines.join("\n"));
}
