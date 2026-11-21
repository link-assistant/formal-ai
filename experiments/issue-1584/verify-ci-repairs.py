from pathlib import Path
import json,subprocess,sys
output=Path(sys.argv[1])
output.mkdir(parents=True,exist_ok=True)
commands=[
 ['node','--max-old-space-size=128','scripts/generate-worker-crate-modules.mjs','--write'],
 ['node','--max-old-space-size=128','scripts/generate-worker-prerequisites.mjs','--write'],
 ['node','--max-old-space-size=128','scripts/translate-es.mjs','--write'],
 ['node','--max-old-space-size=128','scripts/translate-es.mjs','--check'],
 ['node','--max-old-space-size=256','scripts/translate-js-rust.mjs','--write','--meta-language','.meta-language','--jobs','1','js/agentic/crate/cnl_output.mjs'],
 ['node','--max-old-space-size=128','scripts/translate-js-rust.mjs','--verify'],
 ['rustfmt','--edition','2024','--config','skip_children=true','--check','rust/src/cnl_output/projection.rs','rust/tests/unit/cnl_output.rs','rust/src/main.rs'],
]
tests=['rust/tests/web/cnl-output.test.mjs','experiments/issue-1584/server.test.mjs','rust/tests/web/server.test.mjs','rust/tests/web/issue-1550-procedure-census.test.mjs','rust/tests/web/cargo-audit-proof.test.mjs','rust/tests/web/module-map-twin.test.mjs','rust/tests/web/rust-specification-unicode-identifiers.test.mjs','rust/tests/web/rust-specification-fixture-integrity.test.mjs','rust/tests/web/runtime-feature-inventory.test.mjs']
commands += [['timeout','600','node','--max-old-space-size=128','--test','--test-concurrency=1',path] for path in tests]
commands += [['node','--max-old-space-size=128','experiments/issue-1584/capture-projections.mjs',str(output/'projections.jsonl')],['node','--max-old-space-size=256','experiments/formal_ai_subagent/local-gates.mjs']]
if '--gates-only' in sys.argv:
 commands=[command for command in commands if '--test' not in command and 'experiments/issue-1584/capture-projections.mjs' not in command]
for command in commands:
 print(json.dumps({'command':command}),flush=True)
 result=subprocess.run(command)
 if result.returncode: sys.exit(result.returncode)
print('All focused checks completed.',flush=True)
