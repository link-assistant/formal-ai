from pathlib import Path
import gzip, hashlib, json, os, subprocess, sys, traceback

root = Path('/tmp/gh-issue-solver-1794900398048')
scratch = Path('/tmp/issue-1580-research/layout-and-equation-extraction')
if len(sys.argv) == 1:
    scratch.mkdir(exist_ok=False)
    with (scratch / 'worker.log').open('wb') as output:
        process = subprocess.Popen([sys.executable, __file__, 'worker'], cwd=root, stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'pid': process.pid, 'directory': str(scratch)}))
    sys.exit(0)
records = []
environment = dict(os.environ, NODE_OPTIONS='--max-old-space-size=192')
def run(name, arguments):
    with (scratch / (name + '.stdout')).open('wb') as output, (scratch / (name + '.stderr')).open('wb') as errors:
        result = subprocess.run(arguments, cwd=root, env=environment, stdout=output, stderr=errors)
    record = {'name': name, 'argv': arguments, 'exitCode': result.returncode}
    records.append(record)
    (scratch / 'records.json').write_text(json.dumps(records, indent=2) + '\n')
    print(json.dumps(record), flush=True)
    assert result.returncode == 0, record

try:
    assert subprocess.check_output(['git', 'branch', '--show-current'], cwd=root, text=True).strip() == 'issue-1580-8d61a32a61e1'
    layout_parent = root / 'rust/src/es_meta.rs'
    layout_child = root / 'rust/src/es_meta/source_layout.rs'
    equation_parent = root / 'rust/src/calculation.rs'
    equation_child = root / 'rust/src/calculation/linear_equation.rs'
    assert not layout_child.exists() and not equation_child.exists()
    layout_before = layout_parent.read_text()
    equation_before = equation_parent.read_text()
    for name, raw in [('layout-parent', layout_before), ('equation-parent', equation_before)]:
        (scratch / (name + '-before.rs.gz')).write_bytes(gzip.compress(raw.encode(), mtime=0))
    start = layout_before.index('/// The layout of one source:')
    end = layout_before.index('fn own_tree(')
    layout_capture = layout_before[start:end]
    start_render = layout_before.index('/// Emit target source, each token after')
    layout_render = layout_before[start_render:]
    layout_after = layout_before[:start] + layout_before[end:start_render].rstrip() + '\n'
    layout_declarations = '\nmod source_layout;\npub use source_layout::layout_of;\nuse source_layout::{slot_count, write_source};\n'
    anchor = 'use crate::seed::parser::{LinoNode, find_closing_quote, parse_lino, unescape_value};\n'
    assert layout_after.count(anchor) == 1
    layout_after = layout_after.replace(anchor, anchor + layout_declarations)
    layout_prefix = '//! Preserve and emit the whitespace and comments around ES token trees.\n\nuse super::{OwnedPart, OwnedTree, String, TemplatePart, ToString, Tree, Vec};\n\n'
    layout_body = layout_capture.replace('fn slot_count(', 'pub(super) fn slot_count(') + layout_render.replace('fn write_source(', 'pub(super) fn write_source(')
    layout_child.parent.mkdir(exist_ok=True)
    layout_child.write_text(layout_prefix + layout_body)
    layout_parent.write_text(layout_after)
    equation_start = equation_before.index('#[derive(Debug, Clone, Copy, PartialEq)]\nstruct LinearValue')
    equation_end = equation_before.index('pub fn contains_word_operator(')
    equation_block = equation_before[equation_start:equation_end]
    equation_after = equation_before[:equation_start] + equation_before[equation_end:]
    equation_after = equation_after.replace('mod function_phrase;\n', 'mod function_phrase;\nmod linear_equation;\n', 1)
    equation_after = equation_after.replace('pub use numeric_response::numeric_calculation_response;\n', 'pub use numeric_response::numeric_calculation_response;\nuse linear_equation::evaluate_linear_equation;\n', 1)
    equation_prefix = '//! Parse and evaluate the existing single-variable linear equation fallback.\n\nuse super::{ArithmeticError, CalculationEngine, CalculationEvaluation};\n\n'
    equation_body = equation_block.replace('fn evaluate_linear_equation(', 'pub(super) fn evaluate_linear_equation(')
    equation_child.write_text(equation_prefix + equation_body.rstrip() + '\n')
    equation_parent.write_text(equation_after)
    changed = [layout_parent, layout_child, equation_parent, equation_child]
    run('format', ['rustfmt', '--edition', '2024', *map(str, changed)])
    # Reverse only the declared boundaries, imports and private visibility.
    restored_layout = layout_parent.read_text().replace(layout_declarations, '')
    assert restored_layout != layout_parent.read_text()
    restored_layout = restored_layout.replace('fn own_tree(', layout_capture + 'fn own_tree(', 1).rstrip() + '\n\n' + layout_render
    assert restored_layout == layout_before, 'Complete final layout parent inverse'
    actual_layout_body = layout_child.read_text()
    assert actual_layout_body.startswith(layout_prefix)
    actual_layout_body = actual_layout_body[len(layout_prefix):].replace('pub(super) fn slot_count(', 'fn slot_count(').replace('pub(super) fn write_source(', 'fn write_source(')
    assert actual_layout_body == layout_capture + layout_render, 'Every moved layout byte'
    restored_equation = equation_parent.read_text().replace('mod linear_equation;\n', '').replace('use linear_equation::evaluate_linear_equation;\n', '')
    restored_equation = restored_equation.replace('pub fn contains_word_operator(', equation_block + 'pub fn contains_word_operator(', 1)
    assert restored_equation == equation_before, 'Complete final equation parent inverse'
    actual_equation_body = equation_child.read_text()
    assert actual_equation_body.startswith(equation_prefix)
    actual_equation_body = actual_equation_body[len(equation_prefix):].replace('pub(super) fn evaluate_linear_equation(', 'fn evaluate_linear_equation(').rstrip() + '\n\n'
    assert actual_equation_body == equation_block, 'Every moved equation byte'
    for p in changed:
        assert len(p.read_text().splitlines()) <= 900, str(p)
    (scratch / 'inverse-proof.txt').write_text('All final parent bytes and moved function/type/comment bytes reconstruct exactly after undoing only declared module imports, public reexport and private child visibility. No arithmetic constant, parsing branch, source-layout token, public path or test path changed.\n')
    translator = root / 'scripts/translate-es.mjs'
    old_translator = translator.read_text()
    old_reference = '// lives in js/agentic/crate/es_tokenizer.mjs so the agentic port can count\n// tokens too) and the renderer in rust/src/es_meta.rs (`layout_of` and\n'
    new_reference = '// lives in js/agentic/crate/es_tokenizer.mjs so the agentic port can count\n// tokens too) and the renderer in rust/src/es_meta/source_layout.rs (`layout_of` and\n'
    assert old_translator.count(old_reference) == 1
    translator.write_text(old_translator.replace(old_reference, new_reference))
    run('source-currency', ['node', 'scripts/translate-js-rust.mjs', '--verify'])
    run('generated-registry-currency', ['node', 'scripts/lib/generate-native-specification-programs.mjs', '--check'])
    run('literal-owner-currency', ['node', 'scripts/check-hardcoded-language.mjs'])
    run('translator-regression', ['node', '--test', '--test-concurrency=1', 'rust/tests/web/translate-es.test.mjs'])
    run('calculation-regression', ['node', '--test', '--test-concurrency=1', 'rust/tests/web/calculation-response-shape.test.mjs'])
    run('typescript-currency', ['node', 'scripts/translate-es.mjs', '--check'])
    run('source-gates', ['node', 'experiments/formal_ai_subagent/local-gates.mjs', '--only', 'check-abbreviations,check-file-size-js-twin,check-file-size,check-readable-code,check-generate-module-map-js-twin,check-runtime-feature-inventory'])
    run('format-check', ['rustfmt', '--edition', '2024', '--check', *map(str, changed)])
    run('whitespace', ['git', 'diff', '--check'])
    outcome = {'passed': True, 'exactFinalInverse': True, 'publicPathsPreserved': True, 'arithmeticAndLayoutBodiesUnchanged': True, 'modulePaths': [{'parent': str(p.relative_to(root)), 'parentLines': len(p.read_text().splitlines()), 'child': str(c.relative_to(root)), 'childLines': len(c.read_text().splitlines()), 'originalLines': len(original.splitlines()), 'parentSha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'childSha256': hashlib.sha256(c.read_bytes()).hexdigest()} for p, c, original in [(layout_parent, layout_child, layout_before), (equation_parent, equation_child, equation_before)]], 'localRustExecution': False, 'hostedNativeQualificationPending': True, 'independentAuthorshipCredit': 0, 'fullIssueComplete': False}
    (scratch / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    archive = root / 'experiments/issue-1580/evidence/checks/layout-and-equation-extraction'
    archive.mkdir(exist_ok=False)
    for item in scratch.iterdir():
        if item.is_file() and item.name != 'worker.log':
            (archive / (item.name + '.gz')).write_bytes(gzip.compress(item.read_bytes(), mtime=0))
    (archive / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    (archive / 'extract-layout-and-equation.py').write_bytes(Path(__file__).read_bytes())
    (archive / 'README.md').write_text('Separate existing ES source-layout capture/rendering and linear-equation fallback into private children. Final formatted parent and child bytes reconstruct exactly, excluding only declared imports, visibility and the preserved public layout_of reexport. The maintained translator source-path comment follows its actual owner. Existing JavaScript tests, source/generator currency and formatting are separate from pending hosted native qualification. Original 900-line warnings reproduce on both old files; both new parents/children fit the unchanged threshold. No complete issue or independent reasoning credit is claimed.\n')
    print(json.dumps(outcome), flush=True)
except BaseException:
    traceback.print_exc()
    (scratch / 'failure.json').write_text(json.dumps({'passed': False, 'traceback': traceback.format_exc()}) + '\n')
    sys.exit(1)
