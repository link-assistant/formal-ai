import { rustIdentifierIsValid } from '../code_task/identifier_domain.mjs';
import { childrenNamed } from '../host.mjs';

/** Resolve data-owned complete clauses. No resolved clause grants effect authority. */
export function compileCompoundTask(request, document) {
  if (typeof request !== 'string') return null;
  const roots = document?.name === 'compound-task-grammar' ? [document]
    : childrenNamed(document, 'compound-task-grammar');
  if (roots.length !== 1) return null;
  const forms = childrenNamed(roots[0], 'form').map(form => {
    const kinds = childrenNamed(form, 'kind'), patterns = childrenNamed(form, 'pattern');
    if (kinds.length !== 1 || patterns.length !== 1) return null;
    try { return { kind: kinds[0].id, expression: new RegExp(patterns[0].id, 'du') }; }
    catch { return null; }
  });
  if (forms.length === 0 || forms.some(form => form === null)) return null;
  const clauses = [];
  let cursor = 0;
  while (cursor < request.length) {
    const gap = /^[ \t\r\n]*/u.exec(request.slice(cursor))[0];
    cursor += gap.length;
    if (cursor === request.length) break;
    const matches = forms.flatMap(form => {
      const match = form.expression.exec(request.slice(cursor));
      return match && match.index === 0 && match[0].length > 0 ? [{ form, match }] : [];
    });
    if (matches.length !== 1) return null;
    const {form, match} = matches[0];
    const captures = Object.fromEntries(Object.entries(match.groups ?? {}).map(([key, text]) =>
      [key, {text, span: match.indices.groups[key]?.map(index => cursor + index) ?? null}]));
    clauses.push({kind:form.kind, span:[cursor,cursor+match[0].length], captures, status:'unresolved'});
    cursor += match[0].length;
  }
  const one = kind => clauses.filter(clause => clause.kind === kind);
  const kinds = forms.map(form => form.kind);
  if (new Set(kinds).size !== kinds.length || kinds.some(kind => one(kind).length !== 1)) return null;
  const rename = one('private-constant-rename')[0]?.captures;
  const tracked = one('tracked-owner-condition')[0]?.captures;
  const receipt = one('exact-field-receipt')[0]?.captures;
  if (!rename || !tracked || !receipt || rename.sourcePath.text !== tracked.trackedPath.text
    || !rustIdentifierIsValid(rename.previous.text) || !rustIdentifierIsValid(rename.replacement.text)
    || rename.previous.text === rename.replacement.text
    || receipt.requiredIdentifier.text !== rename.replacement.text) return null;
  const pathCaptures = clauses.flatMap(clause => Object.entries(clause.captures))
    .filter(([key]) => key.endsWith('Path') && key !== 'nodePath').map(([,value]) => value.text);
  const safe = value => !value.startsWith('/') && !value.includes('\\')
    && value.split('/').every(part => part !== '' && part !== '.' && part !== '..');
  if (pathCaptures.some(value => !safe(value))) return null;
  const destinations = [rename.sourcePath.text,receipt.receiptPath.text,one('proof-target')[0].captures.proofPath.text];
  if (new Set(destinations).size !== destinations.length) return null;
  return Object.freeze({originalRequest:request,unit:'utf16',wholeRequestConsumed:true,
    clauses: Object.freeze(clauses.map(clause => Object.freeze({ ...clause,
      span: Object.freeze(clause.span), captures: Object.freeze(Object.fromEntries(
        Object.entries(clause.captures).map(([key, capture]) => [key, Object.freeze({ ...capture,
          span: capture.span === null ? null : Object.freeze(capture.span) })]))) }))),
    destinations: Object.freeze(destinations), writeAuthority:false,
    unresolvedNeeds: Object.freeze(clauses.map(clause => clause.kind)),
    compilation:'pending', effects:'not-executed'});
}
