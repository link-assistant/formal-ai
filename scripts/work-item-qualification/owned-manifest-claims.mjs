// Host-only canonical display grammar. No production renderer/parser is imported.
import { supportedManifestClaims } from './manifest-claims.mjs';
const notice = 'These are literal manifest declarations; they do not establish executable runtime use.';
const header = '| Dependency | Declared version | Dependency section | Target qualifier | Literal options |';
const separator = '| --- | --- | --- | --- | --- |';
const semantics = 'literal displayed fields only; executable use, prose roles and whole-task acceptance remain unverified';
const ownerValid = owner => /^[A-Za-z0-9_][A-Za-z0-9_./-]*$/u.test(owner) && !owner.includes('--') && owner.split('/').every(part => part !== '' && part !== '.' && part !== '..');
const canonical = value => Array.isArray(value) ? value.map(canonical) : value !== null && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
export function checkOwnedManifestTables(markdown, facts, observations, {
  expectedOwners = []
} = {}) {
  const results = [],
    blocks = [],
    seen = new Set();
  const unknown = (reason, extra = {}) => results.push({
    status: 'unknown',
    reason,
    ...extra,
    semantics
  });
  if (typeof markdown !== 'string' || Buffer.byteLength(markdown) > 262144) {
    unknown('guide-budget-exceeded');
    return results;
  }
  const lines = markdown.split('\n').map(line => line.endsWith('\r') ? line.slice(0, -1) : line);
  let active = null,
    fence = null,
    invalid = false;
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index],
      hasMarker = /formal-ai-manifest-(?:start|end)/u.test(line);
    const boundary = /^<!-- formal-ai-manifest-(start|end): ([A-Za-z0-9_][A-Za-z0-9_./-]*) -->$/u.exec(line);
    if (hasMarker) {
      if (!boundary || fence !== null || !ownerValid(boundary[2])) {
        unknown('malformed-or-quoted-owned-marker', {
          line: index + 1
        });
        invalid = true;
        continue;
      }
      const [, kind, owner] = boundary;
      if (kind === 'start') {
        if (active !== null || seen.has(owner) || blocks.length >= 16) {
          unknown('nested-duplicate-or-excess-owned-block', {
            owner
          });
          invalid = true;
          continue;
        }
        seen.add(owner);
        active = {
          owner,
          start: index,
          body: []
        };
      } else {
        if (active === null || active.owner !== owner) {
          unknown('unpaired-owned-marker', {
            owner
          });
          invalid = true;
          continue;
        }
        blocks.push(active);
        active = null;
      }
      continue;
    }
    if (active !== null) {
      active.body.push(line);
      continue;
    }
    const delimiter = /^ {0,3}(`{3,}|~{3,})/u.exec(line);
    if (delimiter) {
      const token = delimiter[1];
      if (fence === null) fence = token;else if (token[0] === fence[0] && token.length >= fence.length) fence = null;
    }
  }
  if (active !== null) {
    unknown('unterminated-owned-marker', {
      owner: active.owner
    });
    invalid = true;
  }
  if (invalid) return results;
  for (const owner of expectedOwners) {
    if (!seen.has(owner)) results.push({
      owner,
      status: 'missing',
      reason: 'owned-manifest-block-missing',
      semantics
    });
  }
  const pending = [];
  for (const block of blocks) {
    const {
      owner,
      body
    } = block;
    const source = observations.find(row => row.path === owner && row.status === 'observed');
    if (!source) {
      unknown('complete-frozen-manifest-observation-required', {
        owner
      });
      continue;
    }
    if (body[0] !== `Dependency declarations from \`${owner}\`:` || body[1] !== notice || body[2] !== header || body[3] !== separator) {
      unknown('noncanonical-owned-table-preamble', {
        owner
      });
      continue;
    }
    if (body.length === 4) {
      results.push({
        owner,
        status: 'missing',
        reason: 'owned-manifest-rows-missing',
        semantics
      });
      continue;
    }
    const identities = new Set();
    for (const line of body.slice(4)) {
      if (pending.length >= 32) {
        unknown('owned-table-row-budget');
        return results;
      }
      const cells = /^\| `([^`|\r\n]*)` \| `([^`|\r\n]*)` \| `([^`|\r\n]*)` \| `([^`|\r\n]*)` \| `([^`|\r\n]*)` \|$/u.exec(line);
      if (!cells) {
        unknown('noncanonical-owned-table-row', {
          owner
        });
        continue;
      }
      const [, name, version, section, target, options] = cells;
      const identity = JSON.stringify([name, section, target]);
      if (identities.has(identity)) {
        unknown('duplicate-displayed-declaration', {
          owner,
          name,
          section,
          target
        });
        continue;
      }
      identities.add(identity);
      const candidates = facts.filter(row => row.kind === 'manifest-declaration' && row.manifest === owner && row.sourceIdentity === source.sourceIdentity && row.name === name && row.section === section && (row.version ?? 'not declared') === version && (row.target ?? 'unqualified') === target);
      if (candidates.length === 0) {
        results.push({
          owner,
          name,
          section,
          target,
          sourceIdentity: source.sourceIdentity,
          status: 'contradicted',
          reason: 'displayed-declaration-fields-not-in-frozen-manifest',
          semantics
        });
        continue;
      }
      if (candidates.length !== 1) {
        unknown('ambiguous-display-sentinel-binding', {
          owner,
          name,
          section,
          target
        });
        continue;
      }
      const fact = candidates[0],
        expectedOptions = JSON.stringify(canonical(fact.options ?? {}));
      // Keep the literal options bytes in a strict Python-parsed typed claim;
      // duplicate JSON keys never disappear through a local JSON.parse.
      const typed = {
        kind: fact.kind,
        manifest: owner,
        sourceIdentity: source.sourceIdentity,
        section,
        name,
        version: fact.version,
        ...(Object.hasOwn(fact, 'target') ? {
          target: fact.target
        } : {})
      };
      const prefix = JSON.stringify(typed).slice(0, -1);
      pending.push({
        owner,
        name,
        section,
        target,
        sourceIdentity: source.sourceIdentity,
        options,
        expectedOptions,
        block: '```formal-ai-source-claim\n' + prefix + ',"options":' + options + '}\n```',
        fact
      });
    }
  }
  const checked = pending.length ? supportedManifestClaims(pending.map(row => row.block).join('\n'), facts.map(fact => fact.kind === 'manifest-declaration' && !Object.hasOwn(fact, 'options') ? {
    ...fact,
    options: {}
  } : fact), observations) : [];
  for (let index = 0; index < pending.length; index++) {
    const row = pending[index],
      claim = checked[index];
    const status = claim?.status === 'verified-literal-declaration' && row.options === row.expectedOptions ? 'verified-literal-declaration' : claim?.status === 'contradicted' ? 'contradicted' : 'unknown';
    results.push({
      owner: row.owner,
      name: row.name,
      section: row.section,
      target: row.target,
      sourceIdentity: row.sourceIdentity,
      status,
      reason: status === 'verified-literal-declaration' ? 'exact-canonical-displayed-fields' : claim?.status === 'verified-literal-declaration' ? 'canonical-options-required' : claim?.reason ?? 'independent-claim-check-unavailable',
      semantics
    });
  }
  return results;
}
