// Canonical typed decisions; adapters supply observations, never success claims.
/** @param {number} value @returns {boolean} */
export function safeTapCount(value) {
  return value >= 0 && value <= 9007199254740991 && Math.floor(value) === value;
}

/**
 * @param {boolean} complete
 * @param {number} tests
 * @param {number} passed
 * @param {number} failed
 * @param {number} skipped
 * @param {number} cancelled
 * @returns {string}
 */
export function tapVerificationDecision(complete, tests, passed, failed, skipped, cancelled) {
  if (!complete || !safeTapCount(tests) || !safeTapCount(passed) || !safeTapCount(failed)
    || !safeTapCount(skipped) || !safeTapCount(cancelled)) {
    return 'VerificationReceiptIncomplete';
  }
  if (tests === 0 || passed === 0 || skipped >= tests) {
    return 'NoTestsExecuted';
  }
  if (failed !== 0 || cancelled !== 0 || passed + skipped !== tests) {
    return 'VerificationReceiptIncomplete';
  }
  return '';
}
