// Each worker retains its pinned grammar runtime across independent complete source files.
import { parentPort, workerData } from 'node:worker_threads';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createSourcePacket } from './source-network-packets.mjs';

const { LinkNetwork } = await import(pathToFileURL(join(workerData.upstream, 'js/src/index.js')).href);
const decoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
parentPort.on('message', ({ index, path }) => {
  try {
    const started = performance.now();
    const sourceBytes = readFileSync(join(workerData.root, path)), source = decoder.decode(sourceBytes);
    if (!Buffer.from(source, 'utf8').equals(sourceBytes)) throw new Error('source is not losslessly decoded: ' + path);
    parentPort.postMessage({ event: 'source-network-start', shardIndex: workerData.shardIndex, path,
      sourceBytes: sourceBytes.length });
    const language = path.endsWith('.rs') ? 'rust' : /\.(?:ts|mts|tsx)$/u.test(path) ? 'TypeScript' : 'JavaScript';
    const { compressed, receipt } = createSourcePacket({ path, source, language }, LinkNetwork);
    if (!readFileSync(join(workerData.root, path)).equals(sourceBytes)) throw new Error('source changed during serialization: ' + path);
    if (workerData.output) {
      const destination = join(resolve(workerData.output), receipt.packet);
      mkdirSync(dirname(destination), { recursive: true });
      writeFileSync(destination, compressed);
    }
    parentPort.postMessage({ event: 'source-network-complete', shardIndex: workerData.shardIndex, path,
      elapsedMilliseconds: performance.now() - started, serializedBytes: receipt.serializedBytes,
      compressedBytes: compressed.length });
    parentPort.postMessage({ index, receipt });
  } catch (error) { parentPort.postMessage({ error: error.message }); }
});
