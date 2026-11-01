// Pure evaluator for a deliberately finite relation contract. The host supplies
// an actual complete grammar frontend; this module neither loads WASM nor runs
// observed source. Intended owner: scripts/lib/source-cst-boundary-transform.mjs.
import {replaceAllLiteral} from '../../js/agentic/crate/rust_str.mjs';
import {textOf} from '../../js/logging/lino-codec.mjs';
import {sourceFromLinks, sourceOwner, sourceToLinks, ownedSourceLinks, verifySourceLinks} from '../../js/meta/readable-source.js';
import {comparisonBoundaryRoleKernel,comparisonBoundaryWitnessKernel} from '../../js/agentic/portable/comparison_boundary_kernel.mjs';

const metadata = new Set(['form','role','text','language','path','meaning','semantic-gap']);
const quoted = value => ({quoted:value});
const equal = (left,right) => JSON.stringify(left) === JSON.stringify(right);
const children = node => node.filter(item => Array.isArray(item) && !metadata.has(item[0]));
function field(node, key) {
  const found = node.filter(item => Array.isArray(item) && item[0] === key);
  if (found.length !== 1 || found[0].length !== 2) throw Error('BoundaryFieldRequired:' + key);
  return found[0];
}
function optional(node,key) {
  const found = node.filter(item => Array.isArray(item) && item[0] === key);
  if (found.length > 1) throw Error('BoundaryFieldAmbiguous:' + key);
  return found[0] ? textOf(found[0][1]) : null;
}
const form = node => optional(node,'form');
const role = node => optional(node,'role');
const concrete = node => optional(node,'text') ?? children(node).map(concrete).join('');
function records(node,path = []) {
  if (path.length > 512) throw Error('BoundaryDepthLimit');
  return [{node,path}, ...node.flatMap((item,index) => Array.isArray(item) && !metadata.has(item[0]) ? records(item,[...path,index]) : [])];
}
const at = (node,path) => path.reduce((value,index) => value[index],node);
const ordinary = node => !['whitespace','comment','line_comment','block_comment'].includes(form(node));
function semanticChild(node, name) {
  const found = children(node).filter(child => role(child) === name);
  if (found.length !== 1) throw Error('BoundaryRoleRequired:' + name);
  return found[0];
}
function simpleIdentifier(node) {
  if (form(node) !== 'identifier' || optional(node,'text') === null || !/^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(concrete(node))) throw Error('BoundarySimpleIdentifierRequired');
  return concrete(node);
}
function operand(node, parameters) {
  if (form(node) === 'identifier') {
    const name = simpleIdentifier(node);
    if (!parameters.includes(name)) throw Error('BoundaryFreeReference');
    return {kind:'parameter',name};
  }
  if (['numeric_literal','number'].includes(form(node))) {
    const raw = concrete(node);
    if (!/^(?:[0-9][0-9_]*(?:\.[0-9_]*)?|\.[0-9][0-9_]*)(?:[eE][+-]?[0-9][0-9_]*)?$/u.test(raw)) throw Error('BoundaryDecimalLiteralRequired');
    const value = Number(raw.replaceAll('_',''));
    if (!Number.isFinite(value)) throw Error('BoundaryFiniteLiteralRequired');
    return {kind:'number',value};
  }
  throw Error('BoundaryOperandUnsupported');
}
function binary(node, parameters) {
  if (form(node) !== 'binary_expression') throw Error('BoundaryComparisonRequired');
  const left = semanticChild(node,'left'), right = semanticChild(node,'right');
  const operators = children(node).filter(child => (!role(child) || role(child) === 'operator') && ['<','<=','>','>='].includes(optional(child,'text')));
  if (operators.length !== 1) throw Error('BoundaryOperatorAmbiguous');
  const extra = children(node).filter(child => ordinary(child) && child !== left && child !== right && child !== operators[0]);
  if (extra.length) throw Error('BoundaryComparisonResidual');
  return {left:operand(left,parameters),right:operand(right,parameters),operator:concrete(operators[0]),leaf:operators[0]};
}
function normalize(comparison, subject, operator) {
  const left = comparison.left.kind === 'parameter' && comparison.left.name === subject;
  const right = comparison.right.kind === 'parameter' && comparison.right.name === subject;
  if (left === right) throw Error('BoundarySubjectAmbiguous');
  return {operator,
    bound:left ? comparison.right : comparison.left, reversed:!left,leaf:comparison.leaf};
}
function inspect(document, definition) {
  const source = document[0] === 'owned-source' ? document[2] : document;
  if (textOf(field(source,'language')[1]) !== 'JavaScript') throw Error('BoundaryLanguageUnsupported');
  if (!sourceOwner(document)) throw Error('BoundaryOwnedSourceRequired');
  if (!definition || !['upper','lower'].includes(definition.bound) || typeof definition.inclusive !== 'boolean'
      || typeof definition.declaration !== 'string' || !/^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(definition.declaration)) throw Error('BoundaryDefinitionInvalid');
  const functions = records(document).filter(record => form(record.node) === 'function_declaration'
    && children(record.node).some(child => role(child) === 'name' && concrete(child) === definition.declaration));
  if (functions.length !== 1) throw Error('BoundaryDeclarationAmbiguous');
  const selected = functions[0].node;
  if (records(selected).some(record => optional(record.node,'meaning') !== null || optional(record.node,'semantic-gap') !== null)) throw Error('BoundaryUnresolvedMeaning');
  if (children(selected).some(node => ['async','*'].includes(concrete(node)))) throw Error('BoundaryAsyncOrGeneratorUnsupported');
  let parameterNodes = children(selected).filter(child => role(child) === 'params');
  if (!parameterNodes.length) {
    const group = semanticChild(selected,'parameters');
    if (form(group) !== 'formal_parameters') throw Error('BoundaryParametersUnsupported');
    parameterNodes = children(group).filter(child => ordinary(child) && !['(',')',','].includes(concrete(child)));
  }
  const parameters = parameterNodes.map(simpleIdentifier);
  if (parameters.length < 1 || parameters.length > 3 || new Set(parameters).size !== parameters.length) throw Error('BoundaryParametersUnsupported');
  const body = semanticChild(selected,'body');
  if (!['block_statement','statement_block'].includes(form(body))) throw Error('BoundaryBodyUnsupported');
  const statements = children(body).filter(child => ordinary(child) && !['{','}'].includes(concrete(child)));
  if (statements.length !== 1 || form(statements[0]) !== 'return_statement') throw Error('BoundarySingleReturnRequired');
  const argumentsValue = children(statements[0]).filter(child => role(child) === 'argument');
  const expressionNodes = argumentsValue.length ? argumentsValue : children(statements[0]).filter(child => ordinary(child) && !['return',';'].includes(concrete(child)));
  if (expressionNodes.length !== 1) throw Error('BoundaryReturnExpressionAmbiguous');
  const expression = expressionNodes[0];
  if (!['logical_expression','binary_expression'].includes(form(expression))) throw Error('BoundaryConjunctionRequired');
  const left = semanticChild(expression,'left'), right = semanticChild(expression,'right');
  const operators = children(expression).filter(child => optional(child,'text') === '&&');
  if (operators.length !== 1 || children(expression).filter(child => ordinary(child) && child !== left && child !== right && child !== operators[0]).length) throw Error('BoundaryConjunctionRequired');
  const pair = [binary(left,parameters),binary(right,parameters)];
  const name = operand => operand.kind === 'parameter' ? operand.name : '';
  const roles = comparisonBoundaryRoleKernel(name(pair[0].left),name(pair[0].right),
    name(pair[1].left),name(pair[1].right),pair[0].operator,pair[1].operator,definition.bound,definition.inclusive);
  if (roles[0] !== 'selected') throw Error(roles[1]);
  const subject = roles[2], lower = normalize(pair[Number(roles[5])],subject,roles[7]),
    upper = normalize(pair[Number(roles[6])],subject,roles[8]);
  if (lower.bound.kind === 'parameter' && upper.bound.kind === 'parameter' && lower.bound.name === upper.bound.name) throw Error('BoundaryBoundsAliased');
  const selectedComparison = pair[Number(roles[1])], operator = roles[4];
  const record = records(document).find(item => item.node === selectedComparison.leaf);
  const parent = at(document,functions[0].path.slice(0,-1));
  return {parameters,subject,lower,upper,operator,operatorPath:record.path,
    exported:['export_named_declaration','export_statement'].includes(form(parent)) && !children(parent).some(child => concrete(child) === 'default'),
    oldOperator:concrete(selectedComparison.leaf)};
}

