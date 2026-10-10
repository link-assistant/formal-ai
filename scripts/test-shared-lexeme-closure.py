import hashlib,importlib.util,json,os,re,shutil,tempfile,sys
root=sys.argv[1] if len(sys.argv)>1 else os.path.dirname(os.path.dirname(os.path.realpath(__file__)))
base=root
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
old=load('old',root+'/scripts/audit-total-closure.py');new=old
source=open(root+'/rust/tests/unit/total_closure.rs').read()
base_schema=re.search(r'let base_schema = r"(.*?)";',source,re.S)[1]
fixture=re.search(r'seed_dir.join\("schema-fixture.lino"\),\s*r"(.*?)",',source,re.S)[1]
controls=[]
facet_source=open(root+'/rust/src/seed/meanings/parse.rs').read()
projection=new.facet_label_projection(facet_source)
assert projection['part-of-speech']=='part_of_speech'
assert projection['grammatical-number']=='grammatical_number'
assert projection['self-equation']=='self-equation'
assert len(projection)==7
controls.append('actual seven source facets roundtrip exact original spellings')
assert new.facet_label_projection(facet_source+'\n// "self_equation", "forged"\nconst OTHER_LABEL: &str = "outside_identifier";\n')==projection
controls.append('comments and strings outside canonical initializer preserve projection')
def replace_facet_token(replacement):
 match=re.search(r'const FACET_KINDS: &\[&str\] = &\[(.*?)\];',facet_source,re.S)
 body=match[1].replace('"notation",',replacement,1)
 return facet_source[:match.start(1)]+body+facet_source[match.end(1):]
for invalid in [
 replace_facet_token('"notation", "notation",'),
 replace_facet_token('"self_equation",'),
 replace_facet_token('"mixed_part-name",'),
 replace_facet_token('"QuotedName",'),
 facet_source.replace('const FACET_KINDS','const OTHER_KINDS',1),
 facet_source+'\nconst FACET_KINDS: &[&str] = &["other"];\n',
]:
 try:new.facet_label_projection(invalid);raise AssertionError('invalid projection accepted')
 except RuntimeError:pass
