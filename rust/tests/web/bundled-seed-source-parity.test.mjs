import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {seedFiles,meaningFiles} from '../../../js/agentic/crate/seed_meanings.mjs';
import {sourceQualifiedSeedOwners} from '../../../scripts/generate-seed-registry.mjs';
import {sourceCstFrontends} from '../../../scripts/lib/source-cst-frontends.mjs';

test('bundled raw seed inputs match the actual native declaration CST and source bytes',async()=>{
  await installNodeHost(new WorkerHost());
  const runtime=await sourceCstFrontends();
  try{
    const filename='rust/src/seed/embedded_registry.rs';
    const source=readFileSync(new URL('../../../'+filename,import.meta.url),'utf8');
    const parsed=runtime.forSource(filename,'Rust').parse(source,'Rust');
    const render=node=>node.tokenIndex===undefined?node.children.map(render).join(''):parsed.tokens[node.tokenIndex].text;
    const collect=(node,predicate)=>[...(predicate(node)?[node]:[]),...node.children.flatMap(child=>collect(child,predicate))];
    const functions=collect(parsed.tree,node=>node.term==='function_item'&&node.children.some(child=>child.field==='name'&&render(child)==='seed_files'));
    assert.equal(functions.length,1,'one actual native seed_files declaration');
    const paths=collect(functions[0],node=>node.term==='string_literal').map(node=>JSON.parse(render(node)));
    assert(paths.length>0);assert(paths.every(path=>path.startsWith('data/seed/')&&path.endsWith('.lino')));
    assert.equal(new Set(paths).size,paths.length);
    const meanings=collect(parsed.tree,node=>node.term==='const_item'&&node.children.some(child=>child.field==='name'&&render(child)==='MEANING_FILES'));
    assert.equal(meanings.length,1);
    const root=new URL('../../../',import.meta.url);
    const owners=sourceQualifiedSeedOwners(root.pathname).declarations;
    const ownerPaths=new Map(owners.map(owner=>[owner.symbol,owner.seed_path]));
    const meaningPaths=collect(meanings[0],node=>node.term==='identifier').map(render).filter(name=>ownerPaths.has(name)).map(name=>ownerPaths.get(name));
    assert(meaningPaths.length>0);
    assert.deepEqual(meaningFiles(),meaningPaths,'canonical native lexicon priority');
    const actual=seedFiles();
    assert.deepEqual(actual.map(row=>row[0]),paths,'complete native source order');
    for(const [index,path] of paths.entries()){
      assert.equal(actual[index].length,2);
      assert.ok(actual[index][1]===readFileSync(new URL('../../../'+path,import.meta.url),'utf8'),path+' raw source bytes differ');
    }
  }finally{runtime.close();}
});
