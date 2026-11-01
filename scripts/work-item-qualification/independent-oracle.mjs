// DRAFT private trusted oracle. Import only on the host; never mount in solver container.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {literalManifestFacts,supportedManifestClaims} from './manifest-claims.mjs';
const sha=value=>createHash('sha256').update(value).digest('hex');
export async function candidateReader({api,repository,resource}){
  const pr=await api('repos/'+repository+'/pulls/'+resource.pullRequestNumber);
  assert.equal(pr.base.ref,resource.baseBranch);assert.equal(pr.head.sha,resource.head);
  assert.equal(pr.head.repo.full_name,repository);assert.equal(pr.merged_at,null);
  const files=await api('repos/'+repository+'/pulls/'+pr.number+'/files?per_page=100');
  assert.equal(files.length,pr.changed_files);assert(files.length<100);
  const tree=await api('repos/'+repository+'/git/trees/'+resource.head+'?recursive=1');assert.equal(tree.truncated,false);
  const read=async name=>{
    const member=tree.tree.find(row=>row.path===name);assert(member&&member.type==='blob'&&member.mode==='100644');
    const blob=await api('repos/'+repository+'/git/blobs/'+member.sha);assert.equal(blob.encoding,'base64');
    const bytes=Buffer.from(blob.content,'base64');assert(bytes.length<=262144);
    assert.equal(createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex'),member.sha);
    return bytes.toString();
  };
  return {pr,files,tree,read};
}
export async function checkDocumentationCandidate({api,repository,resource,originalFiles,guide,contradiction=false,kind='original-components'}){
  const rows=[];
  const check=(id,condition)=>rows.push({id,predicateMatched:Boolean(condition),credit:'smoke-only'});
  if(!resource.pullRequestNumber)return {complete:false,semanticAcceptance:'pending',obligations:[{id:'useful_requested_change',status:'unresolved'}]};
  const {pr,files,read}=await candidateReader({api,repository,resource});
  const text=await read(guide),lower=text.toLowerCase();
  const original=originalFiles.find(file=>file.path===guide);assert(original);
  const sourceFacts=literalManifestFacts(originalFiles),checkedClaims=supportedManifestClaims(text,sourceFacts);
  const sourceBindings=originalFiles.filter(file=>file.path!==guide).map(file=>({path:file.path,sourceIdentity:sha(file.content)}));
  const finish=()=>({schema:'formal-ai-work-item-independent-oracle/v1',source:resource.source,candidate:resource.head,guide,
    sourceFacts,sourceBindings,checkedClaims,smokeObservations:rows,
    obligations:[{id:'requested_guide_diff',status:rows.find(row=>row.id==='actual_requested_guide_diff')?.predicateMatched?'passed':'unresolved'},
      {id:'source_consumers_and_role_claims',status:'unresolved'},
      {id:'integration_contradictions_independently_checked',status:'unresolved'}],
    complete:false,semanticAcceptance:'pending',
    reason:'Manifest literal claims can be checked exactly; keyword matches cannot certify requested integration semantics.',
    graderAuthorship:'manually authored trusted checker; zero Formal AI credit',candidateTestsAcceptedAsProof:false});
  check('actual_requested_guide_diff',files.some(file=>file.filename===guide&&file.changes>0)&&sha(text)!==sha(original.content));
  for(const input of originalFiles.filter(file=>file.path!==guide)){
    check('preserved_source:'+input.path,sha(await read(input.path))===sha(input.content));
  }
  if(kind==='different-components'){
    const cargo=originalFiles.find(file=>file.path==='rust/Cargo.toml').content;
    const storage=originalFiles.find(file=>file.path==='rust/src/link_store.rs').content;
    const format=originalFiles.find(file=>file.path==='rust/src/links_format.rs').content;
    assert(cargo.includes('link-cli')&&cargo.includes('lino-objects-codec'));
    assert(storage.includes('link_cli::')&&storage.includes('lino_objects_codec::')&&format.includes('format_indented_ordered'));
    check('held_out_components',['link-cli','lino-objects-codec'].every(name=>text.includes(name)));
    check('held_out_source_bindings',originalFiles.filter(file=>file.path!==guide).every(file=>text.includes(file.path)));
    check('persistence_transaction_lock_boundary',/persist|stor/u.test(lower)&&/transact/u.test(lower)&&/lock/u.test(lower));
    check('notation_parse_format_boundary',/pars/u.test(lower)&&/format/u.test(lower)&&/notation/u.test(lower));
    check('feature_native_constraints',/doublets-native/u.test(lower)&&/optional/u.test(lower)&&/native/u.test(lower));
    check('unsupported_browser_dependency_not_invented',!/browser.{0,100}(?:imports|bundles|depends on).{0,80}(?:link-cli|lino-objects-codec)/su.test(lower));
    const after=await api('repos/'+repository+'/pulls/'+pr.number);assert.equal(after.head.sha,resource.head);
    return finish();
  }
  // Contract facts derive from the pre-fix snapshot, independently of candidate-authored tests.
  const cargo=originalFiles.find(file=>file.path==='rust/Cargo.toml').content;
  const javascript=originalFiles.find(file=>file.path==='js/web-search-component-entry.js').content;
  const native=originalFiles.find(file=>file.path==='rust/src/web_search_core.rs').content;
  const runner=originalFiles.find(file=>file.path==='rust/src/orchestration/runner.rs').content;
  assert(cargo.includes('command-stream')&&cargo.includes('web-capture')&&cargo.includes('web-search'));
  assert(runner.includes('StreamingRunner')&&native.includes('web_capture')&&native.includes('web_search'));
  assert(javascript.includes('getDefaultProviderIds')&&javascript.includes('mergeResults'));
  check('documented_direct_components',['command-stream','web-capture','web-search','@link-assistant/web-search'].every(name=>text.includes(name)));
  check('manifest_and_consumer_provenance',originalFiles.filter(file=>file.path!==guide).every(file=>text.includes(file.path)));
  check('command_execution_boundary',/command.{0,100}execut|execut.{0,100}command/su.test(lower)
    &&/argument|argv/u.test(lower)&&/working directory|working-directory|cwd/u.test(lower));
  check('native_feature_target_boundary',/wasm32|webassembly|wasm/u.test(lower)&&/native/u.test(lower)
    &&/default-features|default features/u.test(lower)&&/search/u.test(lower)&&/merge/u.test(lower));
  check('browser_http_package_boundary',/browser/u.test(lower)&&/http/u.test(lower)&&/capture/u.test(lower)
    &&text.includes('getDefaultProviderIds')&&text.includes('mergeResults'));
  check('acquisition_and_ranking_roles',/acquisi|discover|retriev|search/u.test(lower)&&/rank|merg/u.test(lower));
  check('optional_candidates_remain_separate',/design|ecosystem|compatibility|optional/u.test(lower)
    &&/not (?:a |an |actual |direct |runtime |linked |bundled )*(?:dependency|dependencies|integration|linked)|not linked|unless.{0,80}direct/su.test(lower));
  if(contradiction)check('contradicted_link_store_claim_rejected',
    /(?:search|capture).{0,250}(?:not|rather than).{0,100}(?:link.store|knowledge.store|storage)|(?:not|rather than).{0,100}(?:link.store|knowledge.store).{0,100}(?:search|capture)/su.test(lower));
  const after=await api('repos/'+repository+'/pulls/'+pr.number);assert.equal(after.head.sha,resource.head,'candidate changed during oracle');
  return finish();
}
