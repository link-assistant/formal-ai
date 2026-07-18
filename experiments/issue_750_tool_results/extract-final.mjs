import fs from "node:fs";

const [input, output] = process.argv.slice(2);
const messages = fs
  .readFileSync(input, "utf8")
  .trim()
  .split("\n")
  .filter((line) => line.startsWith("{"))
  .map((line) => JSON.parse(line))
  .filter((event) => event.type === "message" && event.role === "assistant");

if (messages.length === 0) {
  throw new Error("Agent stream did not contain a final assistant message");
}

const text = messages
  .at(-1)
  .content.filter((part) => part.type === "text")
  .map((part) => part.text)
  .join("");
fs.writeFileSync(output, `${text}\n`);
