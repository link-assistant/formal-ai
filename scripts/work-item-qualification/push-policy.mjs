// The frozen fixture transport supports one owned normal push, never a probe
// silently converted into a mutation or an unrequested branch/ref operation.
export function ownedPushRequest(argumentsValue,policy) {
  if(!Array.isArray(argumentsValue)||!Number.isSafeInteger(policy.issueNumber)||policy.issueNumber<=0)throw Error('InvalidOwnedPushRequest');
  const positional=[];
  for(const value of argumentsValue){
    if(typeof value!=='string'||value.includes('\0'))throw Error('InvalidOwnedPushRequest');
    if(['-u','--set-upstream'].includes(value))continue;
    if(value.startsWith('-'))throw Error('UnsupportedPushOption');
    positional.push(value);
  }
  if(positional.length>2)throw Error('UnsupportedPushRefspec');
  if(positional.length&& !['origin','https://github.com/'+policy.repository,
    'https://github.com/'+policy.repository+'.git','git@github.com:'+policy.repository,
    'git@github.com:'+policy.repository+'.git'].includes(positional[0]))throw Error('UnownedPushRemote');
  const branch=positional[1]??null;
  if(branch!==null&&branch!=='HEAD'&&!new RegExp('^issue-'+policy.issueNumber+'-[A-Za-z0-9-]+$').test(branch))throw Error('UnsupportedPushRefspec');
  return {branch};
}
