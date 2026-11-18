import { readText, parseLino } from './host.mjs';
export function sourceDocumentationPolicy() {
  try {
    const parsed = parseLino(readText('data/meta/source-documentation.lino'));
    const root = parsed.name === 'source-documentation' ? parsed : parsed.children?.find(node => node.name === 'source-documentation');
    if (!root) return null;
    const values = name => (root.children ?? []).filter(node => node.name === name).map(node => node.value ?? node.id);
    const policy = {
      actions: values('action'),
      preserve: values('preserve'),
      negations: values('negation'),
      prefixes: values('prefix'),
      template: values('template')[0],
      dependencyCues: values('dependency-cue'),
      versionCues: values('version-cue'),
      sectionCues: values('section-cue')
    };
    for (const name of ['manifest-start', 'manifest-end', 'manifest-title', 'manifest-notice', 'manifest-header', 'manifest-separator', 'manifest-row', 'manifest-absent-version', 'manifest-unqualified-target']) {
      const key = name.replace(/-([a-z])/gu, (_, letter) => letter.toUpperCase());
      policy[key] = values(name)[0];
    }
    policy.sourceUsage={heading: values("source-usage-heading")[0],call: values("source-usage-call")[0],reference: values("source-usage-reference")[0],codeOpen: values("source-usage-code-open")[0],codeClose: values("source-usage-code-close")[0]};
    return policy;
  } catch {
    return null;
  }
}
