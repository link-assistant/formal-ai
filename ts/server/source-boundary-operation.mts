// Intended owner: js/server/source-boundary-operation.mjs.
// Host grammar capability accepts caller-observed source, never reads task paths
// or executes the candidate. Native uses the same typed operation wire shape.
import {posix as path} from 'node:path';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {babelSourceFrontend} from '../../scripts/lib/source-cst-babel.mjs';
import {readSyntaxRules,sourceToLinks,sourceFromLinks,ownedSourceLinks} from '../meta/readable-source.js';
import {selectComparisonBoundaryDefinition,deriveComparisonBoundaryCandidate,deriveComparisonBoundaryRegression} from '../../scripts/lib/source-cst-boundary-transform.mjs';

const digest = value => createHash('sha256').update(value).digest('hex');
const relativeOwner = value => typeof value === 'string' && /^[A-Za-z0-9_][A-Za-z0-9_./-]*\.(?:mjs|js)$/u.test(value)
  && value.split('/').every(part => part !== '' && part !== '.' && part !== '..');
let parse=null,rules=null,regressionForms=null;
try {({parse}=await import('@babel/parser'));}catch{/* Missing installed grammar remains an explicit operation gap. */}
try {rules=readSyntaxRules(readFileSync(new URL('../../data/meta/source-model/syntax-rules.lino',import.meta.url),'utf8'));}catch{/* Never substitute a lexical fallback. */}

try {regressionForms=JSON.parse(readFileSync(new URL('../../data/meta/comparison-boundary-regression.json',import.meta.url),'utf8'));}catch{/* Missing installed forms remain an explicit operation gap. */}

/** No transport receipts, solver prose or source comments can install this
 * capability. The host registration and selected operation are caller-owned.
 * @param {{source:string,owner:string,declaration:string,bound:string,inclusive:boolean,mode:string,testOwner?:string}} request */
export function observedComparisonBoundaryOperation(request) {
  if (!request || typeof request.source !== 'string' || Buffer.byteLength(request.source) > 131072
      || !relativeOwner(request.owner) || !['repair','regression'].includes(request.mode)) throw Error('BoundaryObservedRequestRequired');
  if (typeof parse!=='function'||rules===null)throw Error('BoundaryGrammarUnavailable');
  const frontend = {parse:babelSourceFrontend({parse},request.owner),rules};
  const document = ownedSourceLinks(request.owner,sourceToLinks(request.source,'JavaScript',frontend));
  const definition = selectComparisonBoundaryDefinition(document,{declaration:request.declaration??null,bound:request.bound,inclusive:request.inclusive});
  if (request.mode === 'repair') {
    const result = deriveComparisonBoundaryCandidate(document,definition,frontend);
    const source = sourceFromLinks(result.document);
    return {schema:'observed-comparison-boundary-operation/v1',mode:'repair',owner:request.owner,
      candidate:{path:request.owner,source,beforeSha256:digest(request.source),afterSha256:digest(source),changed:result.changed},
      rule:result.rule,sourceModel:result.document,verification:result.verification,
      authored:false,independentAcceptance:false};
  }
  if (!relativeOwner(request.testOwner) || request.owner === request.testOwner) throw Error('BoundaryRegressionTargetRequired');
  let moduleSpecifier = path.relative(path.dirname(request.testOwner),request.owner);
  if (!moduleSpecifier.startsWith('.')) moduleSpecifier = './' + moduleSpecifier;
  const regression = deriveComparisonBoundaryRegression(document,definition,{moduleSpecifier,frontend,forms:regressionForms});
  return {schema:'observed-comparison-boundary-operation/v1',mode:'regression',owner:request.owner,
    candidate:{path:request.testOwner,source:regression.source,sourceOwnerSha256:digest(request.source),afterSha256:digest(regression.source)},
    sourceModel:document,samples:regression.samples,verification:regression.contract,
    authored:false,independentAcceptance:false};
}
