from pathlib import Path
import gzip,hashlib,json,os,shutil,subprocess,sys,traceback
root=Path('/tmp/gh-issue-solver-1794900398048');scratch=Path('/tmp/issue-1580-research/link-record-fields-extraction')
if len(sys.argv)==1:
    scratch.mkdir(exist_ok=False)
    with (scratch/'worker.log').open('wb') as stream:
        child=subprocess.Popen([sys.executable,__file__,'worker'],cwd=root,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
    print(json.dumps({'pid':child.pid,'directory':str(scratch)}));sys.exit(0)
records=[]
def run(name,argv):
    with (scratch/(name+'.stdout')).open('wb') as stdout,(scratch/(name+'.stderr')).open('wb') as stderr:
        result=subprocess.run(argv,cwd=root,env=dict(os.environ,NODE_OPTIONS='--max-old-space-size=256'),stdout=stdout,stderr=stderr)
    records.append({'name':name,'argv':argv,'exitCode':result.returncode});(scratch/'records.json').write_text(json.dumps(records,indent=2)+'\n')
    print(json.dumps(records[-1]),flush=True);assert result.returncode==0,name
try:
    parent=root/'rust/src/link_store.rs';child=root/'rust/src/link_store/record_fields.rs'
    before=parent.read_text();assert not child.exists() and 900<len(before.splitlines())<=1000
    (scratch/'parent-before.rs.gz').write_bytes(gzip.compress(before.encode(),mtime=0))
    marker='fn ensure_event_id(event: &mut MemoryEvent, sequence: usize) {';start=before.index(marker);block=before[start:]
    names=['canonical_memory_event','ensure_event_id','event_source_id','push_doublet','push_optional_field']
    moved=block
    for name in names:
        needle='fn '+name+'(';assert moved.count(needle)==1;moved=moved.replace(needle,'pub(super) '+needle)
    prefix='//! Canonical memory-event fields and their stable doublet projection.\n\nuse std::collections::BTreeMap;\nuse std::fmt::Write as _;\n\nuse super::{DoubletLink, MemoryEvent, stable_id};\n\n'
    declarations='mod record_fields;\nuse record_fields::{\n    canonical_memory_event, ensure_event_id, event_source_id, push_doublet, push_optional_field,\n};\n'
    anchor='mod staging;\n';cfg='#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]\n'
    after=before[:start].rstrip()+'\n'
    assert after.count(anchor)==1 and after.count('use std::fmt::Write as _;\n')==1
    after=after.replace(anchor,anchor+declarations).replace('use std::fmt::Write as _;\n','')
    # All remaining BTreeMap uses belong to the native-only storage implementation.
    after=after.replace('use std::collections::BTreeMap;\n',cfg+'use std::collections::BTreeMap;\n',1)
    inverse=after.replace(declarations,'').replace(cfg+'use std::collections::BTreeMap;\n','use std::collections::BTreeMap;\n',1).replace('use std::fmt;\n','use std::fmt;\nuse std::fmt::Write as _;\n',1).rstrip()+'\n\n'+block
    assert inverse==before
    child.write_text(prefix+moved);parent.write_text(after)
    for name,path in [('parent',parent),('child',child)]:(scratch/(name+'-before-format.rs')).write_bytes(path.read_bytes())
    run('rust-format',['rustfmt','--edition','2024',str(parent),str(child)])
    compare="import assert from 'node:assert/strict';import fs from 'node:fs';import {lex} from './scripts/self-translation/lexer.mjs';for(const [name,path] of [['parent','rust/src/link_store.rs'],['child','rust/src/link_store/record_fields.rs']]){const old=fs.readFileSync(process.argv[1]+'/'+name+'-before-format.rs','utf8');const now=fs.readFileSync(path,'utf8');assert.deepEqual(lex(now,'Rust').map(t=>[t.type,t.text]),lex(old,'Rust').map(t=>[t.type,t.text]),path);}console.log('Every syntax token and comment preserved by formatting');"
    run('format-token-equivalence',['node','--input-type=module','-e',compare,str(scratch)])
    assert len(parent.read_text().splitlines())<=900 and len(child.read_text().splitlines())<=900
    run('projection-currency',['node','scripts/translate-js-rust.mjs','--verify'])
    run('focused-source-gates',['node','experiments/formal_ai_subagent/local-gates.mjs','--only','check-abbreviations,check-file-size-js-twin,check-file-size,check-readable-code,check-generate-module-map-js-twin,check-runtime-feature-inventory'])
    run('rust-format-check',['rustfmt','--edition','2024','--check',str(parent),str(child)])
    run('whitespace',['git','diff','--check'])
    result={'passed':True,'beforeSha256':hashlib.sha256(before.encode()).hexdigest(),'afterParentSha256':hashlib.sha256(parent.read_bytes()).hexdigest(),'childSha256':hashlib.sha256(child.read_bytes()).hexdigest(),'originalLines':len(before.splitlines()),'parentLines':len(parent.read_text().splitlines()),'childLines':len(child.read_text().splitlines()),'exactInverseBeforeFormatting':True,'formatterPreservesEverySyntaxTokenAndComment':True,'publicNamesAndFunctionBodiesUnchanged':True,'remainingParentImportsAudited':True,'localRustExecution':False,'hostedCompilationPending':True,'fullIssueComplete':False,'independentAuthorshipCredit':0}
    (scratch/'outcome.json').write_text(json.dumps(result,indent=2)+'\n')
    archive=root/'experiments/issue-1580/evidence/checks/link-record-fields-extraction';archive.mkdir(exist_ok=False)
    for path in sorted(scratch.iterdir()):
        if not path.is_file() or path.name=='worker.log':continue
        raw=path.read_bytes();(archive/(path.name+'.gz')).write_bytes(gzip.compress(raw,mtime=0))
    shutil.copyfile(__file__,archive/'extract-link-record-fields.py')
    (archive/'outcome.json').write_text(json.dumps(result,indent=2)+'\n')
    (archive/'README.md').write_text('Move the existing canonical memory-event and doublet-field helpers into a private child module. Function bodies and public paths are unchanged, an exact pre-format inverse reconstructs the complete original file, and formatting preserves every syntax token/comment. The remaining parent BTreeMap import is native-only; fmt::Write belongs to the child. The existing source-size gate is the reproducer: the original exceeds 900 lines, while both resulting modules fit. Local verification excludes native compilation/execution; hosted qualification remains pending.\n')
    print(json.dumps(result),flush=True)
except BaseException:
    traceback.print_exc();(scratch/'failure.json').write_text(json.dumps({'passed':False,'traceback':traceback.format_exc()})+'\n');sys.exit(1)
