// Explicit optional physical provider; never promote caller receipt metadata.
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const adapter = fileURLToPath(new URL('./exclusive-create.py', import.meta.url));

export function exclusiveCreate(root, path, content, rootIdentity = undefined) {
  const result = spawnSync('python3', [adapter], {
    input: JSON.stringify({ root, path, content, root_identity: rootIdentity }), encoding: 'utf8',
    timeout: 30000, maxBuffer: 65536,
  });
  let receipt;
  try { receipt = JSON.parse(result.stdout); }
  catch { receipt = null; }
  if (result.error || result.signal || result.status !== 0
      || receipt?.success !== true || receipt.path !== path
      || receipt.bytes !== Buffer.byteLength(content, 'utf8')) {
    return { content: JSON.stringify({ is_error: true,
      error: result.error?.message ?? receipt?.message ?? 'ExclusiveCreationUnsupported' }),
      is_error: true, source_creation: { path, success: false, exclusive: true,
        error_code: receipt?.code, external_namespace_stability: 'unknown' } };
  }
  return { content: '', source_creation: { path, success: true, exclusive: true,
    no_follow: true, bytes: receipt.bytes, directory_scope: 'bound-descriptors',
    external_namespace_stability: 'unknown' } };
}