/** Missing declaration names are discoverable source facts, never guessed
 * identifiers. Multiple structurally suitable functions remain ambiguous. */
export function selectComparisonBoundaryDefinition(document,{declaration=null,bound,inclusive}) {
  if (declaration !== null) {
    const definition={declaration,bound,inclusive};inspect(document,definition);return definition;
  }
  const candidates=[];
  for (const {node} of records(document).filter(record=>form(record.node)==='function_declaration')) {
    try {
      const definition={declaration:simpleIdentifier(semanticChild(node,'name')),bound,inclusive};
      inspect(document,definition);candidates.push(definition);
    } catch { /* An unsupported function cannot supply this operation's binding. */ }
  }
  if (candidates.length !== 1) throw Error('BoundaryUniqueSuitableDeclarationRequired');
  return candidates[0];
}

/** Typed goal bindings never contain task-local executable code. */
export function deriveComparisonBoundaryCandidate(document, definition, frontend) {
  verifySourceLinks(document,frontend);
  const observed = inspect(document,definition), changed = structuredClone(document);
  const changedLeaf = at(changed,observed.operatorPath), beforeForm = form(changedLeaf);
  field(changedLeaf,'text')[1] = quoted(observed.operator);
  if (beforeForm === observed.oldOperator) field(changedLeaf,'form')[1] = quoted(observed.operator);
  // Babel's relational-token form is shared by <, <=, > and >=. A changed
  // concrete model must be exactly what a fresh grammar obtains from its text.
  verifySourceLinks(changed,frontend);
  const fresh = ownedSourceLinks(sourceOwner(document),sourceToLinks(sourceFromLinks(changed),'JavaScript',frontend));
  if (!equal(fresh,changed)) throw Error('BoundaryIndependentReparseMismatch');
  const restored = structuredClone(changed);
  field(at(restored,observed.operatorPath),'text')[1] = quoted(observed.oldOperator);
  field(at(restored,observed.operatorPath),'form')[1] = quoted(beforeForm);
  if (!equal(restored,document)) throw Error('BoundaryConcreteResidualMismatch');
  const after = inspect(changed,definition);
  if (after.oldOperator !== observed.operator || !equal(after.parameters,observed.parameters)
      || after.subject !== observed.subject) throw Error('BoundaryRoleProofMismatch');
  return {document:changed,changed:observed.oldOperator !== observed.operator,
    rule:['comparison-boundary-transform',['owner',quoted(sourceOwner(document))],
      ['declaration',quoted(definition.declaration)],['bound',quoted(definition.bound)],
      ['inclusive',quoted(String(definition.inclusive))],['preimage',structuredClone(document)],
      ['operator-path',...observed.operatorPath.map(index => quoted(String(index)))],
      ['before',quoted(observed.oldOperator)],['after',quoted(observed.operator)]],
    verification:{schema:'comparison-boundary-proof/v1',completeConcreteResidual:true,
      independentGrammarReparse:true,exactInverse:true,sharedSubject:observed.subject,
      bound:definition.bound,inclusive:definition.inclusive,scope:'finite-number-interval-only',
      arbitrarySemanticEquivalence:false,compilerValidity:'not-checked',authorshipCredit:0}};
}

