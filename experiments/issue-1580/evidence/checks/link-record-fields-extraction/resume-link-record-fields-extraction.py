from pathlib import Path
import gzip,hashlib,json,os,shutil,subprocess,sys,traceback
root=Path('/tmp/gh-issue-solver-1794900398048');scratch=Path('/tmp/issue-1580-research/link-record-fields-extraction')
if len(sys.argv)==1:
    with (scratch/'resumed-worker.log').open('wb') as stream:
        child=subprocess.Popen([sys.executable,__file__,'worker'],cwd=root,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
    print(json.dumps({'pid':child.pid,'directory':str(scratch)}));sys.exit(0)
records=[]
def run(name,argv):
    with (scratch/(name+'.stdout')).open('wb') as stdout,(scratch/(name+'.stderr')).open('wb') as stderr:
        result=subprocess.run(argv,cwd=root,env=dict(os.environ,NODE_OPTIONS='--max-old-space-size=128'),stdout=stdout,stderr=stderr)
    records.append({'name':name,'argv':argv,'exitCode':result.returncode});(scratch/'resumed-records.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(records[-1]),flush=True);assert result.returncode==0,name
try:
    parent=root/'rust/src/link_store.rs';child=root/'rust/src/link_store/record_fields.rs'
    before=gzip.decompress((scratch/'parent-before.rs.gz').read_bytes()).decode();after=parent.read_text();moved=child.read_text()
    prefix='//! Canonical memory-event fields and their stable doublet projection.\n\nuse std::collections::BTreeMap;\nuse std::fmt::Write as _;\n\nuse super::{DoubletLink, MemoryEvent, stable_id};\n\n'
    assert moved.startswith(prefix);block=moved[len(prefix):]
    for name in ['canonical_memory_event','ensure_event_id','event_source_id','push_doublet','push_optional_field']:
        needle='pub(super) fn '+name+'(';assert block.count(needle)==1;block=block.replace(needle,'fn '+name+'(')
    imported='use record_fields::{\n    canonical_memory_event, ensure_event_id, event_source_id, push_doublet, push_optional_field,\n};\n'
    assert after.count(imported)==1 and after.count('mod record_fields;\n')==1
    cfg='#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]\n'
    recovered=after.replace('mod record_fields;\n','').replace(imported,'').replace(cfg+'use std::collections::BTreeMap;\n','use std::collections::BTreeMap;\n',1).replace('use std::fmt;\n','use std::fmt;\nuse std::fmt::Write as _;\n',1).rstrip()+'\n\n'+block
    assert recovered==before,'Final formatted parent and child must exactly reconstruct the whole preimage'
    assert len(after.splitlines())<=900 and len(moved.splitlines())<=900
    (scratch/'final-byte-inverse.txt').write_text('Final formatted sources exactly reconstruct every original byte after undoing only the declared child-module declaration/imports, child visibility and import relocation. rustfmt moved only the new module declaration into its sorted declaration group; the initial stricter whole-file token-order assertion failed on that inserted declaration, and remains retained.\n')
    run('projection-currency',['node','scripts/translate-js-rust.mjs','--verify'])
    run('focused-source-gates',['node','experiments/formal_ai_subagent/local-gates.mjs','--only','check-abbreviations,check-file-size-js-twin,check-file-size,check-readable-code,check-generate-module-map-js-twin,check-runtime-feature-inventory'])
    run('rust-format-check',['rustfmt','--edition','2024','--check',str(parent),str(child)])
    run('whitespace',['git','diff','--check'])
    result={'passed':True,'beforeSha256':hashlib.sha256(before.encode()).hexdigest(),'afterParentSha256':hashlib.sha256(parent.read_bytes()).hexdigest(),'childSha256':hashlib.sha256(child.read_bytes()).hexdigest(),'originalLines':len(before.splitlines()),'parentLines':len(after.splitlines()),'childLines':len(moved.splitlines()),'exactInverseBeforeFormatting':True,'exactInverseAfterFormatting':True,'initialWholeFileTokenOrderCheckPassed':False,'formatterMovedOnlyNewModuleDeclaration':True,'publicNamesAndFunctionBodiesUnchanged':True,'remainingParentImportsAudited':True,'localRustExecution':False,'hostedCompilationPending':True,'fullIssueComplete':False,'independentAuthorshipCredit':0}
    (scratch/'outcome.json').write_text(json.dumps(result,indent=2)+'\n')
    archive=root/'experiments/issue-1580/evidence/checks/link-record-fields-extraction';archive.mkdir(exist_ok=False)
    for path in sorted(scratch.iterdir()):
        if not path.is_file() or path.name.endswith('worker.log'):continue
        raw=path.read_bytes();(archive/(path.name+'.gz')).write_bytes(gzip.compress(raw,mtime=0))
    for name in ['extract-link-record-fields.py','resume-link-record-fields-extraction.py']:shutil.copyfile(Path(__file__).parent/name,archive/name)
    (archive/'outcome.json').write_text(json.dumps(result,indent=2)+'\n')
    (archive/'README.md').write_text('Move existing canonical memory-event and doublet-field helpers into a private child module. Final formatted parent/child sources exactly reconstruct the complete original file after undoing declared module/import/visibility changes. rustfmt moved the newly inserted module declaration into its sorted declaration group; the initial stricter whole-file token-order failure is retained. Function bodies and public paths are unchanged. The remaining parent BTreeMap import is native-only; fmt::Write belongs to the child. The original source is 977 lines; parent 895 and child 92 both fit the unchanged 900-line warning threshold. No local native compilation/execution is performed; hosted qualification remains pending.\n')
    print(json.dumps(result),flush=True)
except BaseException:
    traceback.print_exc();(scratch/'resumed-failure.json').write_text(json.dumps({'passed':False,'traceback':traceback.format_exc()})+'\n');sys.exit(1)
