// The complete readable CST is primary; grammar providers are explicit and content-bound.
import { parentPort, workerData } from 'node:worker_threads';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { createSourceCstPacket } from './source-cst-packets.mjs';
import { sourceCstFrontends } from './source-cst-frontends.mjs';
const frontends = await sourceCstFrontends(workerData.cstRuntimeDirectory);
const decoder = new TextDecoder('utf-8', {
  fatal: true,
  ignoreBOM: true
});
parentPort.on('message', ({
  index,
  path
}) => {
  try {
    const started = performance.now(),
      sourceBytes = readFileSync(join(workerData.root, path)),
      source = decoder.decode(sourceBytes);
    if (!Buffer.from(source, 'utf8').equals(sourceBytes)) throw Error('SourceEncodingMismatch:' + path);
    parentPort.postMessage({
      event: 'source-network-start',
      shardIndex: workerData.shardIndex,
      path,
      sourceBytes: sourceBytes.length
    });
    const language = path.endsWith('.rs') ? 'Rust' : /\.(?:ts|mts|tsx)$/u.test(path) ? 'TypeScript' : 'JavaScript';
    const frontend = frontends.forSource(path, language);
    const {
      compressed,
      receipt
    } = createSourceCstPacket({
      path,
      source,
      language
    }, frontend);
    receipt.frontend = frontend.provenance;
    if (!readFileSync(join(workerData.root, path)).equals(sourceBytes)) throw Error('SourceChangedDuringCst:' + path);
    if (workerData.output) {
      const destination = join(resolve(workerData.output), receipt.packet);
      mkdirSync(dirname(destination), {
        recursive: true
      });
      writeFileSync(destination, compressed, {
        flag: 'wx'
      });
    }
    parentPort.postMessage({
      event: 'source-network-complete',
      shardIndex: workerData.shardIndex,
      path,
      elapsedMilliseconds: performance.now() - started,
      serializedBytes: receipt.serializedBytes,
      compressedBytes: compressed.length
    });
    parentPort.postMessage({
      index,
      receipt
    });
  } catch (error) {
    parentPort.postMessage({
      error: error.message
    });
  }
});
