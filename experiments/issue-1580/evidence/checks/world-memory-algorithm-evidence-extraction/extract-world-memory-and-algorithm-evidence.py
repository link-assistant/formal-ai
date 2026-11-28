from pathlib import Path
import gzip, hashlib, json, os, subprocess, sys, traceback

root = Path('/tmp/gh-issue-solver-1794900398048')
scratch = Path('/tmp/issue-1580-research/world-memory-algorithm-evidence-extraction')
if len(sys.argv) == 1:
    scratch.mkdir(exist_ok=False)
    with (scratch / 'worker.log').open('wb') as output:
        process = subprocess.Popen([sys.executable, __file__, 'worker'], cwd=root, stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'pid': process.pid, 'directory': str(scratch)}))
    sys.exit(0)
records = []; environment = dict(os.environ, NODE_OPTIONS='--max-old-space-size=192')
def run(name, arguments, content=None):
    with (scratch / (name + '.stdout')).open('wb') as output, (scratch / (name + '.stderr')).open('wb') as errors:
        result = subprocess.run(arguments, cwd=root, env=environment, stdout=output, stderr=errors, input=content)
    record = {'name': name, 'argv': arguments, 'exitCode': result.returncode}; records.append(record)
    (scratch / 'records.json').write_text(json.dumps(records, indent=2) + '\n'); print(json.dumps(record), flush=True)
    assert result.returncode == 0, record
    return (scratch / (name + '.stdout')).read_bytes()
