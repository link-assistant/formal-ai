/** Mirrors `fn instruction_view`: a cycle-free source-position projection. */
export function literalInstructionView(request, payload) {
  if (payload === null) return request;
  const { start, end } = payload;
  if (!Number.isInteger(start) || !Number.isInteger(end) || start < 0 || start > end || end > request.length) return null;
  return request.slice(0, start) + ' '.repeat(end - start) + request.slice(end);
}
