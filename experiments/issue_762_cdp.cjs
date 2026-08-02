const fs = require("fs");
const WebSocket = require("../node_modules/ws");

async function main() {
  const targets = await fetch("http://127.0.0.1:9222/json/list").then((response) => response.json());
  const page = targets.find((target) => target.type === "page");
  if (!page) throw new Error("OpenCode renderer target not found");

  const socket = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.once("open", resolve);
    socket.once("error", reject);
  });
  let id = 0;
  const pending = new Map();
  socket.on("message", (message) => {
    const response = JSON.parse(message.toString());
    if (!response.id) return;
    const request = pending.get(response.id);
    pending.delete(response.id);
    if (response.error) request.reject(new Error(JSON.stringify(response.error)));
    else request.resolve(response.result);
  });
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const requestId = ++id;
    pending.set(requestId, { resolve, reject });
    socket.send(JSON.stringify({ id: requestId, method, params }));
  });

  await send("Runtime.enable");
  const expression = process.argv[2] || "document.body.innerText";
  const result = await send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
  const value = result.result.value;
  if (process.env.CDP_SCREENSHOT) {
    await send("Page.enable");
    const screenshot = await send("Page.captureScreenshot", { format: "png" });
    fs.writeFileSync(process.env.CDP_SCREENSHOT, Buffer.from(screenshot.data, "base64"));
  }
  if (typeof value === "string") process.stdout.write(value);
  else process.stdout.write(JSON.stringify(value, null, 2));
  socket.close();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
