"""Externally authored repair recipe. No autonomous authorship credit."""
from pathlib import Path

def replace(file, old, new):
    p=Path(file)
    text=p.read_text()
    if old not in text:
        raise RuntimeError(f'missing source anchor: {file}: {old[:100]}')
    p.write_text(text.replace(old,new))

replace('js/agentic/node-host.mjs', "import path from 'node:path';", "import path from 'node:path';\nimport {readNetworkAllowed} from '../runtime/read-access.mjs';")
replace('js/agentic/node-host.mjs', 'online only when `FORMAL_AI_LIVE_FETCH` is set to a true spelling.', 'online by default; explicit offline and network-deny controls take precedence.')
replace('js/agentic/node-host.mjs', 'online: flagEnabled(process.env.FORMAL_AI_LIVE_FETCH)', 'online: readNetworkAllowed(process.env)')
replace('js/agentic/node-host.mjs', "const flagEnabled = value => ['1', 'true', 'yes', 'on'].includes(String(value ?? '').trim().toLowerCase());\n", '')

replace('rust/src/cli_env.rs', '/// Parse a boolean `#[arg(env = ..)]` value for clap.', '''/// Native twin of `js/runtime/read-access.mjs`: read-only source retrieval is
/// default-on. An explicit operator offline/network deny always wins.
#[must_use]
pub fn read_network_allowed() -> bool {
    if std::env::var("FORMAL_AI_OFFLINE")
        .is_ok_and(|value| parse_bool_env(&value) != Some(false))
    {
        return false;
    }
    if std::env::var("FORMAL_AI_NETWORK").is_ok_and(|value| {
        matches!(value.trim().to_ascii_lowercase().as_str(), "denied" | "closed" | "off" | "0" | "false")
    }) {
        return false;
    }
    std::env::var("FORMAL_AI_LIVE_FETCH")
        .map_or(true, |value| parse_bool_env(&value) == Some(true))
}

/// Parse a boolean `#[arg(env = ..)]` value for clap.''')
replace('rust/src/coding/synthesis_runtime.rs','crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH")','crate::cli_env::read_network_allowed()')
p=Path('rust/src/solver_handlers/web_requests/live_search.rs')
s=p.read_text(); start=s.index('/// Whether the runtime opted'); end=s.index('/// Execute the web search',start)
s=s[:start]+'''/// Read-only retrieval is default-on; explicit offline/deny controls win.
/// See the JavaScript policy twin `js/runtime/read-access.mjs` (#1582).
pub fn live_fetch_enabled() -> bool {
    crate::cli_env::read_network_allowed()
}

'''+s[end:];p.write_text(s)

replace('js/server/structured-log.mjs', "stderrWrite=b=>fs.writeSync(process.stderr.fd,b),openFile", "stderrWrite,terminal=stderrWrite!==undefined,format='lino',openFile")
replace('js/server/structured-log.mjs', "if(!Object.hasOwn(levels,level))throw Error('invalid_log_level');", "if(!Object.hasOwn(levels,level))throw Error('invalid_log_level');\n if(!['lino','nested'].includes(format))throw Error('invalid_log_format');\n stderrWrite??=b=>fs.writeSync(process.stderr.fd,b);")
replace('js/server/structured-log.mjs', "sinks.push({name:'stderr',write:stderrWrite,enabled:true});", "if(terminal)sinks.push({name:'stderr',write:stderrWrite,enabled:true});")
replace('js/server/structured-log.mjs', "const bytes=Buffer.from(line),delivered=[];", "const bytes=Buffer.from(formatLogRecord(line.trimEnd(),format)+'\\n'),delivered=[];")
replace('js/server/structured-log.mjs', "const truth=v=>", '''// Whitespace-only presentation of the same typed tree, without a second codec.
export function formatLogRecord(line,format='lino'){
 if(format==='lino')return line;
 if(format!=='nested')throw Error('invalid_log_format');
 let depth=0,out='';
 for(const character of line){
  if(character==='('){if(depth)out=out.trimEnd()+'\\n'+'  '.repeat(depth);depth++;}
  else if(character===')')depth--;
  out+=character;
 }
 return out;
}

const truth=v=>''')
replace('js/server/structured-log.mjs', "return createLogger({env:environment,level,directory});", "return createLogger({env:environment,level,directory,terminal:options.terminal===true||truth(environment.FORMAL_AI_LOG_STDERR),format:options.logFormat??environment.FORMAL_AI_LOG_FORMAT??'lino',...(options.stderrWrite?{stderrWrite:options.stderrWrite}:{})});")
replace('js/server/main.mjs', "else if (name === '--debug' || name === '--verbose') options.logLevel = 'debug';", "else if (name === '--debug' || name === '--verbose') { options.logLevel = 'debug'; options.terminal = true; }\n    else if (name === '--log-stderr') options.terminal = true;\n    else if (name === '--log-format') { options.logFormat = value(); if (!options.logFormat) options.unsupported = arg; }")
# Configuration errors must remain visible even when normal diagnostics are file-only.
replace('js/server/main.mjs', "createServeLogger({logLevel:'error',env:", "createServeLogger({logLevel:'error',terminal:true,env:")

