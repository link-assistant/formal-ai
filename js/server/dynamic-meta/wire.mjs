// Kept separate so the ordinary server can boot without loading the plug-in
// dependencies or SQLite when the operator hasn't configured a meta store.
export const META_PATH = '/v1/dynamic-meta';
export const META_REQUEST_LIMIT = 256 * 1024;

export function recipePrompt(prompt) {
  if (typeof prompt !== 'string' || prompt.length > META_REQUEST_LIMIT) return null;
  const match = /^(?:Run recipe (\S+) on|Use recipe (\S+) to rewrite) ("[^\n]*")\.?$/iu.exec(prompt.trim());
  if (!match) return null;
  try {
    return { action: 'run', recipe: match[1] ?? match[2], input: JSON.parse(match[3]) };
  } catch {
    return null;
  }
}
