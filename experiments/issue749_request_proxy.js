// Diagnostic reverse proxy for issue #749 external CLI request shapes.
// Usage: node experiments/issue749_request_proxy.js
import http from "node:http";
import fs from "node:fs";

const listenPort = Number(process.env.ISSUE749_PROXY_PORT || "8791");
const upstreamPort = Number(process.env.ISSUE749_UPSTREAM_PORT || "8790");
const output = process.env.ISSUE749_PROXY_LOG || "/tmp/issue749-requests.jsonl";

http.createServer((request, response) => {
  const chunks = [];
  request.on("data", (chunk) => chunks.push(chunk));
  request.on("end", () => {
    const body = Buffer.concat(chunks);
    fs.appendFileSync(output, JSON.stringify({
      method: request.method,
      url: request.url,
      body: body.toString("utf8"),
    }) + "\n");

    const upstream = http.request({
      hostname: "127.0.0.1",
      port: upstreamPort,
      method: request.method,
      path: request.url,
      headers: { ...request.headers, host: `127.0.0.1:${upstreamPort}` },
    }, (upstreamResponse) => {
      response.writeHead(upstreamResponse.statusCode, upstreamResponse.headers);
      upstreamResponse.pipe(response);
    });
    upstream.on("error", (error) => {
      response.writeHead(502, { "content-type": "text/plain" });
      response.end(error.message);
    });
    upstream.end(body);
  });
}).listen(listenPort, "127.0.0.1", () => {
  process.stderr.write(`issue749 proxy listening on ${listenPort}\n`);
});