controls.append('duplicate collision mixed unknown missing and ambiguous projection refuse')
with tempfile.TemporaryDirectory(prefix='pr1188-scoped-closure-') as f:
 os.makedirs(f+'/data/meta');os.makedirs(f+'/data/seed')
 def put(path,text):
  os.makedirs(os.path.dirname(f+'/'+path),exist_ok=True)
  with open(f+'/'+path,'w') as out:out.write(text)
 put('data/seed/roles.lino','roles\n');put('data/seed/schema-fixture.lino',fixture)
 for custom in [True,False]:
  put('data/meta/total-closure-schema.lino',base_schema+('  declaration_identity_head custom_record\n  literal_operand_head custom_literal\n' if custom else ''))
  before=old.audit(f);after=new.audit(f)
  assert all(after[key]==value for key,value in before.items())
  if custom:
   semantic=['genuine_intent_reference','genuine_method_reference','genuine_source_reference','genuine_slug_reference','genuine_leaf_reference','genuine_custom_reference']
   excluded=['ignored_comment_token','response_identity','family_identity','evidence_identity','task_identity','intent_identity','custom_identity','literal_cue','literal_word','literal_prefix','literal_substring','literal_custom']
   assert all(value in after['unresolved'] for value in semantic)
   assert all(value not in after['unresolved'] for value in excluded)
   for key,count in [('ignored_full_line_comments',1),('excluded_declaration_identities',6),('excluded_literal_operands',5),('schema_declaration_identity_heads',6),('schema_literal_operand_heads',5)]:assert after[key]==count
  else:assert {'custom_identity','literal_custom'}<=set(after['unresolved'])
  controls.append('original native-authored fixture custom='+str(custom))
 put('data/seed/schema-fixture.lino','')
 put('data/meta/total-closure-schema.lino',base_schema+'  scoped-shared-lexeme-schema data/meta/shared-lexeme-fields-schema.lino\n')
 technical=open(root+'/data/meta/shared-lexeme-fields-schema.lino').read();put('data/meta/shared-lexeme-fields-schema.lino',technical)
 for p in ['js/seed_loader.js','ts/seed_loader.ts','rust/src/seed/parser.rs','rust/src/seed/meanings/parse.rs']:put(p,open(root+'/'+p).read())
 normal='meanings\n  sample\n    shared-lexeme-fields unbound_scope_probe\n      part_of_speech noun\n    lexeme en\n      surface\n        text one\n        use-lexeme-fields unbound_scope_probe\n    lexeme es\n      surface\n        text two\n        use-lexeme-fields unbound_scope_probe\n'
 put('data/seed/shared.lino',normal+'  other\n    semantic unbound_scope_probe\n')
 report=new.audit(f);assert report['resolved_scoped_reference_bindings']==1;assert 'unbound_scope_probe' in report['unresolved'];controls.append('local identity never globally resolves ordinary same-name occurrence')
 put('data/seed/shared.lino',normal.replace('part_of_speech noun','part_of_speech invented_noun_value'))
 report=new.audit(f);assert 'invented_noun_value' in report['unresolved'];controls.append('shared semantic field value remains audited')
 for label,bad in [
  ('missing',normal.replace('shared-lexeme-fields unbound_scope_probe','other-declaration unbound_scope_probe')),
  ('duplicate',normal.replace('shared-lexeme-fields unbound_scope_probe','shared-lexeme-fields unbound_scope_probe\n    shared-lexeme-fields other\n      part_of_speech noun')),
  ('foreign reference',normal.replace('use-lexeme-fields unbound_scope_probe','use-lexeme-fields foreign',1)),
  ('crossroot',normal+'meanings\n  foreign\n    lexeme en\n      surface\n        use-lexeme-fields unbound_scope_probe\n'),
  ('cycle',normal.replace('use-lexeme-fields unbound_scope_probe','use-lexeme-fields unbound_scope_probe\n          use-lexeme-fields unbound_scope_probe',1)),
  ('operator alias',normal.replace('shared-lexeme-fields','shared_lexeme_fields')),
  ('effect',normal.replace('part_of_speech noun','effect mutation')),
  ('reserved owner',normal.replace('  sample\n','  meanings\n')),
 ]:
  put('data/seed/shared.lino',bad);report=new.audit(f)
  assert any(key.startswith('scoped:') or key.startswith('scoped-invalid:') for key in report['unresolved']),label
  controls.append(label+' remains unresolved')
 put('data/seed/shared.lino',normal)
 put('data/meta/shared-lexeme-fields-schema.lino',technical.replace('  field part-of-speech','  field effect'))
 try:new.audit(f);raise AssertionError('unknown schema accepted')
 except RuntimeError:pass
 controls.append('unknown schema refuses')
 put('data/meta/shared-lexeme-fields-schema.lino',technical);put('js/seed_loader.js',open(root+'/js/seed_loader.js').read()+'\n// runtime drift\n')
 try:new.audit(f);raise AssertionError('runtime drift accepted')
 except RuntimeError:pass
 controls.append('runtime source drift refuses')
 put('js/seed_loader.js',open(root+'/js/seed_loader.js').read())
 put('rust/src/seed/meanings/parse.rs',facet_source+'\n// facet authority drift\n')
 try:new.audit(f);raise AssertionError('facet source drift accepted')
 except RuntimeError:pass
 controls.append('facet source authority drift refuses')
print(json.dumps({'controls':controls,'count':len(controls),'originalNativeFixtureSourceSHA256':hashlib.sha256(source.encode()).hexdigest(),'nativeExecution':0}))
print('SCOPED_CLOSURE_CONTROLS_PASS count='+str(len(controls))+' originalNativeFixturesPreserved=true')
