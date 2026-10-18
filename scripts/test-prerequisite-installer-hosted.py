"""Hosted-only direct controls for the actual installer, with finite fake transport/process observations."""
import contextlib, hashlib, importlib.util, io, json, os, pathlib, tempfile, types, unittest, zipfile
from unittest.mock import patch
repository=pathlib.Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('actual_installer',repository/'data/meta/prerequisite-installer.py')
installer=importlib.util.module_from_spec(spec);spec.loader.exec_module(installer)

class InstallerControls(unittest.TestCase):
    def setUp(self):
        if os.environ.get('GITHUB_ACTIONS')!='true': raise RuntimeError('hosted-only: do not execute product installer locally')
        self.temporary=tempfile.TemporaryDirectory(prefix='formal-ai-installer-control-');self.saved=os.getcwd();os.chdir(self.temporary.name);self.addCleanup(self.restore)
    def restore(self):
        os.chdir(self.saved);self.temporary.cleanup()
    def fixture(self,entries=None):
        program='heldout-compiler';buffer=io.BytesIO()
        with zipfile.ZipFile(buffer,'w') as bundle:
            for name,mode in entries or [('package/bin/'+program,0o100755)]:
                entry=zipfile.ZipInfo(name);entry.external_attr=mode<<16;bundle.writestr(entry,b'fixture executable version 1.2')
        archive=buffer.getvalue();digest=hashlib.sha256(archive).hexdigest();doc='https://publisher.invalid/setup';api='https://api.github.com/repos/vendor/compiler/releases/tags/v1';download='https://github.com/vendor/compiler/releases/download/v1/compiler.zip'
        asset={'id':4,'name':'compiler.zip','digest':'sha256:'+digest,'size':len(archive),'browser_download_url':download}
        release={'id':3,'draft':False,'prerelease':False,'assets':[asset]}
        responses={doc:b'<a href="https://github.com/vendor/compiler/releases/tag/v1">release</a> compiler.zip',api:json.dumps(release).encode(),download:archive}
        grant={'schema':'formal-ai-workspace-prerequisites-v1','scope':'workspace','programs':[program],'bootstrap':{'program':'python3','arguments':['.formal-ai/bootstrap/prerequisite-installer.py']},'allowNetwork':True,'networkHosts':['publisher.invalid','api.github.com','github.com'],'processes':[],'probes':[['--version']],'maxArchiveBytes':1048576,'maxExpandedBytes':1048576}
        request={'program':program,'publisher':{'program':program,'source_id':'official','host':'publisher.invalid','documentation':doc},'dependencies':[],'grant':grant}
        return request,responses,asset
    def run_fixture(self,request,responses,exit_code=0):
        class Opener:
            def open(self,request,timeout): return io.BytesIO(responses[request.full_url])
        def process(arguments,**options):
            options['stdout'].write(b'fixture version 1.2\n');return types.SimpleNamespace(returncode=exit_code)
        with patch.object(installer.urllib.request,'build_opener',return_value=Opener()),patch.object(installer.subprocess,'run',side_effect=process) as observed,contextlib.redirect_stdout(io.StringIO()):
            installer.execute(request)
            return observed.call_count
    def test_heldout_program_uses_generic_archive_and_exact_workspace(self):
        request,responses,_=self.fixture();self.assertEqual(self.run_fixture(request,responses),1)
        active=pathlib.Path('.formal-ai/toolchains/active-heldout-compiler.bin').read_text().strip();self.assertTrue(pathlib.Path(active).is_relative_to(pathlib.Path.cwd()));self.assertTrue((pathlib.Path(active)/'heldout-compiler').is_file())
    def test_absent_program_grant_runs_no_process(self):
        request,responses,_=self.fixture();request['grant']['programs']=[]
        with self.assertRaisesRegex(ValueError,'program_not_granted'): self.run_fixture(request,responses)
        self.assertFalse(pathlib.Path('.formal-ai').exists())
    def test_denied_network_runs_no_transport(self):
        request,responses,_=self.fixture();request['grant']['allowNetwork']=False
        with self.assertRaisesRegex(ValueError,'network_not_granted'): self.run_fixture(request,responses)
        self.assertFalse(pathlib.Path('.formal-ai').exists())
    def test_ungranted_document_host_is_refused(self):
        request,responses,_=self.fixture();request['grant']['networkHosts'].remove('publisher.invalid')
        with self.assertRaisesRegex(ValueError,'url_not_granted'): self.run_fixture(request,responses)
    def test_digest_mismatch_prevents_extraction_and_probe(self):
        request,responses,_=self.fixture();api=next(url for url in responses if 'api.github.com' in url);release=json.loads(responses[api]);release['assets'][0]['digest']='sha256:'+'0'*64;responses[api]=json.dumps(release).encode()
        with self.assertRaisesRegex(ValueError,'archive_digest_mismatch'): self.run_fixture(request,responses)
        self.assertFalse(pathlib.Path('.formal-ai/toolchains/active-heldout-compiler.bin').exists())
    def test_traversal_cannot_escape_workspace(self):
        request,responses,_=self.fixture([('../escape',0o100755)])
        with self.assertRaisesRegex(ValueError,'unsafe_archive_path'): self.run_fixture(request,responses)
        self.assertFalse(pathlib.Path('../escape').exists())
    def test_symlink_entry_is_not_extracted(self):
        request,responses,_=self.fixture([('package/bin/heldout-compiler',0o120777)])
        with self.assertRaisesRegex(ValueError,'unsupported_archive_entry'): self.run_fixture(request,responses)
    def test_duplicate_executable_candidates_are_refused(self):
        request,responses,_=self.fixture([('one/bin/heldout-compiler',0o100755),('two/bin/heldout-compiler',0o100755)])
        with self.assertRaisesRegex(ValueError,'executable_ambiguous_or_missing'): self.run_fixture(request,responses)
    def test_failed_probe_never_publishes_active_prefix(self):
        request,responses,_=self.fixture()
        with self.assertRaisesRegex(ValueError,'installed_probe_failed'): self.run_fixture(request,responses,exit_code=1)
        self.assertFalse(pathlib.Path('.formal-ai/toolchains/active-heldout-compiler.bin').exists())
    def test_dependency_process_requires_exact_argv(self):
        request,responses,_=self.fixture();request['dependencies']=['ungranted-runtime']
        with self.assertRaisesRegex(ValueError,'dependency_probe_missing'): self.run_fixture(request,responses)
    def test_untrusted_release_repository_link_is_refused(self):
        request,responses,_=self.fixture();doc=request['publisher']['documentation'];responses[doc]=b'<a href="https://evil.invalid/vendor/compiler/releases/tag/v1">release</a> compiler.zip'
        with self.assertRaisesRegex(ValueError,'publisher_release_link_ambiguous_or_missing'): self.run_fixture(request,responses)


    def test_existing_prefix_is_reverified_without_second_archive_download(self):
        request,responses,_=self.fixture();self.run_fixture(request,responses)
        download=next(url for url in responses if '/releases/download/' in url);del responses[download]
        self.assertEqual(self.run_fixture(request,responses),1)
    def test_tampered_retained_archive_cannot_authorize_cached_execution(self):
        request,responses,_=self.fixture();self.run_fixture(request,responses)
        prefix=next(pathlib.Path('.formal-ai/toolchains').glob('heldout-compiler-*'));(prefix/'.formal-ai-source-archive.zip').write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError,'cached_archive_digest_mismatch'): self.run_fixture(request,responses)
    def test_tampered_executable_cannot_hide_behind_a_valid_archive(self):
        request,responses,_=self.fixture();self.run_fixture(request,responses)
        active=pathlib.Path('.formal-ai/toolchains/active-heldout-compiler.bin').read_text().strip();(pathlib.Path(active)/'heldout-compiler').write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError,'cached_file_digest_mismatch'): self.run_fixture(request,responses)
    def test_cached_symlink_is_refused_before_probe(self):
        request,responses,_=self.fixture();self.run_fixture(request,responses)
        active=pathlib.Path('.formal-ai/toolchains/active-heldout-compiler.bin').read_text().strip();executable=pathlib.Path(active)/'heldout-compiler';executable.unlink();executable.symlink_to('/usr/bin/false')
        with self.assertRaisesRegex(ValueError,'cached_symlink'): self.run_fixture(request,responses)

    def test_cached_permission_drift_is_refused(self):
        request,responses,_=self.fixture();self.run_fixture(request,responses)
        active=pathlib.Path('.formal-ai/toolchains/active-heldout-compiler.bin').read_text().strip();(pathlib.Path(active)/'heldout-compiler').chmod(0o777)
        with self.assertRaisesRegex(ValueError,'cached_mode_mismatch'): self.run_fixture(request,responses)
    def test_receipt_hardlink_cannot_overwrite_outside_workspace(self):
        request,responses,_=self.fixture();self.run_fixture(request,responses)
        with tempfile.TemporaryDirectory(prefix='formal-ai-outside-control-') as external:
            outside=pathlib.Path(external)/'untouched';outside.write_text('outside bytes')
            prefix=next(pathlib.Path('.formal-ai/toolchains').glob('heldout-compiler-*'));receipt=prefix/'formal-ai-install-receipt.json';receipt.unlink();os.link(outside,receipt)
            self.run_fixture(request,responses);self.assertEqual(outside.read_text(),'outside bytes')


    def test_current_same_origin_documentation_hop_authenticates_release(self):
        request,responses,_=self.fixture();doc=request['publisher']['documentation'];page='https://publisher.invalid/releases/2.0.html'
        responses[page]=responses[doc];responses[doc]=b'<a href="/releases/2.0.html">Current Version</a>'
        self.assertEqual(self.run_fixture(request,responses),1)
    def test_ambiguous_current_documentation_is_refused(self):
        request,responses,_=self.fixture();responses[request['publisher']['documentation']]=b'<a href="/one">Current Version</a><a href="/two">Latest Release</a>'
        with self.assertRaisesRegex(ValueError,'current_documentation_ambiguous'): self.run_fixture(request,responses)
    def test_cross_origin_current_link_cannot_authorize_a_document(self):
        request,responses,_=self.fixture();responses[request['publisher']['documentation']]=b'<a href="https://foreign.invalid/current">Current Version</a>'
        with self.assertRaisesRegex(ValueError,'publisher_release_link_ambiguous_or_missing'): self.run_fixture(request,responses)
    def test_platform_archive_requires_unique_observed_host_identity(self):
        assets=[{'name':'compiler-x86_64-pc-linux.zip'},{'name':'compiler-aarch64-pc-linux.zip'},{'name':'compiler-x86_64-pc-windows.zip'},{'name':'compiler.zip'}]
        self.assertEqual(installer.select_release_archive(assets,'','Linux','x86_64'),assets[0])
        self.assertEqual(installer.select_release_archive(assets,'compiler.zip','Linux','x86_64'),assets[3])
        with self.assertRaisesRegex(ValueError,'ambiguous_or_missing'): installer.select_release_archive(assets+[dict(assets[0])],'','Linux','x86_64')
        with self.assertRaisesRegex(ValueError,'ambiguous_or_missing'): installer.select_release_archive(assets,'compiler-x86_64-pc-windows.zip','Linux','x86_64')
        with self.assertRaisesRegex(ValueError,'unsupported_host_platform'): installer.select_release_archive(assets,'','Plan9','x86_64')
    def test_generic_unmentioned_archive_cannot_replace_platform_evidence(self):
        with self.assertRaisesRegex(ValueError,'ambiguous_or_missing'): installer.select_release_archive([{'name':'compiler.zip'}],'','Linux','x86_64')


    def test_current_series_uses_highest_strict_version_without_language_rules(self):
        request,responses,_=self.fixture();doc=request['publisher']['documentation']
        responses[doc]=b'<a href="/release/2.9.9.html">Current 2.9.x release</a><a href="/release/2.10.0.html">Current 2.10.x release</a><a href="/release/99.0.0-preview.html">Current preview release</a>'
        responses['https://publisher.invalid/release/2.10.0.html']=b'<a href="https://github.com/vendor/compiler/releases/tag/v1">release</a> compiler.zip'
        self.assertEqual(self.run_fixture(request,responses),1)
    def test_same_highest_version_at_distinct_urls_refuses(self):
        request,responses,_=self.fixture();responses[request['publisher']['documentation']]=b'<a href="/one/2.10.0.html">Current release</a><a href="/two/2.10.0.html">Current release</a>'
        with self.assertRaisesRegex(ValueError,'current_documentation_ambiguous'): self.run_fixture(request,responses)

if __name__=='__main__': unittest.main()
