import pathlib,json,hashlib
p=pathlib.Path('/private/tmp/pr1188-ci-T4047');changes=[]
def change(path,before,after):
 f=pathlib.Path(path);b=f.read_text();assert b.count(before)==1;(a:=b.replace(before,after));changes.append(dict(path=str(f.resolve()),before_sha256=hashlib.sha256(b.encode()).hexdigest(),after_sha256=hashlib.sha256(a.encode()).hexdigest(),content=a,before=b))
change('js/worker/formal_ai_worker_source_fetch.js','error: `no cached capture for ${url}`','error: answerFor("source_capture_offline_cache_miss", "en").replace("{url}", () => String(url))')
change('rust/src/source_fetch.rs','Self::OfflineCacheMiss(url) => write!(formatter, "no cached capture for {url}"),','''Self::OfflineCacheMiss(url) => {
                let template = crate::seed::localized_response("source_capture_offline_cache_miss", "en")
                    .ok_or(std::fmt::Error)?;
                formatter.write_str(&template.replace("{url}", url))
            }''')
f=pathlib.Path('data/seed/multilingual-responses-concept-lookup.lino');b=f.read_text();a=b+'''  response response_source_capture_offline_cache_miss_en
    intent source_capture_offline_cache_miss
    language en
    text "no cached capture for {url}"
''';changes.append(dict(path=str(f.resolve()),before_sha256=hashlib.sha256(b.encode()).hexdigest(),after_sha256=hashlib.sha256(a.encode()).hexdigest(),content=a,before=b));(p/'request.json').write_text(json.dumps(dict(changes=changes,original_job=114023635717),indent=2))
