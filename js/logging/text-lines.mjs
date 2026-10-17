// Stateless physical line splitting, preserving the existing public Rust-compatible API.
export function rustLines(text) {
  const lines = text.split('\n');
  const terminated = lines.length > 0 && lines[lines.length - 1] === '';
  if (terminated) lines.pop();
  return lines.map((line, index) => (line.endsWith('\r') && (terminated || index < lines.length - 1)
    ? line.slice(0, -1) : line));
}
