// The route table, read from data/meta/server-routes.lino (the single source
// of truth both servers are held to).

import { childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';

export const ROUTES_FILE = 'data/meta/server-routes.lino';

/**
 * @typedef {{
 *   id: string, method: string, paths: Array<string>, auth: string,
 *   deprecated: boolean, example: string,
 *   matchers: Array<{exact?: string, regex?: RegExp, names?: Array<string>}>
 * }} Route
 */

function escapeRegex(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Compile one manifest path into a matcher.
 * @param {string} pathText
 * @param {Map<string, string>} params
 */
function compilePath(pathText, params) {
  if (!pathText.includes('{')) return { exact: pathText };
  const names = [];
  let source = '^';
  let rest = pathText;
  for (let open = rest.indexOf('{'); open >= 0; open = rest.indexOf('{')) {
    const close = rest.indexOf('}', open);
    const name = rest.slice(open + 1, close);
    source += escapeRegex(rest.slice(0, open));
    source += `(${params.get(name) || '.+'})`;
    names.push(name);
    rest = rest.slice(close + 1);
  }
  source += `${escapeRegex(rest)}$`;
  return { regex: new RegExp(source, 's'), names };
}

/** @param {string} text @returns {Array<Route>} */
export function parseRoutes(text) {
  return childrenNamed(parseLino(text), 'route').map((node) => {
    const params = new Map(
      childrenNamed(node, 'param').map((param) => [param.value, childValue(param, 'pattern')]),
    );
    const paths = childrenNamed(node, 'path').map((pathNode) => pathNode.value);
    return {
      id: node.value,
      method: childValue(node, 'method'),
      paths,
      auth: childValue(node, 'auth') || 'bearer',
      deprecated: childValue(node, 'deprecated') === 'true',
      example: childValue(node, 'example'),
      matchers: paths.map((pathText) => compilePath(pathText, params)),
    };
  });
}

let cached = null;

/** @returns {Array<Route>} */
export function serverRoutes() {
  if (!cached) cached = parseRoutes(readRepoFile(ROUTES_FILE));
  return cached;
}

/**
 * The route serving `method path`, with its parameters, or null.
 * Patterned paths are tried first, as Rust tries its dynamic routes first.
 * @param {string} method
 * @param {string} path
 * @param {Array<Route>} [routes]
 * @returns {{route: Route, params: Record<string, string>} | null}
 */
export function matchRoute(method, path, routes = serverRoutes()) {
  const candidates = routes.filter((route) => route.method === method);
  for (const route of candidates) {
    for (const matcher of route.matchers) {
      if (!matcher.regex) continue;
      const found = matcher.regex.exec(path);
      if (found) {
        const params = {};
        matcher.names.forEach((name, index) => {
          params[name] = found[index + 1];
        });
        return { route, params };
      }
    }
  }
  for (const route of candidates) {
    if (route.matchers.some((matcher) => matcher.exact === '*' || matcher.exact === path)) {
      return { route, params: {} };
    }
  }
  return null;
}

/**
 * One concrete request path per manifest path (patterns use `example`).
 * @param {Route} route
 * @returns {Array<string>}
 */
export function concretePaths(route) {
  const out = [];
  for (const pathText of route.paths) {
    if (pathText === '*' || pathText.includes('{')) {
      if (route.example && !out.includes(route.example)) out.push(route.example);
    } else {
      out.push(pathText);
    }
  }
  return out;
}
