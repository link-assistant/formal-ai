import { readFileSync } from 'node:fs';

/** All consumers reject an invalid file before changing any release state. */
export function parseChangesetFile(filePath, packageName = 'MyPackage') {
  const content = readFileSync(filePath, 'utf8').replaceAll('\r\n', '\n');
  const match = /^---\n([^]*?)\n---(?:\n|$)([^]*)$/.exec(content);
  const escaped = packageName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const header = match?.[1].trim();
  const type = header?.match(
    new RegExp(
      `^(?:'${escaped}'|"${escaped}"|${escaped}):[ \\t]*(major|minor|patch)[ \\t]*$`,
    ),
  )?.[1];
  if (!type || !match[2].trim()) {
    throw new Error(
      `Invalid changeset ${filePath}. Expected ---, '${packageName}': major|minor|patch, ---, and a nonempty description.`,
    );
  }
  return { type, description: match[2].trim() };
}
