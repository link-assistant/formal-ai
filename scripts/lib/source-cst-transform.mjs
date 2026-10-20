// Declaration-bound transformations over the existing complete readable source pivot.
import {parseLinks,writeLink,textOf} from '../../js/logging/lino-codec.mjs';
import {sourceFromLinks,sourceOwner,verifySourceLinks} from '../../js/meta/readable-source.js';
const quotes={quotedText:'json'};
const quoted=value=>({quoted:value});
const equal=(left,right)=>JSON.stringify(left)===JSON.stringify(right);
const metadata=new Set(['form','role','text','language','path','meaning','semantic-gap']);

// This persisted program owns matching, captures, invariants, refusals and inverse.
// The evaluator below implements these operations; it does not contain task names.
export const PARAMETER_ALPHA_CONTRACT=`(source-transform-contract
 (version "parameter-alpha/v1")
 (match (concept "function") (name "definition.declaration"))
 (capture (binding "unique-simple-parameter") (references "same-flat-scope"))
 (preconditions "complete-preimage" "unique-declaration" "fresh-lexical-target" "no-shadowing" "no-attribute-rewriters" "no-dynamic-lookup" "no-nested-scopes" "known-reference-roles")
 (target (operation "rename-bound-identifier-leaves") (value "definition.target"))
 (preserve "complete-concrete-residual" "types" "effect-and-evaluation-order" "comments-and-layout")
 (refuse "ambiguity" "capture" "shorthand-property" "attribute-rewriter" "macro" "unresolved-meaning" "unsupported-type-reference" "grammar-drift")
 (inverse (operation "swap-binding-and-target") (preimage "complete-result"))
 (scope "capture-free-alpha-equivalence" "source-reflection-not-certified" "no-execution-authority")
 (facets
  (facet (language "JavaScript") (function "function_declaration") (parameters "params") (identifier "identifier") (body "block_statement")
   (forbidden "function_expression" "arrow_function_expression" "object_method" "class_method" "class_declaration" "class_expression" "with_statement")
   (dynamic "eval" "Function" "arguments" "toString"))
  (facet (language "TypeScript") (function "function_declaration") (parameters "params") (identifier "identifier") (body "block_statement")
   (forbidden "function_expression" "arrow_function_expression" "object_method" "class_method" "class_declaration" "class_expression" "with_statement")
   (dynamic "eval" "Function" "arguments" "toString"))
  (facet (language "Rust") (function "function_item") (parameters "parameters") (identifier "identifier") (body "block")
   (forbidden "closure_expression" "macro_invocation" "use_declaration" "struct_expression" "impl_item" "trait_item" "attribute_item" "inner_attribute_item" "lifetime" "label") (dynamic))))`;
const contract=parseLinks(PARAMETER_ALPHA_CONTRACT,quotes)[0];
function field(node,name){const found=node.filter(item=>Array.isArray(item)&&item[0]===name);if(found.length!==1)throw Error('AmbiguousTransformField:'+name);return found[0];}
function optional(node,name){const found=node.filter(item=>Array.isArray(item)&&item[0]===name);if(found.length>1)throw Error('AmbiguousTransformField:'+name);return found[0]??null;}
const value=(node,name)=>textOf(field(node,name)[1]);
const form=node=>optional(node,'form')?value(node,'form'):null;
const role=node=>optional(node,'role')?value(node,'role'):'';
const text=node=>optional(node,'text')?value(node,'text'):null;
const children=node=>node.filter(item=>Array.isArray(item)&&!metadata.has(item[0]));
function records(node,ancestors=[],path=[]){
 const result=[{node,ancestors,path,parent:ancestors.at(-1)??null}];
 node.forEach((item,index)=>{if(Array.isArray(item)&&!metadata.has(item[0]))result.push(...records(item,[...ancestors,node],[...path,index]));});
 return result;
}
function identifier(node){
 if(form(node)!=='identifier')throw Error('UnsupportedParameterPattern');
 if(text(node)!==null)return node;
 const leaves=children(node).filter(child=>form(child)==='identifier'&&text(child)!==null);
 if(leaves.length!==1)throw Error('AmbiguousParameterIdentifier');
 return leaves[0];
}
function declarationName(node){const names=children(node).filter(child=>role(child)==='name');return names.length===1?text(identifier(names[0])):null;}
function language(documentModel){return value(documentModel[0]==='owned-source'?documentModel[2]:documentModel,'language');}

