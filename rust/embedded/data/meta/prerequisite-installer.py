#!/usr/bin/env python3
"""Client-owned bounded workspace installer. No publisher shell text is executed."""
import hashlib, html, json, os, pathlib, platform, re, shutil, ssl, stat, subprocess, sys, tempfile, urllib.error, urllib.parse, urllib.request, zipfile
from html.parser import HTMLParser

def require(condition, reason):
    if not condition:
        raise ValueError(reason)

def record(phase, **values):
    print(json.dumps(dict(schema='formal-ai-prerequisite-observation-v1', phase=phase, **values), sort_keys=True), flush=True)

class Links(HTMLParser):
    def __init__(self):
        super().__init__(); self.links=[]; self.labels=[]; self.anchor=None
    def handle_starttag(self, tag, attrs):
        if tag == 'a':
            self.links.extend(value for key,value in attrs if key == 'href' and value)
            self.anchor=[dict(attrs).get('href'),[]]
    def handle_data(self,data):
        if self.anchor is not None: self.anchor[1].append(data)
    def handle_endtag(self,tag):
        if tag=='a' and self.anchor is not None:
            if self.anchor[0]: self.labels.append((self.anchor[0],''.join(self.anchor[1])))
            self.anchor=None


def documentation_alias(url, publisher_host, hosts):
    """An alias never grants an additional network host."""
    value=urllib.parse.urlsplit(url)
    host=value.hostname or ''
    bare=host.removeprefix('www.')
    if not publisher_host or bare!=publisher_host.removeprefix('www.'):
        return None
    alternate=bare if host.startswith('www.') else 'www.'+bare
    if value.scheme!='https' or value.username or value.password or value.port not in (None,443) or host not in hosts or alternate not in hosts:
        return None
    authority=alternate+(':443' if value.port==443 else '')
    return urllib.parse.urlunsplit(value._replace(netloc=authority,fragment=''))

def transient_document_failure(error):
    if isinstance(error,urllib.error.HTTPError):
        return 500<=error.code<=599
    reason=error.reason if isinstance(error,urllib.error.URLError) else error
    return not isinstance(reason,ssl.SSLError) and isinstance(reason,(TimeoutError,ConnectionError))

def diagnostic_url(url):
    value=urllib.parse.urlsplit(url)
    return urllib.parse.urlunsplit((value.scheme,value.hostname or '',value.path,'',''))

def publisher_document(documentation, fetch, publisher_host=None, hosts=()):
    """At most one authenticated same-origin current-release documentation hop."""
    attempted=set()
    def read_document(url):
        require(url not in attempted,'documentation_already_attempted')
        attempted.add(url)
        try:
            raw,digest=fetch(url,2097152)
            return url,raw,digest
        except Exception as error:
            record('documentation_fetch_failed',url=diagnostic_url(url),status=error.code if isinstance(error,urllib.error.HTTPError) else None,error_type=type(error).__name__)
            alternate=documentation_alias(url,publisher_host,hosts)
            if not transient_document_failure(error) or alternate is None or alternate in attempted:
                raise
            attempted.add(alternate)
            try:
                raw,digest=fetch(alternate,2097152)
            except Exception as alternate_error:
                record('documentation_fetch_failed',url=diagnostic_url(alternate),status=alternate_error.code if isinstance(alternate_error,urllib.error.HTTPError) else None,error_type=type(alternate_error).__name__)
                raise
            record('documentation_alias_selected',url=diagnostic_url(alternate),sha256=digest)
            return alternate,raw,digest
    documentation,raw,document_digest=read_document(documentation)
    document=raw.decode('utf-8'); parser=Links(); parser.feed(document)
    origin=urllib.parse.urlsplit(documentation)
    candidates=[]
    for link,label in parser.labels:
        value=urllib.parse.urlsplit(urllib.parse.urljoin(documentation,link))
        if re.fullmatch(r'(?:current version|latest release|current release)', ' '.join(label.lower().split())):
            if (value.scheme,value.hostname,value.port)==(origin.scheme,origin.hostname,origin.port) and not value.username and not value.password and not value.query and value.path!=origin.path:
                url=urllib.parse.urlunsplit(value._replace(fragment=''))
                if url not in candidates: candidates.append(url)
    require(len(candidates)<=1,'current_documentation_ambiguous')
    if not candidates:
        current=[]
        for link,label in parser.labels:
            if not re.match(r'current\b.*\brelease\b',' '.join(label.lower().split())): continue
            value=urllib.parse.urlsplit(urllib.parse.urljoin(documentation,link))
            if (value.scheme,value.hostname,value.port)!=(origin.scheme,origin.hostname,origin.port) or value.username or value.password or value.query: continue
            version=re.search(r'(?:^|/)(?:v)?(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:\.html?)?/?$',value.path)
            if version:
                current.append((tuple(int(part) for part in version.groups()),urllib.parse.urlunsplit(value._replace(fragment=''))))
        if current:
            highest=max(version for version,_ in current)
            candidates=sorted({url for version,url in current if version==highest})
            require(len(candidates)==1,'current_documentation_ambiguous')
    if candidates:
        documentation=candidates[0]
        documentation,raw,document_digest=read_document(documentation)
        document=raw.decode('utf-8'); parser=Links(); parser.feed(document)
    return documentation,document,parser,document_digest

