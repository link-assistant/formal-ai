// Read-only network access is available by default (#1582). This grants no
// command execution, filesystem writes, or permission to override a caller deny.
const truth=value=>['1','true','yes','on'].includes(String(value).trim().toLowerCase());
const falsity=value=>['0','false','no','off'].includes(String(value).trim().toLowerCase());
export function readNetworkAllowed(env={}) {
  if(env.FORMAL_AI_OFFLINE!==undefined&&!falsity(env.FORMAL_AI_OFFLINE))return false;
  if(['denied','closed','off','0','false'].includes(String(env.FORMAL_AI_NETWORK??'').trim().toLowerCase()))return false;
  return env.FORMAL_AI_LIVE_FETCH===undefined||truth(env.FORMAL_AI_LIVE_FETCH);
}

// The serving worker can read HTTP(S) sources without acquiring write access.
export function createReadOnlyFetch({env=process.env,fetchImpl=globalThis.fetch,timeoutMs=30_000}={}) {
  if(!Number.isSafeInteger(timeoutMs)||timeoutMs<=0||timeoutMs>2_147_483_647)throw new Error('read_network_timeout');
  return (url,options={})=>{
    if(!readNetworkAllowed(env))return Promise.reject(new Error('read_network_denied'));
    const parsed=new URL(url);
    if(!['http:','https:'].includes(parsed.protocol))return Promise.reject(new Error('read_network_protocol'));
    if(!['GET','HEAD'].includes(String(options.method??'GET').toUpperCase()))return Promise.reject(new Error('read_network_method'));
    // AbortSignal stays attached through response-body consumption, unlike a
    // timer cleared as soon as headers arrive. Caller cancellation still wins.
    const deadline=AbortSignal.timeout(timeoutMs);
    const signal=options.signal?AbortSignal.any([options.signal,deadline]):deadline;
    return fetchImpl(url,{...options,signal});
  };
}
