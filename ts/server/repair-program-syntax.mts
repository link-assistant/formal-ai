// Host syntax capability only: parsing retrieved source never evaluates it.
let parse = null;
try {
  ({ parse } = await import('@babel/parser'));
} catch {
  // Missing or unsupported host parser leaves a named validation gap.
}

export function validateRepairProgram(language, source) {
  const slug = String(language ?? '').trim().toLowerCase();
  if (!['javascript', 'js', 'typescript', 'ts'].includes(slug)) return null;
  if (typeof source !== 'string') return false;
  if (typeof parse !== 'function') return null;
  const typed = slug === 'typescript' || slug === 'ts';
  try {
    parse(source, { sourceType: 'unambiguous', errorRecovery: false, plugins: typed ? ['typescript'] : [] });
    return true;
  } catch {
    return false;
  }
}
