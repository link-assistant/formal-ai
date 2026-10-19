// Canonical evidence-window policy over host-observed request identities.
/** @param {boolean[]} users @param {boolean[]} cues @param {string[]} identities @returns {number} */
export function continuationEvidenceWindowStart(users, cues, identities) {
  if (users.length !== cues.length || users.length !== identities.length) {
    return users.length;
  }
  let identity = '';
  let found = false;
  let window = 0;
  for (let index = users.length - 1; index >= 0; index -= 1) {
    if (users[index] && !cues[index]) {
      if (!found) {
        identity = identities[index];
        found = true;
        window = index + 1;
      } else {
        if (identities[index] !== identity) {
          return window;
        }
        window = index + 1;
      }
    }
  }
  return window;
}
