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

/** Recognized protocol fields cannot disappear through lexical filtering.
 * @param {number} recognizedCount @param {number} validCount @returns {boolean} */
export function tapCounterSyntaxValid(recognizedCount, validCount) {
  return safeTapCount(recognizedCount) && safeTapCount(validCount)
    && recognizedCount <= 1 && recognizedCount === validCount;
}

/** @param {number} count @returns {boolean} */
export function sourceCaptureUniqueAccepted(count) {
  return count === 1;
}

/**
 * @param {boolean} identity @param {boolean} fetch @param {boolean} requested
 * @param {number} declarations @returns {boolean}
 */
export function sourceCaptureDeclarationAccepted(identity, fetch, requested, declarations) {
  return identity && fetch && requested && sourceCaptureUniqueAccepted(declarations);
}

/** @param {number} receipts @param {boolean} ordered @param {boolean} named @returns {boolean} */
export function sourceCaptureReceiptIdentityAccepted(receipts, ordered, named) {
  return sourceCaptureUniqueAccepted(receipts) && ordered && named;
}



/** A single canonical conjunction owner; omitted observations use the true identity.
 * @param {boolean} first @param {boolean} second @param {boolean} third @returns {boolean} */
export function sourceCaptureAllObserved(first, second, third) {
  return first && second && third;
}





/** @param {boolean} observed @param {number} length @param {number} maximum @returns {boolean} */
export function sourceCaptureTextAccepted(observed, length, maximum) {
  return observed && safeTapCount(length) && length > 0 && length <= maximum;
}

/** @param {boolean} observed @param {boolean} decimal @param {boolean} leadingZero
 * @param {number} length @returns {boolean} */
export function sourceCaptureTimestampTextAccepted(observed, decimal, leadingZero, length) {
  return sourceCaptureTextAccepted(observed, length, 20) && decimal && !leadingZero;
}



/**
 * @param {boolean} record @param {number} fields @param {boolean} scoreObserved
 * @param {number} score @param {boolean} policyObserved @param {number} policyLength
 * @returns {boolean}
 */
export function sourceCaptureTrustAccepted(record, fields, scoreObserved, score, policyObserved, policyLength) {
  return record && fields === 2 && scoreObserved && score >= -9007199254740991
    && score <= 9007199254740991 && sourceCaptureTextAccepted(policyObserved, policyLength, 256);
}

/**
 * @param {boolean} record @param {number} fields @param {boolean} nameObserved
 * @param {number} nameLength @param {boolean} urlObserved @param {number} urlLength
 * @returns {boolean}
 */
export function sourceCaptureLicenceAccepted(record, fields, nameObserved, nameLength, urlObserved, urlLength) {
  return record && fields === 2 && sourceCaptureTextAccepted(nameObserved, nameLength, 256)
    && sourceCaptureTextAccepted(urlObserved, urlLength, 2048);
}