def github_release_document(documentation, fetch, publisher_host):
    """A catalog-owned canonical release API is also a complete discovery document."""
    value=urllib.parse.urlsplit(documentation)
    if value.hostname!='api.github.com': return None
    match=re.fullmatch(r'/repos/([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)/releases/(latest|tags/[^/]+)',value.path)
    require(value.scheme=='https' and value.port in (None,443) and not value.username and not value.password and not value.query and not value.fragment and publisher_host=='api.github.com' and match is not None,'invalid_release_documentation')
    repo,endpoint=match.groups()
    raw,document_digest=fetch(documentation,2097152)
    release=json.loads(raw)
    require(isinstance(release,dict) and release.get('draft') is False and release.get('prerelease') is False,'unstable_release')
    tag=release.get('tag_name')
    require(isinstance(tag,str) and bool(tag),'release_document_tag_missing')
    require(release.get('html_url')=='https://github.com/'+repo+'/releases/tag/'+urllib.parse.quote(tag,safe=''),'release_document_repository_mismatch')
    if endpoint.startswith('tags/'):
        require(urllib.parse.unquote(endpoint[5:])==tag,'release_document_tag_mismatch')
    return repo,endpoint,release,document_digest

def select_release_archive(assets,document,system=None,machine=None):
    """Use observed host identity; metadata names never grant processes or network."""
    system=(system or platform.system()).lower()
    machine=(machine or platform.machine()).lower()
    systems={'linux':('linux',),'darwin':('darwin','macos','osx','apple'),'windows':('windows','win32','win64')}
    architectures={'x86_64':('x86_64','amd64','x64'),'amd64':('x86_64','amd64','x64'),'aarch64':('aarch64','arm64'),'arm64':('aarch64','arm64')}
    require(system in systems and machine in architectures,'unsupported_host_platform')
    def has(name,token):
        return re.search(r'(?:^|[-_.])'+re.escape(token)+r'(?:[-_.]|$)',name) is not None
    def compatible(asset,explicit):
        name=asset['name'].lower()
        observed_systems={key for key,tokens in systems.items() if any(has(name,token) for token in tokens)}
        observed_arches={tokens for tokens in architectures.values() if any(has(name,token) for token in tokens)}
        if observed_systems and observed_systems!={system}: return False
        if observed_arches and observed_arches!={architectures[machine]}: return False
        return explicit or (observed_systems=={system} and observed_arches=={architectures[machine]})
    zipped=[asset for asset in assets if isinstance(asset.get('name'),str) and asset['name'].lower().endswith('.zip')]
    explicit=[asset for asset in zipped if asset['name'] in html.unescape(document)]
    if explicit:
        candidates=[asset for asset in explicit if compatible(asset,True)]
    else:
        candidates=[asset for asset in zipped if compatible(asset,False)]
    require(len(candidates)==1,'documented_archive_ambiguous_or_missing')
    return candidates[0]

