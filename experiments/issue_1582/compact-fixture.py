from pathlib import Path
p=Path('rust/tests/integration/with_formal_ai.rs');s=p.read_text();a=s.index('pub fn write_fake_cli(');b=s.index('fn write_fake_tui_cli(',a);fixture=s[a:b];s=s[:a]+'pub use fixture_env::write_fake_cli;\n\n'+s[b:];p.write_text(s)
p=Path('rust/tests/integration/with_formal_ai/fixture_env.rs');p.write_text(p.read_text()+'\nuse std::path::Path;\nuse std::os::unix::fs::PermissionsExt;\n\n'+fixture)
