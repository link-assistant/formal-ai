import { readdirSync, readFileSync } from 'node:fs';

const directory = new URL('../../fixtures/source-qualified-definition/source-cache/', import.meta.url);

export function sourceDefinitionCaptures() {
  return readdirSync(directory).filter(name => name.endsWith('.meta')).map(name => {
    const [, ...entries] = readFileSync(new URL(name, directory), 'utf8').trimEnd().split('\n');
    const fields = Object.fromEntries(entries.map(entry => {
      const separator = entry.indexOf('=');
      return [entry.slice(0, separator), entry.slice(separator + 1)];
    }));
    const { url, sha256, fetched_at: fetchedAt } = fields;
    const text = readFileSync(new URL(`objects/${sha256}.body`, directory), 'utf8');
    return { ok: true, url, text, sha256, fetchedAt, cached: true };
  });
}

export async function installSourceDefinitionCaptures(host) {
  await host.run('for (const capture of __captures) sourceWalkCaptureCache.set(capture.url, capture)', {
    __captures: sourceDefinitionCaptures(),
  });
}