def discovery_request(url, environment=None):
    """Read credentials only for HTTPS GitHub release metadata; never forward them."""
    value=urllib.parse.urlsplit(url)
    request=urllib.request.Request(url,headers={'User-Agent':'formal-ai-prerequisite-discovery','Accept':'application/vnd.github+json' if value.hostname=='api.github.com' else '*/*'})
    metadata=(value.scheme=='https' and value.hostname=='api.github.com' and value.port in (None,443)
              and not value.username and not value.password and not value.query and not value.fragment
              and re.fullmatch(r'/repos/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+/releases/(?:latest|tags/[^/]+)',value.path) is not None)
    if metadata:
        environment=os.environ if environment is None else environment
        token=environment.get('GH_TOKEN') or environment.get('GITHUB_TOKEN')
        if token:
            require(isinstance(token,str) and '\r' not in token and '\n' not in token,'invalid_github_read_token')
            request.add_unredirected_header('Authorization','Bearer '+token)
    return request


def execute(request):
    grant=request['grant']; program=request['program']; publisher=request['publisher']
    require(grant.get('schema') == 'formal-ai-workspace-prerequisites-v1' and grant.get('scope') == 'workspace', 'grant_required')
    require(isinstance(grant.get('programs'),list) and program in grant['programs'], 'program_not_granted')
    require(re.fullmatch(r'[A-Za-z0-9_+.-]+',program) is not None, 'invalid_program')
    require(grant.get('bootstrap') == dict(program='python3',arguments=['.formal-ai/bootstrap/prerequisite-installer.py']), 'bootstrap_not_granted')
    require(grant.get('allowNetwork') is True, 'network_not_granted')
    hosts=grant.get('networkHosts',[])
    require(isinstance(hosts,list) and all(isinstance(host,str) for host in hosts), 'invalid_network_grant')
    maximum=grant.get('maxArchiveBytes',134217728); expanded=grant.get('maxExpandedBytes',536870912)
    require(isinstance(maximum,int) and 0 < maximum <= 134217728 and isinstance(expanded,int) and 0 < expanded <= 536870912,'invalid_size_grant')
    workspace=pathlib.Path.cwd().resolve()
    def safe_path(relative):
        require(not relative.is_absolute() and all(part not in ('..','.') for part in relative.parts), 'outside_workspace')
        current=workspace
        for part in relative.parts:
            current=current/part
            require(not current.is_symlink(), 'symlink_workspace_component')
        return current
    root=safe_path(pathlib.Path('.formal-ai/toolchains')); root.mkdir(parents=True,exist_ok=True)
    def url_allowed(url):
        value=urllib.parse.urlsplit(url)
        return value.scheme == 'https' and value.hostname in hosts and not value.username and not value.password and value.port in (None,443)
    failed_fetch_urls=set()
    class Redirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self,req,fp,code,msg,headers,newurl):
            require(url_allowed(newurl),'redirect_not_granted')
            require(newurl not in failed_fetch_urls,'redirect_to_failed_fetch')
            return super().redirect_request(req,fp,code,msg,headers,newurl)
    opener=urllib.request.build_opener(Redirect())
    def fetch(url,limit,destination=None):
        require(url_allowed(url),'url_not_granted')
        digest=hashlib.sha256(); total=0; chunks=[]
        try:
            response=opener.open(discovery_request(url),timeout=30)
        except Exception as error:
            failed_fetch_urls.add(url)
            if isinstance(error,urllib.error.HTTPError): failed_fetch_urls.add(error.url)
            raise
        with response:
            while True:
                chunk=response.read(65536)
                if not chunk: break
                total+=len(chunk); require(total<=limit,'download_limit_exceeded'); digest.update(chunk)
                if destination: destination.write(chunk)
                else: chunks.append(chunk)
        record('fetch',url=url,sha256=digest.hexdigest(),size=total)
        return b''.join(chunks),digest.hexdigest()
    documentation=publisher['documentation']; require(publisher.get('program') == program,'publisher_program_mismatch')
    release_document=github_release_document(documentation,fetch,publisher.get('host'))
    if release_document is not None:
        repo,endpoint,release,document_digest=release_document
        document=''
    else:
        documentation,document,parser,document_digest=publisher_document(documentation,fetch,publisher.get('host'),hosts)
        releases=[]
        for link in parser.links:
            value=urllib.parse.urlsplit(urllib.parse.urljoin(documentation,link)); parts=value.path.strip('/').split('/')
            if value.scheme == 'https' and value.hostname == 'github.com' and len(parts)>=3 and parts[2]=='releases':
                repo='/'.join(parts[:2]); require(re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repo) is not None,'invalid_release_repository')
                if len(parts)>=5 and parts[3]=='tag': endpoint='tags/'+urllib.parse.quote(urllib.parse.unquote('/'.join(parts[4:])),safe='')
                else: endpoint='latest'
                candidate=(repo,endpoint)
                if candidate not in releases: releases.append(candidate)
        require(len(releases)==1,'publisher_release_link_ambiguous_or_missing')
        repo,endpoint=releases[0]; api='https://api.github.com/repos/'+repo+'/releases/'+endpoint
        release_raw,_=fetch(api,2097152); release=json.loads(release_raw); require(release.get('draft') is False and release.get('prerelease') is False,'unstable_release')
    asset=select_release_archive(release.get('assets',[]),document)
    digest=asset.get('digest',''); require(re.fullmatch(r'sha256:[a-f0-9]{64}',digest) is not None,'asset_digest_missing')
    download=asset['browser_download_url']; require(download.startswith('https://github.com/'+repo+'/releases/download/'),'asset_repository_mismatch')
    tag=release.get('tag_name') or (urllib.parse.unquote(endpoint[5:]) if endpoint.startswith('tags/') else None)
    require(isinstance(tag,str) and tag and download.startswith('https://github.com/'+repo+'/releases/download/'+urllib.parse.quote(tag,safe='')+'/'),'asset_release_tag_mismatch')
    require(isinstance(asset.get('size'),int) and 0<asset['size']<=maximum,'archive_too_large')
    require(shutil.disk_usage(root).free>maximum+expanded+16777216,'insufficient_disk')
    def probe(executable,arguments,dependency=False):
        name=pathlib.Path(executable).name
        if dependency:
            require(any(row.get('program')==name and row.get('arguments')==arguments for row in grant.get('processes',[])), 'dependency_process_not_granted')
        else:
            require(arguments in grant.get('probes',[]),'probe_arguments_not_granted')
        with tempfile.TemporaryFile() as output:
            completed=subprocess.run([str(executable),*arguments],stdout=output,stderr=subprocess.STDOUT,timeout=30,check=False)
            size=output.tell(); require(size<=1048576,'probe_output_limit'); output.seek(0); text=output.read(4096).decode('utf-8','replace')
        record('probe',program=name,executable=str(pathlib.Path(executable).resolve()),executable_sha256=hashlib.sha256(pathlib.Path(executable).read_bytes()).hexdigest(),arguments=arguments,exit_code=completed.returncode,output=text)
        return completed.returncode==0
    for dependency in request.get('dependencies',[]):
        matches=[row for row in grant.get('processes',[]) if row.get('program')==dependency]
        require(len(matches)==1 and shutil.which(dependency) is not None,'dependency_probe_missing')
        require(probe(shutil.which(dependency),matches[0]['arguments'],True),'dependency_unusable')
    prefix=safe_path(pathlib.Path('.formal-ai/toolchains')/(program+'-'+digest[7:]))
    cached=prefix.exists(); retained=prefix/'.formal-ai-source-archive.zip'
    def file_digest(file):
        digest=hashlib.sha256()
        with file.open('rb') as input_file:
            while True:
                chunk=input_file.read(65536)
                if not chunk: break
                digest.update(chunk)
        return digest.hexdigest()
    with tempfile.TemporaryDirectory(prefix='download-',dir=root) as temporary:
        archive=retained if cached else pathlib.Path(temporary)/'archive.zip'
        if cached:
            require(prefix.is_dir() and not retained.is_symlink() and retained.is_file(),'cached_archive_missing')
            require(retained.stat().st_size<=maximum and file_digest(retained)==digest[7:],'cached_archive_digest_mismatch')
            observed=digest[7:]
        else:
            with archive.open('xb') as output: _,observed=fetch(download,maximum,output)
            require(observed==digest[7:],'archive_digest_mismatch')
        with zipfile.ZipFile(archive) as bundle:
            entries=bundle.infolist(); require(0<len(entries)<=10000 and sum(entry.file_size for entry in entries)<=expanded,'archive_inventory_limit')
            names=set()
            for entry in entries:
                relative=pathlib.PurePosixPath(entry.filename); mode=entry.external_attr>>16
                require(not relative.is_absolute() and all(part not in ('..','.') for part in relative.parts) and '\\' not in entry.filename and str(relative) not in names and all(part not in ('','.','..') for part in entry.filename.rstrip('/').split('/')),'unsafe_archive_path')
                require(str(relative) not in ('.formal-ai-source-archive.zip','formal-ai-install-receipt.json'),'reserved_archive_entry')
                require(not stat.S_ISLNK(mode) and (stat.S_IFMT(mode) in (0,stat.S_IFREG,stat.S_IFDIR)),'unsupported_archive_entry'); names.add(str(relative))
            executable=[entry for entry in entries if not entry.is_dir() and pathlib.PurePosixPath(entry.filename).name==program and 'bin' in pathlib.PurePosixPath(entry.filename).parts]
            require(len(executable)==1,'executable_ambiguous_or_missing')
            if cached:
                expected={str(pathlib.PurePosixPath(entry.filename)) for entry in entries if not entry.is_dir()}|{'.formal-ai-source-archive.zip','formal-ai-install-receipt.json'}
                for target in prefix.rglob('*'):
                    require(not target.is_symlink(),'cached_symlink')
                    require(not target.is_file() or str(target.relative_to(prefix)) in expected,'cached_unexpected_file')
                for entry in entries:
                    if entry.is_dir(): continue
                    target=prefix/pathlib.PurePosixPath(entry.filename); require(target.is_file(),'cached_file_missing')
                    expected_digest=hashlib.sha256()
                    with bundle.open(entry) as input_file:
                        while True:
                            chunk=input_file.read(65536)
                            if not chunk: break
                            expected_digest.update(chunk)
                    require(file_digest(target)==expected_digest.hexdigest(),'cached_file_digest_mismatch')
                    expected_mode=0o700 if (entry.external_attr>>16)&0o111 else 0o600
                    require(stat.S_IMODE(target.stat().st_mode)==expected_mode,'cached_mode_mismatch')
                record('cache_verified',asset_sha256=observed,files=len(entries))
            else:
                prefix.mkdir()
                for entry in entries:
                    target=prefix/pathlib.PurePosixPath(entry.filename)
                    if entry.is_dir(): target.mkdir(parents=True,exist_ok=True); continue
                    target.parent.mkdir(parents=True,exist_ok=True)
                    with bundle.open(entry) as input_file,target.open('xb') as output: shutil.copyfileobj(input_file,output,65536)
                    target.chmod(0o700 if (entry.external_attr>>16)&0o111 else 0o600)
                shutil.copyfile(archive,retained)
    executable_path=prefix/executable[0].filename
    with executable_path.open('rb') as input_file: header=input_file.read(256)
    if header.startswith(b'#!'):
        import shlex
        interpreter=shlex.split(header.splitlines()[0][2:].decode('utf-8'))
        if interpreter and pathlib.Path(interpreter[0]).name=='env': interpreter=interpreter[1:]
        require(len(interpreter)==1,'unsupported_interpreter_directive'); name=pathlib.Path(interpreter[0]).name
        require(shutil.which(name) is not None and probe(shutil.which(name),['--version'],True),'interpreter_unusable')
    require(any(probe(executable_path,args) for args in grant.get('probes',[])),'installed_probe_failed')
    active=safe_path(pathlib.Path('.formal-ai/toolchains')/('active-'+program+'.bin'))
    require(not active.is_symlink(),'active_prefix_symlink')
    with tempfile.NamedTemporaryFile(mode='w',encoding='utf-8',dir=root,delete=False) as output:
        output.write(str(executable_path.parent)+'\n'); active_temporary=output.name
    os.replace(active_temporary,active)
    receipt=dict(program=program,publisher_source_id=publisher.get('source_id'),documentation=documentation,document_sha256=document_digest,repository=repo,release_id=release['id'],asset_id=asset['id'],asset_sha256=observed,prefix=str(prefix),executable=str(executable_path))
    with tempfile.NamedTemporaryFile(mode='w',encoding='utf-8',dir=prefix,delete=False) as output:
        output.write(json.dumps(receipt,sort_keys=True)+'\n'); receipt_temporary=output.name
    os.replace(receipt_temporary,prefix/'formal-ai-install-receipt.json'); record('installed',**receipt)

if __name__=='__main__':
    try:
        require(len(sys.argv)==2,'one_request_required'); execute(json.loads(sys.argv[1]))
    except Exception as error:
        record('refused',reason=str(error)); sys.exit(1)