function finiteSamples(observed,definition) {
  const literal = bound => bound.kind === 'number' ? bound.value : null;
  let low = literal(observed.lower.bound), high = literal(observed.upper.bound);
  if (low === null) low = high === null ? 0 : high - Math.max(1,Math.abs(high) / 2);
  if (high === null) high = low + Math.max(2,Math.abs(low) / 2);
  const width = high - low, middle = low + width / 2;
  if (![low,high,width,middle].every(Number.isFinite) || !(low < middle && middle < high)) throw Error('BoundaryFiniteWitnessUnavailable');
  const assignments = {};
  if (observed.lower.bound.kind === 'parameter') assignments[observed.lower.bound.name] = low;
  if (observed.upper.bound.kind === 'parameter') assignments[observed.upper.bound.name] = high;
  const lowerInclusive = definition.bound === 'lower' ? definition.inclusive : observed.lower.operator === '>=';
  const upperInclusive = definition.bound === 'upper' ? definition.inclusive : observed.upper.operator === '<=';
  return [low-width,low,middle,high,high+width].map(value => {
    const expected=comparisonBoundaryWitnessKernel(value,low,high,lowerInclusive,upperInclusive);
    const values = {...assignments,[observed.subject]:value};
    if (observed.parameters.some(name => !Object.hasOwn(values,name))) throw Error('BoundaryUnusedParameterUnsupported');
    const args = observed.parameters.map(name => values[name]);
    if (!args.every(Number.isFinite)) throw Error('BoundaryFiniteWitnessUnavailable');
    return {args,expected};
  });
}

/** These samples are derived from a typed interval contract before selecting
 * any edit. They are candidate evidence; independent graders remain separate. */
export function deriveComparisonBoundaryRegression(document,definition,{moduleSpecifier,frontend,forms}) {
  verifySourceLinks(document,frontend);
  if (typeof moduleSpecifier !== 'string' || !/^\.\.?\/[A-Za-z0-9_./-]+\.(?:mjs|js)$/u.test(moduleSpecifier)
      || moduleSpecifier.includes('\\')) throw Error('BoundaryRegressionModuleUnsupported');
  const observed = inspect(document,definition), samples = finiteSamples(observed,definition);
  if (!observed.exported) throw Error('BoundaryRegressionNamedExportRequired');
  if (!forms || !Array.isArray(forms.imports) || !forms.imports.every(value => typeof value === 'string')) throw Error('BoundaryRegressionFormsRequired');
  const render = (key,bindings) => {
    if (typeof forms[key] !== 'string') throw Error('BoundaryRegressionFormsRequired');
    return Object.entries(bindings).reduce((text,[name,value]) => replaceAllLiteral(text,'{'+name+'}',String(value)),forms[key]);
  };
  const name = render('title',{bound:definition.bound,polarity:forms.polarities?.[definition.inclusive?'inclusive':'exclusive']});
  const source = [...forms.imports,render('binding',{declaration:definition.declaration,module:JSON.stringify(moduleSpecifier)}),'',
    render('open',{name:JSON.stringify(name)}),...samples.map(sample => render('assertion',{
      arguments:sample.args.map(value => JSON.stringify(value)).join(', '),expected:sample.expected})),forms.close,''].join('\n');
  sourceToLinks(source,'JavaScript',frontend);
  return {source,samples,contract:{schema:'comparison-boundary-regression/v1',declaration:definition.declaration,
    subject:observed.subject,bound:definition.bound,inclusive:definition.inclusive,
    scope:'finite-number-interval-only',independentAcceptance:false,authorshipCredit:0}};
}
