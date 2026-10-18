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

if __name__=='__main__': unittest.main()
