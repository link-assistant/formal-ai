/** Reversible authoring only under an actual host-owned single-writer lease. */
export function createCandidateTransaction(io, path, identity) {
  if (!io.ownsLease()) throw new Error('MissingSingleWriterLease');
  if (io.inspect(path).kind !== 'absent') throw new Error('UnownedExistingDestination');
  let phase = 'prepared';
  const unchanged = () => {
    const state = io.inspect(path);
    return state.kind === 'file' && state.identity === identity;
  };
  return Object.freeze({
    authorizeWrite() {
      if (phase !== 'prepared') throw new Error('InvalidCandidatePhase');
      if (!io.ownsLease()) throw new Error('MissingSingleWriterLease');
      if (io.inspect(path).kind !== 'absent') throw new Error('UnownedExistingDestination');
      return {
        state: phase,
        authorized: true
      };
    },
    recordWrite() {
      if (!io.ownsLease()) throw new Error('MissingSingleWriterLease');
      if (phase !== 'prepared') throw new Error('InvalidCandidatePhase');
      if (!unchanged()) throw new Error('UnobservedCandidateWrite');
      phase = 'written';
      return {
        state: phase,
        removed: false
      };
    },
    abort() {
      if (phase !== 'written') throw new Error('InvalidCandidatePhase');
      if (!io.ownsLease()) throw new Error('MissingSingleWriterLease');
      if (!unchanged()) {
        phase = 'refused-drift';
        return {
          state: phase,
          removed: false
        };
      }
      const removed = io.removeUnchangedCandidate(path, identity);
      phase = removed ? 'rolled-back' : 'refused-drift';
      return {
        state: phase,
        removed,
        processDisposition: 'unverified'
      };
    },
    finish(receipt) {
      if (phase !== 'written') throw new Error('InvalidCandidatePhase');
      if (!io.ownsLease()) throw new Error('MissingSingleWriterLease');
      const disposition = io.processDisposition(receipt);
      if (!['succeeded', 'failed', 'incomplete'].includes(disposition)) throw new Error('UnownedProcessDisposition');
      if (!unchanged()) {
        phase = 'refused-drift';
        return {
          state: phase,
          removed: false
        };
      }
      if (disposition === 'succeeded') {
        phase = 'committed';
        return {
          state: phase,
          removed: false
        };
      }
      const removed = io.removeUnchangedCandidate(path, identity);
      phase = removed ? 'rolled-back' : 'refused-drift';
      return {
        state: phase,
        removed
      };
    }
  });
}
