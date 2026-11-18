import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
// Metadata-only Python policy verification runs in hosted CI, never a compiler or network.
// Workstation verification is limited to JavaScript-only test files.
test('actual official generic archive is a candidate only under authenticated catalog discovery', () => {
  const result = spawnSync('python3', ['-c', String.raw`import importlib.util,json,pathlib,copy
spec=importlib.util.spec_from_file_location('installer','data/meta/prerequisite-installer.py')
installer=importlib.util.module_from_spec(spec);spec.loader.exec_module(installer)
capture=json.loads(pathlib.Path('rust/tests/fixtures/prerequisite-discovery/scala-portable-archive.json').read_text())
release=capture['release'];assets=release['assets']
chosen=installer.select_release_archive(assets,'','Linux','x86_64',catalog=True)
assert chosen['name'].endswith('.zip') and chosen['digest'].startswith('sha256:')
assert chosen['browser_download_url'].startswith(release['html_url'].replace('/tag/','/download/')+'/')
def refuse(items,catalog=True,system='Linux',machine='x86_64'):
 try:installer.select_release_archive(items,'',system,machine,catalog=catalog)
 except ValueError:return
 raise AssertionError('ambiguous, untrusted or incompatible archive accepted')
refuse(assets,catalog=False)
refuse(assets+[dict(chosen)])
refuse([dict(chosen,name='compiler-x86_64-pc-windows.zip')])
refuse([dict(chosen,name='compiler-aarch64-pc-linux.zip')])
refuse(assets,system='Plan9')
refuse(assets+[dict(chosen,name='different.zip')])
assert installer.select_release_archive([dict(chosen,name='compiler-x86_64-pc-linux.zip')],'','Linux','x86_64')['name']=='compiler-x86_64-pc-linux.zip'
print('PORTABLE_ARCHIVE_CATALOG_PASSED')`], {encoding:'utf8',timeout:10000,maxBuffer:65536});
  assert.equal(result.status,0,result.stderr);
  assert.match(result.stdout,/PORTABLE_ARCHIVE_CATALOG_PASSED/u);
});
