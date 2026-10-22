// Structural subprocess guard: local process provenance, not whole-program I/O proof.
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
const blocking = new Set(['output', 'status', 'wait', 'wait_with_output']);
const synchronous = new Set(['spawnSync', 'execSync', 'execFileSync']);
export async function subprocessBoundaryAuditor(root) {
  const runtime = await import(pathToFileURL(path.join(root, 'js/vendor/tree-sitter/web-tree-sitter.mjs')));
  await runtime.Parser.init();
  const parser = new runtime.Parser();
  parser.setLanguage(await runtime.Language.load(path.join(root, 'js/vendor/tree-sitter/tree-sitter-rust.wasm')));
  const require = createRequire(path.join(root, 'package.json'));
  const babel = require('@babel/parser');
  function rust(source) {
    const tree = parser.parse(source);
    try {
      if (!tree || tree.rootNode.hasError) throw Error('InvalidRustBoundarySyntax');
      const nodes = [], parents = new Map();
      function walk(node, parent) { nodes.push(node); if (parent) parents.set(node.id, parent); for (const child of node.namedChildren) walk(child, node); }
      walk(tree.rootNode);
      const field = (node, name) => node?.childForFieldName(name);
      const text = node => node ? source.slice(node.startIndex, node.endIndex) : '';
      const commandNames = new Set(), childNames = new Set(), processNamespaces = new Set();
      for (const node of nodes.filter(node => node.type === 'use_declaration')) {
        const declaration = text(node).replace(/\s+/gu, '');
        for (const [name, target] of [['Command',commandNames], ['Child',childNames]]) {
          const direct = new RegExp('^(?:pub)?use(?:std|tokio)::process::'+name+'(?:as([A-Za-z_][A-Za-z_0-9]*))?;$').exec(declaration);
          if (direct) target.add(direct[1] || name);
          if (/^(?:pub)?use(?:std|tokio)::process::\{/u.test(declaration)) {
            const member = new RegExp('(?:\\{|,)'+name+'(?:as([A-Za-z_][A-Za-z_0-9]*))?(?:,|\\})').exec(declaration);
            if (member) target.add(member[1] || name);
          }
        }
        const namespace = /^(?:pub)?use(?:std|tokio)::process(?:as([A-Za-z_][A-Za-z_0-9]*))?;$/u.exec(declaration);
        if (namespace) processNamespaces.add(namespace[1] || 'process');
      }
      function typeKind(value) {
        const name=value.replace(/[&\s]/gu,'').replace(/^mut/u,'');
        if (commandNames.has(name) || /^(?:std|tokio)::process::Command$/u.test(name)) return 'command';
        if (childNames.has(name) || /^(?:std|tokio)::process::Child$/u.test(name)) return 'child';
        for (const namespace of processNamespaces) { if (name===namespace+'::Command') return 'command'; if(name===namespace+'::Child') return 'child'; }
        return null;
      }
      function enclosing(node, type) { for (let at=node;at;at=parents.get(at.id)) if(at.type===type) return at; return null; }
      const bindings = [], fields = new Map();
      for (const node of nodes) {
        if (node.type==='field_declaration') {
          const owner=enclosing(node,'struct_item'), name=text(field(node,'name')), kind=typeKind(text(field(node,'type')));
          if(owner && name && kind) fields.set(text(field(owner,'name'))+':'+name,kind);
        }
        if (node.type==='parameter' || node.type==='let_declaration') {
          const pattern=field(node,'pattern'), scope=enclosing(node,'block') || enclosing(node,'function_item');
          if(pattern && scope && /^(?:mut\s+)?[A-Za-z_][A-Za-z_0-9]*$/u.test(text(pattern))) bindings.push({node,scope,name:text(pattern).replace(/^mut\s+/u,''),kind:typeKind(text(field(node,'type'))),type:text(field(node,'type')).replace(/[&\s]/gu,'').replace(/^mut/u,''),value:field(node,'value')});
        }
      }
      function binding(name, at) {
        return bindings.filter(record=>record.name===name
          && record.scope.startIndex<=at.startIndex
          && record.scope.endIndex>=at.endIndex
          && (record.node.type==='parameter'||record.node.startIndex<at.startIndex))
          .sort((left,right)=>right.scope.startIndex-left.scope.startIndex || right.node.startIndex-left.node.startIndex)[0];
      }
      function provenance(node, depth=0) {
        if(!node || depth>32)return null;
        if(node.type==='identifier')return binding(text(node),node)?.kind || null;
        if(node.type==='reference_expression' || node.type==='parenthesized_expression' || node.type==='try_expression') return provenance(node.namedChildren[0],depth+1);
        if(node.type==='field_expression') {
          const value=field(node,'value'), name=text(field(node,'field'));
          let owner=binding(text(value),node)?.type;
          if(text(value)==='self') owner=text(field(enclosing(node,'impl_item'),'type'));
          return fields.get(owner+':'+name) || null;
        }
        if(node.type==='call_expression') {
          const callee=field(node,'function'), name=text(callee);
          if(name.endsWith('::new') && typeKind(name.slice(0,-5))==='command') return 'command';
          if(callee?.type==='field_expression') {
            const method=text(field(callee,'field')), receiver=provenance(field(callee,'value'),depth+1);
            if(receiver==='command' && method==='spawn')return 'child';
            if(receiver==='child' && ['unwrap','expect'].includes(method))return 'child';
            if(receiver==='command' && !blocking.has(method))return 'command';
          }
        }
        return null;
      }
      for(let pass=0;pass<bindings.length;pass++) { let changed=false; for(const record of bindings)if(!record.kind && record.value){const kind=provenance(record.value);if(kind){record.kind=kind;changed=true;}} if(!changed)break; }
      const violations=[], unresolved=[], allowed=[];
      for(const node of nodes.filter(node=>node.type==='call_expression')) {
        const callee=field(node,'function'); if(callee?.type!=='field_expression')continue;
        const method=text(field(callee,'field'));if(!blocking.has(method))continue;
        const kind=provenance(field(callee,'value'));
        const record={line:node.startPosition.row+1,column:node.startPosition.column+1,operation:method,receiver:kind || 'unresolved'};
        if(!kind){unresolved.push(record);continue;}
        let terminal=false;
        if(method==='status')for(let at=node;at;at=parents.get(at.id))if(at.type==='if_expression') {
          const condition=text(field(at,'condition')).replace(/\s+/gu,'');
          const consequence=field(at,'consequence');
          if(condition==='std::io::stdin().is_terminal()&&std::io::stdout().is_terminal()' && consequence && node.startIndex>=consequence.startIndex && node.endIndex<=consequence.endIndex)terminal=true;
        }
        if(terminal)allowed.push({...record,policy:'foreground-terminal-cancellation'});else violations.push(record);
      }
      // Opaque macro token trees retain lexical call shapes, not macro expansion semantics.
      for (const macro of nodes.filter(node=>node.type==='macro_invocation')) {
        const tokenTree=macro.namedChildren.find(node=>node.type==='token_tree');
        if(!tokenTree)continue;
        const leaves=[];
        function flatten(node) {
          if(/(?:string|char)_literal|comment/u.test(node.type))return;
          if(!node.children.length){leaves.push(node);return;}
          for(const child of node.children)flatten(child);
        }
        flatten(tokenTree);
        const pairs=new Map(), stack=[];
        for(let index=0;index<leaves.length;index++){
          const value=text(leaves[index]);
          if(['(','[','{'].includes(value))stack.push(index);
          else if([')',']','}'].includes(value)){
            const opening=stack.pop();
            if(opening===undefined || text(leaves[opening])!=={')':'(',']':'[','}':'{'}[value])throw Error('MacroTokenDelimiterMismatch');
            pairs.set(opening,index);
          }
        }
        if(stack.length)throw Error('MacroTokenDelimiterMismatch');
        for(let index=2;index+1<leaves.length;index++){
          const method=text(leaves[index]);
          if(!blocking.has(method)||text(leaves[index-1])!=='.'||text(leaves[index+1])!=='('||!pairs.has(index+1))continue;
          const receiver=leaves[index-2];
          let kind=receiver.type==='identifier'?provenance(receiver):null;
          // Existing typed owner/field authority also applies inside opaque token trees.
          if(receiver.type==='identifier' && index>=4 && text(leaves[index-3])==='.' && leaves[index-4].type==='identifier'){
            const value=leaves[index-4], owner=text(value)==='self'?text(field(enclosing(macro,'impl_item'),'type')):binding(text(value),receiver)?.type;
            kind=fields.get(owner+':'+text(receiver))||null;
          }
          const record={line:receiver.startPosition.row+1,column:receiver.startPosition.column+1,operation:method,receiver:kind||'unresolved',coverage:'opaque-macro-token-call-shape'};
          if(kind)violations.push(record);else unresolved.push(record);
        }
      }
      return {violations,unresolved,allowed,coverage:'local-process-provenance-only'};
    } finally {tree?.delete();}
  }
  function javascript(source, filename='source.mjs', {createImportExpressions} = {}) {
    const ast=babel.parse(source,{sourceType:'unambiguous',createImportExpressions,plugins:filename.endsWith('.ts')?['typescript']:[],errorRecovery:false});
    const direct=new Map(), namespaces=new Set(), nodes=[];
    function visit(node){if(!node || typeof node!=='object')return;if(typeof node.type==='string')nodes.push(node);for(const [key,value] of Object.entries(node))if(!['loc','start','end','extra'].includes(key)){if(Array.isArray(value))value.forEach(visit);else if(value && typeof value==='object')visit(value);}}
    visit(ast);
    const childModule=node=>node?.type==='StringLiteral' && ['node:child_process','child_process'].includes(node.value);
    for(const node of nodes) {
      if(node.type==='ImportDeclaration' && childModule(node.source))for(const entry of node.specifiers){if(entry.type==='ImportNamespaceSpecifier'||entry.type==='ImportDefaultSpecifier')namespaces.add(entry.local.name);else if(synchronous.has(entry.imported.name))direct.set(entry.local.name,entry.imported.name);}
      if(node.type==='VariableDeclarator') {
        let value=node.init;if(value?.type==='AwaitExpression')value=value.argument;
        const moduleSource = value?.type === 'ImportExpression' ? value.source
          : value?.type === 'CallExpression' &&
            ((value.callee.type === 'Identifier' && value.callee.name === 'require') ||
             value.callee.type === 'Import') ? value.arguments[0] : null;
        if(childModule(moduleSource)) {
          if(node.id.type==='Identifier')namespaces.add(node.id.name);
          else if(node.id.type==='ObjectPattern')for(const entry of node.id.properties)if(entry.type==='ObjectProperty' && synchronous.has(entry.key.name))direct.set(entry.value.name,entry.key.name);
        }
      }
    }
    const violations=[],allowed=[];
    for(const node of nodes.filter(node=>node.type==='CallExpression')) {
      const callee=node.callee;
      const operation=callee.type==='Identifier'&&direct.has(callee.name)?direct.get(callee.name):callee.type==='MemberExpression'&&callee.object.type==='Identifier'&&namespaces.has(callee.object.name)?(callee.computed?callee.property.value:callee.property.name):null;
      if(!operation || !synchronous.has(operation))continue;
      const options=node.arguments[2];
      const properties=new Map(options?.type==='ObjectExpression'?options.properties.filter(property=>property.type==='ObjectProperty'&&!property.computed).map(property=>[property.key.name||property.key.value,property.value]):[]);
      const finiteIgnore=operation==='spawnSync' && properties.get('stdio')?.value==='ignore' && properties.get('killSignal')?.value==='SIGKILL' && Number.isSafeInteger(properties.get('timeout')?.value) && properties.get('timeout').value>0 && properties.get('timeout').value<=1000 && !options.properties.some(property=>property.type==='SpreadElement');
      const record={line:node.loc.start.line,column:node.loc.start.column+1,operation};
      if(finiteIgnore)allowed.push({...record,policy:'finite-ignore-sigkill-startup'});else violations.push(record);
    }
    return {violations,unresolved:[],allowed,coverage:'direct-child-process-sync-imports-only'};
  }
  return {rust,javascript,close(){parser.delete();}};
}
