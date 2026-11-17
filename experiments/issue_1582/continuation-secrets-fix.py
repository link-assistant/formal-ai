from pathlib import Path
path=Path('scripts/check-secrets.sh')
source=path.read_text().replace('# Exit code 0 = no secrets found; non-zero = secretlint reported a finding.', '# Exit code 0 = no secrets found; non-zero = a finding or scanner failure.')
source=source.replace("printf '%s\\0'", "# npm expands each batch into one quoted shell command. Linux limits each\n# argument (including that command) to 128 KiB, below xargs' default batch\n# size after quoting. Leave room for npm's prefix and shell escaping.\nprintf '%s\\0'")
assert 'xargs -0 npx --yes' in source
path.write_text(source.replace('xargs -0 npx --yes', 'xargs -0 -s 16384 npx --yes'))
for name in ['rust/tests/web/bounded-subprocess-boundaries.test.mjs','experiments/issue_1582/continuation-secrets-regression.py']:
 path=Path(name)
 source=path.read_text()
 source=source.replace('value.replaceAll("\'", "\'\\\\\\\\\'\'")', 'value.replaceAll("\'", "\'" + String.fromCharCode(92) + "\'\'")')
 source=source.replace("+ '\\\\\\\\n');", "+ '\\\\n');")
 source=source.replace(".split('\\\\n').map", ".split('\\n').map")
 path.write_text(source)
