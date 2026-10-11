// Append-only memory log helpers: record events through window.FormalAiMemory,
// wait for pending writes and download exported memory files.

const pendingMemoryWrites = new Set();

export const MEMORY_EXPORT_FILENAME = "formal-ai-memory.lino";

export function recordMemoryEvent(payload) {
  if (typeof window === "undefined" || !window.FormalAiMemory) {
    return Promise.resolve(null);
  }
  try {
    const write = window.FormalAiMemory.appendEvent(payload).catch(() => null);
    pendingMemoryWrites.add(write);
    return write.finally(() => {
      pendingMemoryWrites.delete(write);
    });
  } catch (_error) {
    return Promise.resolve(null);
  }
}

export function waitForMemoryWrites() {
  if (pendingMemoryWrites.size === 0) {
    return Promise.resolve();
  }
  return Promise.allSettled(Array.from(pendingMemoryWrites)).then(() => null);
}

export function downloadTextFile(filename, text) {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return;
  }
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}
