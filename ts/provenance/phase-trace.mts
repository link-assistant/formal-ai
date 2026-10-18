// Opt-in operational phases contain no requests, source, paths, environment or evidence.
import {encodeTyped} from '../logging/typed-codec.mjs';
export function observedProvenancePhase(phase, action) {
 if (process.env.FORMAL_AI_PROVENANCE_TRACE !== '1') return action();
 const start=performance.now();
 const emit=event=>process.stderr.write(encodeTyped({event:'provenance-phase',phase,state:event,elapsedMs:Math.round(performance.now()-start)})+'\n');
 emit('started');
 try { const value=action(); emit('completed'); return value; }
 catch(error) { emit('failed'); throw error; }
}