replace('rust/src/structured_log.rs','static SERVE: AtomicBool = AtomicBool::new(false);','static SERVE: AtomicBool = AtomicBool::new(false);\nstatic TERMINAL: AtomicBool = AtomicBool::new(false);')
replace('rust/src/structured_log.rs','pub fn configure(silent: bool, serve: bool)', 'pub fn configure(silent: bool, _serve: bool)')
replace('rust/src/structured_log.rs','SERVE.store(serve, Ordering::Relaxed);', 'SERVE.store(!silent, Ordering::Relaxed);')
replace('rust/src/structured_log.rs','if serve && !silent {','if !silent {')
replace('rust/src/structured_log.rs','fn rank(level: &str)', '''/// Explicit terminal diagnostics; normal wrapper/server output stays clear.
pub fn configure_terminal(verbose: bool) {
    TERMINAL.store(verbose, Ordering::Relaxed);
}

fn rank(level: &str)''')
replace('rust/src/structured_log.rs','let Ok(line) = encode(&redact(record)) else {','let Ok(mut line) = encode(&redact(record)) else {')
replace('rust/src/structured_log.rs','    let _guard = WRITE_LOCK', '''    let control = record["schema"] == "formal-ai-control-v1";
    if !control && std::env::var("FORMAL_AI_LOG_FORMAT").is_ok_and(|value| value == "nested") {
        line = nested_record(&line);
    }
    let _guard = WRITE_LOCK''')
replace('rust/src/structured_log.rs','if !STDERR_DISABLED.load(Ordering::Relaxed) && writeln!(io::stderr().lock(), "{line}").is_err()', '''if (control || TERMINAL.load(Ordering::Relaxed)
        || crate::solver_config::flag_enabled("FORMAL_AI_LOG_STDERR"))
        && !STDERR_DISABLED.load(Ordering::Relaxed) && writeln!(io::stderr().lock(), "{line}").is_err()''')
replace('rust/src/structured_log.rs','/// A bounded nonrecursive fallback:', '''/// Whitespace-only rendering of the typed tree; string punctuation is encoded.
fn nested_record(line: &str) -> String {
    let mut depth = 0;
    let mut out = String::new();
    for character in line.chars() {
        if character == '(' {
            if depth > 0 {
                out.truncate(out.trim_end().len());
                out.push('\\n');
                out.push_str(&"  ".repeat(depth));
            }
            depth += 1;
        } else if character == ')' {
            depth -= 1;
        }
        out.push(character);
    }
    out
}

/// A bounded nonrecursive fallback:''')
replace('rust/src/main.rs','formal_ai::dialog_log::configure_verbose(verbose);','formal_ai::dialog_log::configure_verbose(verbose);\n    formal_ai::dialog_log::structured_log::configure_terminal(verbose);')
