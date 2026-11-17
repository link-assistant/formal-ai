"""Supply the shared transport deadline; retain caller cancellation unchanged."""
from pathlib import Path
p = Path('js/server/read-access.mjs')
s = p.read_text().replace('fetchImpl=globalThis.fetch}={}) {', 'fetchImpl=globalThis.fetch,timeoutMs=30_000}={}) {\n  if(!Number.isSafeInteger(timeoutMs)||timeoutMs<=0||timeoutMs>2_147_483_647)throw new Error(\'read_network_timeout\');')
assert 'read_network_timeout' in s
s = s.replace('    return fetchImpl(url,options);', '''    // AbortSignal stays attached through response-body consumption, unlike a
    // timer cleared as soon as headers arrive. Caller cancellation still wins.
    const deadline=AbortSignal.timeout(timeoutMs);
    const signal=options.signal?AbortSignal.any([options.signal,deadline]):deadline;
    return fetchImpl(url,{...options,signal});''')
p.write_text(s)
