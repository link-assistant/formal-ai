#!/usr/bin/env node
// Issue #1632: tiny capture proxy between Claude Code and a formal-ai server.
// Dumps every request body (and the upstream status/elapsed time) to a
// directory so the auto-mode classifier / small-fast-model requests can be
// inspected. Usage:
//   node experiments/issue-1632-claude-capture-proxy.mjs <listenPort> <upstreamPort> <outDir>
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';

const [listenPort = '18093', upstreamPort = '18092', outDir = '/tmp/claude-capture'] =
  process.argv.slice(2);
fs.mkdirSync(outDir, { recursive: true });
let counter = 0;

http
  .createServer((req, res) => {
    const chunks = [];
    req.on('data', (chunk) => chunks.push(chunk));
    req.on('end', () => {
      const id = String(++counter).padStart(3, '0');
      const body = Buffer.concat(chunks);
      fs.writeFileSync(path.join(outDir, `${id}-request.json`), body);
      const started = Date.now();
      const upstream = http.request(
        {
          host: '127.0.0.1',
          port: Number(upstreamPort),
          method: req.method,
          path: req.url,
          headers: { ...req.headers, host: `127.0.0.1:${upstreamPort}` },
        },
        (upstreamRes) => {
          const out = [];
          upstreamRes.on('data', (chunk) => out.push(chunk));
          upstreamRes.on('end', () => {
            const payload = Buffer.concat(out);
            fs.writeFileSync(path.join(outDir, `${id}-response.txt`), payload);
            fs.appendFileSync(
              path.join(outDir, 'index.log'),
              `${id} ${req.method} ${req.url} -> ${upstreamRes.statusCode} in ${Date.now() - started}ms\n`,
            );
            res.writeHead(upstreamRes.statusCode, upstreamRes.headers);
            res.end(payload);
          });
        },
      );
      upstream.on('error', (error) => {
        fs.appendFileSync(path.join(outDir, 'index.log'), `${id} upstream error ${error.message}\n`);
        res.writeHead(502);
        res.end();
      });
      upstream.end(body);
    });
  })
  .listen(Number(listenPort), '127.0.0.1');
