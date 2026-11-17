"""Supplied review repair; execution through Formal AI earns no authorship credit."""
from pathlib import Path

for name in ['rust/src/solver_dispatch.rs', 'rust/src/meta_method_dispatch.rs']:
    p = Path(name)
    s = p.read_text()
    assert 'crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH")' in s
    p.write_text(s.replace('crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH")',
                           'crate::solver_config::read_network_allowed()'))

p = Path('rust/src/version_resolution.rs')
s = p.read_text().replace('/// The opt-in that lets generation reach the publishers\' APIs.\nconst LIVE_FETCH_ENV_VAR: &str = "FORMAL_AI_LIVE_FETCH";\n', '')
s = s.replace('/// the publisher\'s live API when `FORMAL_AI_LIVE_FETCH` opts the run into\n    /// the network (the repository\'s one live-retrieval switch), captured into',
              '/// the publisher\'s live API under the shared default-on read policy, captured into')
s = s.replace('crate::cli_env::flag_enabled(LIVE_FETCH_ENV_VAR)', 'crate::solver_config::read_network_allowed()')
p.write_text(s)
p = Path('rust/src/external_benchmarks/mod.rs')
s = p.read_text().replace('run_suite_with_online(manifest, slice, repository_root, live_fetch_enabled())',
    'run_suite_with_online(\n        manifest,\n        slice,\n        repository_root,\n        crate::solver_config::read_network_allowed(),\n    )')
s = s.replace('fn live_fetch_enabled() -> bool {\n    crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH")\n}\n\n', '')
p.write_text(s)
p = Path('rust/tests/unit/specification/external_benchmarks.rs')
s = p.read_text().replace('external_benchmarks::run_suite(editeval, DEFAULT_SLICE, &repo_root())',
                         'external_benchmarks::run_suite_with_online(editeval, DEFAULT_SLICE, &repo_root(), false)')
s = s.replace('external_benchmarks::run_suite(&missing_payload, 1, &repo_root())',
              'external_benchmarks::run_suite_with_online(&missing_payload, 1, &repo_root(), false)')
p.write_text(s)
p = Path('rust/src/source_fetch.rs')
p.write_text(p.read_text().replace('curl-backed transport used by opt-in live retrieval', 'curl-backed transport used by read-only live retrieval'))
p = Path('rust/src/solver_unknown_reasoning.rs')
s = p.read_text().replace('// response that names the focus. Live provider fetches stay behind\n        // the `FORMAL_AI_LIVE_FETCH` opt-in, so the runtime\'s offline mode\n        // keeps its no-network boundary.',
    '// response that names the focus. Provider reads use the shared\n        // default-on policy; the runtime\'s explicit offline mode still\n        // keeps its no-network boundary.')
p.write_text(s)
p = Path('rust/src/solver_handlers/web_requests.rs')
s = p.read_text().replace('/// fetches stay opt-in via `FORMAL_AI_LIVE_FETCH`, matching the combined\n/// offline gate `solver_dispatch.rs` applies to its `web_search` route.',
    '/// reads use the shared default-on policy, combined with the caller\'s\n/// explicit offline mode on every route.')
s = s.replace('// opt-in is applied inside `answer_web_search_query`.', '// policy is applied inside `answer_web_search_query`.')
s = s.replace('/// `FORMAL_AI_LIVE_FETCH` opt-in is applied here so every caller gets the\n/// combined offline gate `solver_dispatch.rs` uses at its `web_search` call\n/// site.', '/// shared read policy is applied here so every caller respects explicit\n/// offline and operator network-deny settings.')
p.write_text(s)
p = Path('js/agentic/crate/version_resolution.mjs')
p.write_text(p.read_text().replace('* (default `data`), online only under `FORMAL_AI_LIVE_FETCH`. A host without',
                                 '* (default `data`), online under the shared default-on read policy. A host without'))
p = Path('docs/requirements/issue-1548-vision-blockers-and-plan.md')
p.write_text(p.read_text().replace('`cached_or_research` is called from a test only and live fetching is off by default.',
    '`cached_or_research` is called from a test only. Read-only live fetching is now on by default under #1582; discovery orchestration remains partial.'))

p = Path('rust/tests/integration/with_formal_ai/issue_1582.rs')
s = p.read_text()
assert 'native_fetch_defaults_to_online_and_honors_each_explicit_deny' not in s
s += r'''

#[test]
fn native_fetch_defaults_to_online_and_honors_each_explicit_deny() {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    let dir = tmpdir();
    let listener = TcpListener::bind("127.0.0.1:0").expect("source server");
    listener.set_nonblocking(true).expect("nonblocking");
    let address = listener.local_addr().expect("address");
    let stop = Arc::new(AtomicBool::new(false));
    let server_stop = Arc::clone(&stop);
    let server = std::thread::spawn(move || {
        let start = std::time::Instant::now();
        let mut requests = 0;
        while !server_stop.load(Ordering::SeqCst)
            && start.elapsed() < std::time::Duration::from_secs(15)
        {
            if let Ok((mut stream, _)) = listener.accept() {
                stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).expect("read bound");
                stream.set_write_timeout(Some(std::time::Duration::from_secs(2))).expect("write bound");
                let mut request = [0; 8192];
                let count = stream.read(&mut request).expect("source request");
                assert!(String::from_utf8_lossy(&request[..count]).starts_with("GET /"));
                requests += 1;
                let body = "native-loopback-source-1582";
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).expect("response");
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        requests
    });
    let mut outputs = Vec::new();
    for (name, deny) in [
        ("default", None),
        ("offline", Some(("FORMAL_AI_OFFLINE", "1"))),
        ("network-denied", Some(("FORMAL_AI_NETWORK", "denied"))),
        ("live-disabled", Some(("FORMAL_AI_LIVE_FETCH", "0"))),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_formal-ai"));
        command.args(["chat", "--prompt", &format!("Fetch http://{address}/{name}")])
            .env("HOME", &dir)
            .env("FORMAL_AI_SOURCE_CACHE_DIR", dir.join(name))
            .env("NO_PROXY", "127.0.0.1")
            .env_remove("FORMAL_AI_OFFLINE")
            .env_remove("FORMAL_AI_NETWORK")
            .env_remove("FORMAL_AI_LIVE_FETCH")
            .env_remove("FORMAL_AI_LOG_STDERR")
            .stdin(Stdio::null());
        if let Some((key, value)) = deny { command.env(key, value); }
        outputs.push((name, command.output().expect("native fetch")));
    }
    stop.store(true, Ordering::SeqCst);
    let requests = server.join().expect("source worker");
    for (name, output) in outputs {
        assert!(output.status.success(), "{name}: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).contains("native-loopback-source-1582"), name == "default", "{name}: {}", String::from_utf8_lossy(&output.stdout));
    }
    assert_eq!(requests, 1, "explicit denies must prevent transport calls");
    std::fs::remove_dir_all(dir).expect("cleanup");
}
'''
p.write_text(s)
