function requireCondition(value, message = 'UnboundSourceContract') {
  if (!value) throw Error(message);
}
const invariant = Object.freeze({
  ok: requireCondition,
  equal(left, right, message) {
    requireCondition(Object.is(left, right), message);
  },
  notEqual(left, right, message) {
    requireCondition(!Object.is(left, right), message);
  }
});
export default invariant;