/** Create a complete rule program from observed content and declaration-qualified captures. */
export function createSourceCstTransformRule(document,{declaration,binding,target}){
 sourceFromLinks(document);const owner=sourceOwner(document);
 if(!owner)throw Error('OwnedTransformSourceRequired');
 for(const name of [declaration,binding,target])if(typeof name!=='string'||!/^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(name))throw Error('UnsupportedTransformIdentifier');
 if(binding===target||declaration===binding)throw Error('AmbiguousOrEmptyTransform');
 return ['source-transform',structuredClone(contract),['definition',['owner',quoted(owner)],['declaration',quoted(declaration)],['binding',quoted(binding)],['target',quoted(target)]],['preimage',structuredClone(document)]];
}
/** Parse only supported declared operations; missing or weakened contracts never acquire authority. */
export function readSourceCstTransformRule(notation){
 const network=typeof notation==='string'?parseLinks(notation,quotes):[notation];
 if(network.length!==1||network[0]?.[0]!=='source-transform'||network[0].length!==4)throw Error('SourceTransformRuleRequired');
 const rule=network[0];if(!equal(field(rule,'source-transform-contract'),contract))throw Error('UnsupportedSourceTransformContract');
 const definition=field(rule,'definition'),preimage=field(rule,'preimage');
 if(definition.length!==5||preimage.length!==2)throw Error('MalformedSourceTransformRule');
 const names={declaration:value(definition,'declaration'),binding:value(definition,'binding'),target:value(definition,'target')};
 const canonical=createSourceCstTransformRule(preimage[1],names);
 if(!equal(rule,canonical)||value(definition,'owner')!==sourceOwner(preimage[1]))throw Error('SourceTransformRulePreimageMismatch');
 return {network:rule,definition:names,preimage:preimage[1]};
}
/** Persist full recursive preimages and readable operations, never source blobs or hashes. */
export function writeSourceCstTransformRule(rule){
 const {network}=readSourceCstTransformRule(rule);
 const pretty=(node,indent='')=>node.every(item=>!Array.isArray(item))?indent+writeLink(node,quotes):indent+'('+node[0]+'\n'+node.slice(1).map(item=>pretty(item,indent+'  ')).join('\n')+')';
 return pretty(network)+'\n';
}
function staticName(record){
 const name=role(record.node),parentForm=record.parent?form(record.parent):null;
 if(name==='label')return true;
 if(name.includes('|'))return false;
 if((name==='property'&&['member_expression','optional_member_expression'].includes(parentForm))||(name==='key'&&parentForm==='object_property'))return !children(record.parent).some(node=>text(node)==='[');
 return false;
}
function bindingSelection(document,definition){
 const facet=field(contract,'facets').slice(1).find(item=>value(item,'language')===language(document));
 if(!facet)throw Error('UnsupportedTransformLanguage');
 const functionForm=value(facet,'function');
 const functions=records(document).filter(record=>form(record.node)===functionForm&&declarationName(record.node)===definition.declaration);
 if(functions.length!==1)throw Error('AmbiguousOrMissingTransformDeclaration');
 const selected=functions[0].node;
 const siblings=functions[0].parent?children(functions[0].parent):[];
 let preceding=siblings.indexOf(selected)-1;
 while(preceding>=0&&['whitespace','comment','line_comment','block_comment'].includes(form(siblings[preceding])))preceding--;
 if(preceding>=0&&form(siblings[preceding])?.includes('attribute'))throw Error('UnsupportedTransformAttributeRewriter');
 const all=records(selected),forbidden=field(facet,'forbidden').slice(1).map(textOf),dynamic=field(facet,'dynamic').slice(1).map(textOf);
 if(all.some(record=>record.node!==selected&&(form(record.node)===functionForm||forbidden.includes(form(record.node))||form(record.node)?.startsWith('jsx'))))throw Error('UnsupportedTransformScopeOrEffect');
 if(all.some(record=>optional(record.node,'meaning')||optional(record.node,'semantic-gap')))throw Error('UnresolvedTransformMeaning');
 if(all.some(record=>form(record.node)?.includes('identifier')&&dynamic.includes(text(record.node))))throw Error('DynamicTransformLookup');
 let parameters;
 if(value(facet,'parameters')==='params')parameters=children(selected).filter(node=>role(node)==='params');
 else {
  const groups=children(selected).filter(node=>role(node)===value(facet,'parameters'));
  if(groups.length!==1)throw Error('AmbiguousTransformParameters');
  const members=children(groups[0]).filter(node=>!['(',')',',','whitespace','comment'].includes(form(node)));
  if(members.some(node=>form(node)!=='parameter'))throw Error('UnsupportedParameterPattern');
  parameters=members.map(node=>{const patterns=children(node).filter(child=>role(child)==='pattern');if(patterns.length!==1)throw Error('UnsupportedParameterPattern');return patterns[0];});
 }
 const bindings=parameters.map(identifier),matches=bindings.filter(node=>text(node)===definition.binding);
 if(matches.length!==1)throw Error('AmbiguousOrMissingParameterBinding');
 const body=children(selected).filter(node=>role(node)==='body'&&form(node)===value(facet,'body'));
 if(body.length!==1)throw Error('AmbiguousTransformBody');
 const selectedBinding=matches[0],touched=[];
 for(const record of all){
  const spelling=text(record.node),kind=form(record.node);
  if(!kind?.includes('identifier')||spelling===null||staticName(record))continue;
  if(spelling===definition.target)throw Error('TransformBindingCapture');
  if(spelling!==definition.binding)continue;
  if(record.node===selectedBinding){touched.push(record.node);continue;}
  if(kind!=='identifier'||record.ancestors.some(node=>form(node)?.startsWith('ts')||form(node)==='scoped_identifier'))throw Error('UnsupportedTransformTypeOrReference');
  if(['name','pattern','param'].includes(role(record.node)))throw Error('TransformBindingShadow');
  if(role(record.node).includes('|')||record.ancestors.some(node=>form(node)?.includes('pattern')))throw Error('AmbiguousTransformReference');
  if(!record.ancestors.includes(body[0]))throw Error('UnsupportedTransformReferenceScope');
  touched.push(record.node);
 }
 return touched;
}
function transform(document,rule,frontend){
 const parsed=readSourceCstTransformRule(rule);
 if(!equal(document,parsed.preimage))throw Error('SourceTransformCompletePreimageMismatch');
 verifySourceLinks(document,frontend);
 const changed=structuredClone(document),touched=bindingSelection(changed,parsed.definition);
 for(const node of touched)field(node,'text')[1]=quoted(parsed.definition.target);
 // Fresh grammar must reproduce the exact same full tree with only bound names changed.
 verifySourceLinks(changed,frontend);
 const restored=structuredClone(changed);
 for(const node of bindingSelection(restored,{...parsed.definition,binding:parsed.definition.target,target:parsed.definition.binding}))field(node,'text')[1]=quoted(parsed.definition.binding);
 if(!equal(restored,document))throw Error('TransformTypeOrEffectResidualMismatch');
 return changed;
}
function commentReferenceNeeds(document,binding){
 const concrete=node=>text(node)??children(node).map(concrete).join('');
 const mentions=node=>(concrete(node).match(/[A-Za-z_$][A-Za-z0-9_$]*/gu)??[]).includes(binding);
 return records(document).filter(record=>record.node[0]==='comment'&&mentions(record.node)).map(record=>({owner:sourceOwner(document),cstPath:record.path,text:concrete(record.node),status:'unresolved',need:'parameter-spelling-mentioned;comment-meaning-and-attachment-require-independent-validation'}));
}
/** Apply the declared rule and independently check its exact full-model inverse. */
export function applySourceCstTransform(document,rule,frontend){
 const parsed=readSourceCstTransformRule(rule),changed=transform(document,rule,frontend);
 const commentNeeds=commentReferenceNeeds(document,parsed.definition.binding);
 const inverse=createSourceCstTransformRule(changed,{...parsed.definition,binding:parsed.definition.target,target:parsed.definition.binding});
 if(!equal(transform(changed,inverse,frontend),document))throw Error('SourceTransformInverseMismatch');
 return {document:changed,
  inverse,
  verification:{schema:'source-cst-alpha-transform/v1',
  binding:'declaration-qualified-parameter',
  typesAndEffects:'unchanged-alpha-residual',
  concreteResidual:true,
  independentGrammarReparse:true,
  exactInverse:true,
  sourceReflection:'not-certified',
  compilerValidity:'not-checked',
  commentMeaning:'not-certified',
  documentationSynchronization:commentNeeds.length?'unresolved-comment-references':'not-certified',
  commentNeeds,
  arbitrarySemanticEquivalence:false,
  authorshipCredit:0}};
}
/** Independently reject changed types, effects, comments, order or unrelated tokens. */
export function verifySourceCstTransform(before,after,rule,frontend){
 verifySourceLinks(after,frontend);
 if(!equal(transform(before,rule,frontend),after))throw Error('TransformTypeOrEffectResidualMismatch');
 return true;
}
