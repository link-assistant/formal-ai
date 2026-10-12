let worker;
let timer;
const records = [];
const status = document.querySelector("#status");
const log = document.querySelector("#log");
function record(value) { records.push(value); log.textContent = JSON.stringify(records, null, 2); }
function stop() { clearTimeout(timer); if (worker) worker.terminate(); worker = null; }
function deadline(milliseconds) {
  clearTimeout(timer);
  timer = setTimeout(() => { stop(); status.textContent = "Stopped: browser worker deadline exceeded";
    record({ kind: "timed-out", deadlineMs: milliseconds, scope: "browser-worker" }); }, milliseconds);
}
document.querySelector("#load").onclick = () => {
  stop(); records.length = 0; status.textContent = "Loading explicitly selected Python runtime…";
  worker = new Worker("python-worker.js"); deadline(120000);
  worker.onmessage = ({ data }) => { clearTimeout(timer); record(data); status.textContent = data.kind;
    document.querySelector("#run").disabled = data.kind !== "ready" && data.kind !== "observed"; };
  worker.onerror = event => { clearTimeout(timer); record({ kind: "worker-error", error: event.message }); stop(); };
  worker.postMessage({ kind: "load" });
};
document.querySelector("#run").onclick = () => {
  if (!worker) return;
  document.querySelector("#run").disabled = true; deadline(30000);
  worker.postMessage({ kind: "run", code: document.querySelector("#code").value });
};
document.querySelector("#stop").onclick = () => { stop(); status.textContent = "Stopped"; document.querySelector("#run").disabled = true; };
document.querySelector("#download").onclick = () => {
  const url = URL.createObjectURL(new Blob([JSON.stringify({ recordedAt: new Date().toISOString(),
    userAgent: navigator.userAgent, hardwareConcurrency: navigator.hardwareConcurrency,
    records }, null, 2)], { type: "application/json" }));
  const link = document.createElement("a"); link.href = url; link.download = "webvm-measurements.json"; link.click(); URL.revokeObjectURL(url);
};
window.addEventListener("pagehide", stop);
