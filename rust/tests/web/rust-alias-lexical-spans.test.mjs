import test from 'node:test';
import assert from 'node:assert/strict';
import {sourceQualifiedAliasReferences} from '../../../experiments/formal_ai_subagent/source-qualified-rust-aliases.mjs';
const move={from:'sample/src/domain/catalog/leaf.rs',to:'sample/src/domain/catalog/description.rs'};
const consumer='use lexical_library::exposed::leaf::item;';
function context(root='pub(crate) mod domain;\npub use domain::catalog as exposed;\n') {
 return new Map([['sample/Cargo.toml','[lib]\nname="lexical_library"\n'],['sample/src/lib.rs',root],['sample/src/domain/mod.rs','pub mod catalog;\n']]);
}
function project(text,sources=context()){return sourceQualifiedAliasReferences(text,'sample/tests/consumer.rs',[move],sources);}
for(const [name,prefix] of [
 ['ordinary line comment containing raw and block markers',String.raw`// data/cache/**/*.lino r#"quoted"# /* text`+'\n'],
 ['ordinary string containing raw and nested-comment markers',String.raw`const NOTE: &str = "r#\"quoted\"# /* outer /* inner */";`+'\n'],
 ['ordinary block comment containing raw marker',String.raw`/* r#"quoted"# */`+'\n'],
 ['Unicode character escape',String.raw`const MARK: char = '\u{201c}';`+'\n'],
 ['astral character literal',"const MARK: char = '😀';\n"],
 ['escaped apostrophe character',String.raw`const MARK: char = '\'';`+'\n'],
 ['escaped backslash character',String.raw`const MARK: char = '\\';`+'\n'],
 ['ASCII hex character escape',String.raw`const MARK: char = '\x41';`+'\n'],
 ['ordinary lifetime preserves alias offsets',"fn keep<'a>(x: &'a str) -> &'a str { x }\n"],
]) test(name+' preserves exact alias span',()=>{
 const source=prefix+consumer;const rows=project(source);assert.equal(rows.length,1);
 assert.equal(source.slice(rows[0].start,rows[0].end),'leaf');
 assert.equal(rows[0].start,source.indexOf('::leaf::')+2);
});
for(const [name,prefix] of [
 ['actual nested block comment','/* outer /* inner */ tail */\n'],
 ['actual raw string',String.raw`const NOTE: &str = r#"raw"#;`+'\n'],
 ['actual raw byte string',String.raw`const NOTE: &[u8] = br#"raw"#;`+'\n'],
 ['actual raw C string',String.raw`const NOTE = cr#"raw"#;`+'\n'],
 ['surrogate Unicode character',String.raw`const MARK: char = '\u{d800}';`+'\n'],
 ['out of range Unicode character',String.raw`const MARK: char = '\u{110000}';`+'\n'],
 ['invalid Unicode character digits',String.raw`const MARK: char = '\u{xyz}';`+'\n'],
 ['unterminated ordinary string','const NOTE: &str = "unterminated\n'],
 ['unterminated block comment','/* unterminated\n'],
]) test(name+' remains unsupported',()=>assert.throws(()=>project(prefix+consumer)));
test('quoted foreign namespace cannot acquire owned alias references',()=>{
 const source=String.raw`const NOTE: &str = "lexical_library::exposed::leaf::item";`+'\nuse foreign_library::exposed::leaf::item;';
 assert.equal(project(source).length,0);
});
test('alias collision remains refused',()=>{
 assert.throws(()=>project(consumer,context('pub(crate) mod domain;\npub use domain::catalog as exposed;\npub mod exposed;\n')));
});
