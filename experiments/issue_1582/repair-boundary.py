"""Keep the shared native read policy in solver configuration, not CLI surface."""
from pathlib import Path
p=Path('rust/src/cli_env.rs');s=p.read_text();a=s.index('/// Native twin of');b=s.index('/// Parse a boolean',a);policy=s[a:b];p.write_text(s[:a]+s[b:])
p=Path('rust/src/solver_config.rs');p.write_text(p.read_text()+'\n'+policy)
for p in Path('rust/src').rglob('*.rs'):
 s=p.read_text();updated=s.replace('crate::cli_env::read_network_allowed()', 'crate::solver_config::read_network_allowed()')
 if updated!=s:p.write_text(updated)
