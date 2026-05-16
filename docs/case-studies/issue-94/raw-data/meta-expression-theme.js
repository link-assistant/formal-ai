const themeStorageKey = 'meta-expression.theme.v1';
export const THEMES = Object.freeze(['auto', 'light', 'dark']);

export function loadTheme() {
  try {
    const stored = globalThis.localStorage?.getItem(themeStorageKey);
    if (stored && THEMES.includes(stored)) {
      return stored;
    }
  } catch {
    // ignore storage errors
  }
  return 'auto';
}

export function persistTheme(theme) {
  try {
    globalThis.localStorage?.setItem(themeStorageKey, theme);
  } catch {
    // ignore storage errors
  }
}

export function effectiveTheme(theme) {
  if (theme === 'light' || theme === 'dark') {
    return theme;
  }
  const prefersDark = globalThis.matchMedia?.(
    '(prefers-color-scheme: dark)'
  )?.matches;
  return prefersDark ? 'dark' : 'light';
}

export function applyTheme(theme, root = globalThis.document?.documentElement) {
  if (!root) {
    return;
  }
  const resolved = effectiveTheme(theme);
  root.dataset.theme = resolved;
  root.dataset.themePreference = theme;
}

export function nextTheme(current) {
  const index = THEMES.indexOf(current);
  return THEMES[(index + 1) % THEMES.length];
}

export function themeIcon(theme) {
  if (theme === 'light') {
    return '☀';
  }
  if (theme === 'dark') {
    return '☾';
  }
  return '◐';
}

export function watchSystemTheme(callback) {
  const mq = globalThis.matchMedia?.('(prefers-color-scheme: dark)');
  if (!mq) {
    return () => {};
  }
  const listener = () => callback();
  if (typeof mq.addEventListener === 'function') {
    mq.addEventListener('change', listener);
    return () => mq.removeEventListener('change', listener);
  }
  if (typeof mq.addListener === 'function') {
    mq.addListener(listener);
    return () => mq.removeListener(listener);
  }
  return () => {};
}