try:
    assert subprocess.check_output(['git', 'branch', '--show-current'], cwd=root, text=True).strip() == 'issue-1580-8d61a32a61e1'
    specifications = [
        {'parent': 'rust/src/world_model.rs', 'child': 'rust/src/world_model/dialogue_memory.rs', 'start': '/// Append-only event kinds for the current/general context boundary.', 'end': None,
         'anchor': 'use crate::world_model_cycles::collapse_truth_cycle;\n',
         'declarations': '\nmod dialogue_memory;\npub use dialogue_memory::{\n    ContextAccessEvent, ContextAccessEventKind, GeneralMemoryCommitError, GeneralMemoryPermission,\n    WorldModel,\n};\n',
         'childPrefix': '//! Permissioned access between dialogue-local and general world contexts.\n\nuse super::{Action, Context, ContextDiff, Prediction, RecalculationReport, stable_id};\nuse std::error::Error;\nuse std::fmt;\n\n',
         'parentEdits': [('use std::error::Error;\n', ''), ('use std::fmt::{self, Write as _};', 'use std::fmt::Write as _;')], 'privateExports': []},
        {'parent': 'rust/src/algorithm_discovery.rs', 'child': 'rust/src/algorithm_discovery/evidence.rs', 'start': 'fn candidate_identity(', 'end': "fn find_node<'a>(",
         'anchor': 'mod execution;\n', 'declarations': 'mod evidence;\n',
         'extraDeclarations': 'use evidence::{candidate_evidence_identity, candidate_identity, subsumes, validate_occurrence};\n',
         'childPrefix': '//! Stable algorithm evidence identities and held-out occurrence validation.\n\nuse super::{\n    AlgorithmCandidate, AlgorithmStep, ArgumentPattern, ExecutionTrace, HeldOutAlgorithmTest,\n    LinkAddress, Occurrence, structured_diagnostic,\n};\nuse std::collections::{BTreeMap, BTreeSet};\n\n',
         'parentEdits': [], 'privateExports': ['candidate_identity', 'candidate_evidence_identity', 'validate_occurrence', 'subsumes']}
    ]
    for specification in specifications:
        parent = root / specification['parent']; child = root / specification['child']
        assert not child.exists()
        before = parent.read_text(); (scratch / (parent.stem + '-before.rs.gz')).write_bytes(gzip.compress(before.encode(), mtime=0))
        start = before.index(specification['start']); end = before.index(specification['end']) if specification['end'] else len(before)
        block = before[start:end]; after = before[:start] + before[end:]
        if not specification['end']: after = after.rstrip() + '\n'
        assert after.count(specification['anchor']) == 1
        after = after.replace(specification['anchor'], specification['anchor'] + specification['declarations'])
        if 'extraDeclarations' in specification:
            after = after.replace('mod ranking;\n', 'mod ranking;\n' + specification['extraDeclarations'])
        for old, new in specification['parentEdits']:
            assert after.count(old) == 1; after = after.replace(old, new)
        body = block
        for name in specification['privateExports']:
            assert body.count('fn ' + name + '(') == 1
            body = body.replace('fn ' + name + '(', 'pub(super) fn ' + name + '(')
        child.parent.mkdir(exist_ok=True); child.write_text(specification['childPrefix'] + body.rstrip() + '\n'); parent.write_text(after)
        specification.update({'before': before, 'block': block, 'expectedParent': after, 'startOffset': start, 'endOffset': end})
    changed = [str(root / specification[kind]) for specification in specifications for kind in ['parent', 'child']]
    run('format', ['rustfmt', '--edition', '2024', *changed])
    proofs = []
    for specification in specifications:
        parent = root / specification['parent']; child = root / specification['child']
        # Final parent must equal the formatted declared removal/import transformation.
        formatted_parent = run(parent.stem + '-parent-normalization', ['rustfmt', '--edition', '2024', '--emit', 'stdout'], specification['expectedParent'].encode()).decode()
        assert parent.read_text() == formatted_parent
        content = child.read_text(); start = content.index(specification['start'].replace('fn ', 'pub(super) fn ', 1)) if specification['privateExports'] else content.index(specification['start'])
        prefix = content[:start]
        expected_prefix = run(parent.stem + '-imports-normalization', ['rustfmt', '--edition', '2024', '--emit', 'stdout'], (specification['childPrefix'] + 'fn placeholder() {}\n').encode()).decode().split('fn placeholder() {}')[0]
        assert prefix == expected_prefix
        restored = content[start:]
        for name in specification['privateExports']:
            restored = restored.replace('pub(super) fn ' + name + '(', 'fn ' + name + '(')
        normalized = run(parent.stem + '-moved-body-inverse', ['rustfmt', '--edition', '2024', '--emit', 'stdout'], restored.encode()).decode()
        original = run(parent.stem + '-original-body-normalization', ['rustfmt', '--edition', '2024', '--emit', 'stdout'], specification['block'].encode()).decode()
        assert normalized == original, 'All moved tokens, comments, literals, branches and public declarations reconstruct after declared visibility/formatting normalization'
        for path in [parent, child]: assert len(path.read_text().splitlines()) <= 900
        proofs.append({'parent': specification['parent'], 'child': specification['child'], 'originalLines': len(specification['before'].splitlines()), 'parentLines': len(parent.read_text().splitlines()), 'childLines': len(child.read_text().splitlines()), 'parentSha256': hashlib.sha256(parent.read_bytes()).hexdigest(), 'childSha256': hashlib.sha256(child.read_bytes()).hexdigest(), 'movedBodyNormalizationInverse': True, 'parentDeclaredTransformationOnly': True})
    for name, arguments in [
        ('source-currency', ['node', 'scripts/translate-js-rust.mjs', '--verify']),
        ('generated-registry-currency', ['node', 'scripts/lib/generate-native-specification-programs.mjs', '--check']),
        ('literal-owner-currency', ['node', 'scripts/check-hardcoded-language.mjs']),
        ('algorithm-regression', ['node', '--test', '--test-concurrency=1', 'rust/tests/web/issue-0531-pattern-inference.test.mjs']),
        ('process-boundary-regression', ['node', '--test', '--test-concurrency=1', 'rust/tests/web/bounded-subprocess-boundaries.test.mjs']),
        ('recipe-regression', ['node', '--test', '--test-concurrency=1', 'rust/tests/web/agentic-recipes-b.test.mjs']),
        ('source-gates', ['node', 'experiments/formal_ai_subagent/local-gates.mjs', '--only', 'check-abbreviations,check-file-size-js-twin,check-file-size,check-readable-code,check-generate-module-map-js-twin,check-runtime-feature-inventory']),
        ('format-check', ['rustfmt', '--edition', '2024', '--check', *changed]),
        ('whitespace', ['git', 'diff', '--check'])]: run(name, arguments)
    outcome = {'passed': True, 'modulePaths': proofs, 'publicPathsPreserved': True, 'localRustExecution': False, 'hostedNativeQualificationPending': True, 'fullIssueComplete': False, 'independentAuthorshipCredit': 0}
    (scratch / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    archive = root / 'experiments/issue-1580/evidence/checks/world-memory-algorithm-evidence-extraction'; archive.mkdir(exist_ok=False)
    for item in scratch.iterdir():
        if item.is_file() and item.name != 'worker.log': (archive / (item.name + '.gz')).write_bytes(gzip.compress(item.read_bytes(), mtime=0))
    (archive / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    (archive / 'extract-world-memory-and-algorithm-evidence.py').write_bytes(Path(__file__).read_bytes())
    (archive / 'README.md').write_text('Move the existing dialogue/general-memory permission boundary and algorithm evidence identity/held-out validation into private child modules. Public type paths are reexported unchanged. Final parents equal only the declared removal/import transformation; all moved bodies reconstruct under Rust formatting after reversing visibility declarations. No local Rust execution or full-issue completion is claimed. Hosted native qualification remains pending.\n')
    print(json.dumps(outcome), flush=True)
except BaseException:
    traceback.print_exc(); (scratch / 'failure.json').write_text(json.dumps({'passed': False, 'traceback': traceback.format_exc()}) + '\n'); sys.exit(1)
