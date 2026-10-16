// Exact private driver evidence. No autonomous authorship, native-memory or replay claim.
import { AsyncLocalStorage } from 'node:async_hooks';
import fs from 'node:fs';
import path from 'node:path';
import { createHash, randomUUID } from 'node:crypto';
import { provenancePrimitives, rawBytes } from './session.mjs';
import { blake3Hex } from '../crypto/blake3.mjs';
import { encodeTyped, decodeTyped } from '../server/structured-log.mjs';
const contexts = new AsyncLocalStorage();
const primitives = provenancePrimitives({
  encodeTyped,
  decodeTyped
});
const wire = value => Buffer.from(JSON.stringify(value), 'utf8');
const digest = bytes => blake3Hex(bytes);
export const activeDriveCapture = () => {
  const current = contexts.getStore();
  return current?.active ? current : null;
};
export async function withDriveCapture(options, workspace, prompt, sourceId, run) {
  let ancestor = path.resolve(options.directory);
  const suffix = [];
  while (!fs.existsSync(ancestor)) {
    suffix.unshift(path.basename(ancestor));
    const parent = path.dirname(ancestor);
    if (parent === ancestor) throw Error('provenance_directory_unresolvable');
    ancestor = parent;
  }
  const parent = path.join(fs.realpathSync(ancestor), ...suffix),
    working = fs.realpathSync(workspace);
  if (parent === working || parent.startsWith(working + path.sep)) throw Error('provenance_evidence_must_be_outside_workspace');
  fs.mkdirSync(parent, {
    recursive: true,
    mode: 0o700
  });
  const directory = fs.mkdtempSync(path.join(parent, 'session-'));
  fs.chmodSync(directory, 0o700);
  const sessionIdentifier = randomUUID();
  const initial = wire({
    scope: 'driver-dialogue-only',
    prompt
  });
  const session = primitives.createSession({
    sessionId: sessionIdentifier,
    contextId: 'js-dogfood-drive',
    initialMemory: initial,
    runtimeId: process.version,
    sourceId
  });
  const chain = path.join(directory, 'chain.lino');
  fs.writeFileSync(chain, session.lines().join('\n') + '\n', {
    mode: 0o600,
    flag: 'wx'
  });
  let memoryHash = digest(initial),
    state = memoryHash,
    output = null,
    payloadIndex = 0;
  const append = (kind, payload) => {
    const record = session.append(kind, payload);
    fs.appendFileSync(chain, record.line + '\n');
    return record.hash;
  };
  function retain(value) {
    const bytes = wire(value),
      name = String(payloadIndex++).padStart(6, '0') + '.lino';
    const encoded = encodeTyped(rawBytes(bytes));
    fs.writeFileSync(path.join(directory, name), encoded + '\n', {
      mode: 0o600,
      flag: 'wx'
    });
    return {
      file: name,
      blake3: digest(bytes),
      size: bytes.length
    };
  }
  const capture = {
    active: true,
    input(messages, tools) {
      const evidence = retain({
        messages,
        tools
      });
      const previousMemoryHash = memoryHash;
      memoryHash = evidence.blake3;
      const deltaHash = digest(wire({
        previousMemoryHash,
        currentMemoryHash: memoryHash
      }));
      state = digest(Buffer.concat([Buffer.from(state, 'hex'), Buffer.from(deltaHash, 'hex')]));
      append('message_in', {
        scope: 'driver-dialogue-only',
        evidence,
        previousMemoryHash,
        memoryHash,
        deltaHash,
        state
      });
    },
    plannerResponse(plan) {
      append('planner_response', {
        evidence: retain(plan)
      });
    },
    output(plan) {
      output = append('message_out', {
        scope: 'driver-dispatched-plan',
        evidence: retain(plan),
        memoryHash,
        state
      });
      const hasTrace = plan !== null && typeof plan === 'object' && (Object.hasOwn(plan, 'reasoning') || Object.hasOwn(plan, 'reasoning_content') || Object.hasOwn(plan, 'derivation'));
      append('thinking', {
        messageOut: output,
        status: hasTrace ? 'captured-fields' : 'unavailable',
        evidence: retain({
          reasoning: plan?.reasoning ?? null,
          reasoning_content: plan?.reasoning_content ?? null,
          derivation: plan?.derivation ?? null
        })
      });
    },
    before(file) {
      if (!output) throw Error('message_out_required');
      try {
        const status = fs.lstatSync(file);
        if (!status.isFile() || status.isSymbolicLink()) return {
          refused: true
        };
        return {
          bytes: fs.readFileSync(file),
          messageOut: output
        };
      } catch (error) {
        if (error.code === 'ENOENT') return {
          bytes: null,
          messageOut: output
        };
        throw error;
      }
    },
    after(file, relative, before) {
      if (before.refused) return;
      let after;
      try {
        if (!fs.lstatSync(file).isFile() || fs.lstatSync(file).isSymbolicLink()) return;
        after = fs.readFileSync(file);
      } catch (error) {
        if (error.code === 'ENOENT') return;
        throw error;
      }
      if (before.bytes !== null && before.bytes.equals(after)) return;
      const blob = bytes => bytes === null ? null : {
        blake3: digest(bytes),
        gitBlobSha1: createHash('sha1').update(Buffer.from('blob ' + bytes.length + '\0')).update(bytes).digest('hex'),
        size: bytes.length
      };
      append('edit', {
        path: relative,
        preimage: blob(before.bytes),
        postimage: blob(after),
        messageOut: before.messageOut,
        scope: 'observed-driver-write-or-edit'
      });
    },
 };
  try {
    const result = await contexts.run(capture, run);
    append('terminal', {
      reason: result.stop ?? 'unknown'
    });
    const anchor = session.anchor();
    fs.writeFileSync(path.join(directory, 'anchor.lino'), encodeTyped(anchor) + '\n', {
      mode: 0o600,
      flag: 'wx'
    });
    return {
      ...result,
      provenance: {
        directory,
        anchor,
        scope: 'driver-dialogue-and-direct-write-edit-observations',
        authorship: 'not-established',
        replay: 'not-performed'
      }
    };
  } catch (error) {
    try {
      append('failure', {
        name: String(error?.name ?? 'ThrownValue'),
        messageHash: digest(Buffer.from(String(error?.message ?? error)))
      });
      append('terminal', {
        reason: 'exception'
      });
      fs.writeFileSync(path.join(directory, 'anchor.lino'), encodeTyped(session.anchor()) + '\n', {
        mode: 0o600,
        flag: 'wx'
      });
    } catch {/* A storage failure leaves incomplete evidence; never replace the original failure. */}
    throw error;
  } finally {
    capture.active = false;
  }
}
