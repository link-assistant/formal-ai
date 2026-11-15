/** @param {string} operator @returns {string} */
export function comparisonBoundaryReverseKernel(operator) {
  if (operator === '<') return '>';
  if (operator === '<=') return '>=';
  if (operator === '>') return '<';
  if (operator === '>=') return '<=';
  return '';
}

/**
 * Inputs are structurally observed operands: identifiers retain their spelling;
 * a numeric literal is the empty string. Empty inputs never invent a binding.
 * Output selected fields: tag, comparison index, shared subject, old operator,
 * new operator, lower index, upper index, normalized lower/upper operators.
 * @param {string} leftFirst @param {string} rightFirst
 * @param {string} leftSecond @param {string} rightSecond
 * @param {string} operatorFirst @param {string} operatorSecond
 * @param {string} bound @param {boolean} inclusive
 * @returns {string[]}
 */
export function comparisonBoundaryRoleKernel(leftFirst, rightFirst, leftSecond, rightSecond,
  operatorFirst, operatorSecond, bound, inclusive) {
  if (bound !== 'lower' && bound !== 'upper') return ['refused','BoundaryRoleUnsupported'];
  if (comparisonBoundaryReverseKernel(operatorFirst) === '' || comparisonBoundaryReverseKernel(operatorSecond) === '') return ['refused','BoundaryOperatorUnsupported'];
  let subject = '';
  let matches = 0;
  if (leftFirst !== '' && (leftFirst === leftSecond || leftFirst === rightSecond)) {
    subject = leftFirst;
    matches += 1;
  }
  if (rightFirst !== '' && rightFirst !== leftFirst && (rightFirst === leftSecond || rightFirst === rightSecond)) {
    subject = rightFirst;
    matches += 1;
  }
  if (matches !== 1) return ['refused','BoundarySharedSubjectAmbiguous'];
  if (leftFirst === rightFirst || leftSecond === rightSecond) return ['refused','BoundarySubjectAmbiguous'];
  const normalizedFirst = leftFirst === subject ? operatorFirst : comparisonBoundaryReverseKernel(operatorFirst);
  const normalizedSecond = leftSecond === subject ? operatorSecond : comparisonBoundaryReverseKernel(operatorSecond);
  const firstLower = normalizedFirst === '>' || normalizedFirst === '>=';
  const secondLower = normalizedSecond === '>' || normalizedSecond === '>=';
  if (firstLower === secondLower) return ['refused','BoundaryIntervalRequired'];
  const lowerIndex = firstLower ? '0' : '1';
  const upperIndex = firstLower ? '1' : '0';
  const selected = bound === 'lower' ? lowerIndex : upperIndex;
  const chosenLeft = selected === '0' ? leftFirst : leftSecond;
  const oldOperator = selected === '0' ? operatorFirst : operatorSecond;
  const normalizedTarget = bound === 'upper' ? (inclusive ? '<=' : '<') : (inclusive ? '>=' : '>');
  const newOperator = chosenLeft === subject ? normalizedTarget : comparisonBoundaryReverseKernel(normalizedTarget);
  return ['selected',selected,subject,oldOperator,newOperator,lowerIndex,upperIndex,
    firstLower ? normalizedFirst : normalizedSecond,firstLower ? normalizedSecond : normalizedFirst];
}

/** The independently declared finite-number interval contract is evaluated
 * before edit selection; arbitrary JavaScript values are outside this claim.
 * @param {number} value @param {number} lower @param {number} upper
 * @param {boolean} lowerInclusive @param {boolean} upperInclusive
 * @returns {boolean}
 */
export function comparisonBoundaryWitnessKernel(value,lower,upper,lowerInclusive,upperInclusive) {
  return (lowerInclusive ? value >= lower : value > lower)
    && (upperInclusive ? value <= upper : value < upper);
}
