// The worker-thread half of js/server/dreaming-replay.mjs: one booted
// solver realm (js/server/worker-host.mjs `WorkerHost`, the same worker the
// HTTP server answers with) serving
// rust/src/dreaming_application.rs `replay_answer_with_amendments` requests:
// `solve_with_amendment_records(&UniversalSolver::default(), input, &[],
// amendments).answer`. Each reply is posted before the shared flag is raised,
// so the blocked caller can take it with `receiveMessageOnPort`.

import { workerData } from 'node:worker_threads';

import { symbolicFromWorker } from './solve.mjs';
import { solveWithAmendmentRecords } from './standing-requirements.mjs';
import { WorkerHost } from './worker-host.mjs';

const { port, signal } = workerData;
const host = new WorkerHost();

port.on('message', async ({ input, amendments }) => {
  let reply;
  try {
    const solve = async (prompt, history) => symbolicFromWorker(await host.solve(prompt, history), history);
    const answer = await solveWithAmendmentRecords(solve, input, [], amendments);
    reply = { answer: answer.answer };
  } catch (error) {
    reply = { error: String(error?.stack || error) };
  }
  port.postMessage(reply);
  Atomics.store(signal, 0, 1);
  Atomics.notify(signal, 0);
});
