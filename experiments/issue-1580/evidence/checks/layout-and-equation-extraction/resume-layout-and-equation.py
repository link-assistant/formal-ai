from pathlib import Path
import gzip, hashlib, json, os, subprocess, sys, traceback

root = Path('/tmp/gh-issue-solver-1794900398048')
scratch = Path('/tmp/issue-1580-research/layout-and-equation-extraction')
if len(sys.argv) == 1:
    for name in ['resume-worker.log', 'source-gates.stdout', 'source-gates.stderr']:
        path = scratch / name
        if path.exists():
            preserved = scratch / (name + '.initial.gz')
            assert not preserved.exists()
            preserved.write_bytes(gzip.compress(path.read_bytes(), mtime=0))
    with (scratch / 'resume-worker.log').open('wb') as output:
        process = subprocess.Popen([sys.executable, __file__, 'worker'], cwd=root, stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'pid': process.pid, 'directory': str(scratch)}))
    sys.exit(0)
records = json.loads((scratch / 'records.json').read_text())
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
    layout_before = gzip.decompress((scratch / 'layout-parent-before.rs.gz').read_bytes()).decode()
    equation_before = gzip.decompress((scratch / 'equation-parent-before.rs.gz').read_bytes()).decode()
    layout_capture = layout_before[layout_before.index('/// The layout of one source:'):layout_before.index('fn own_tree(')]
    layout_render = layout_before[layout_before.index('/// Emit target source, each token after'):]
    layout_declarations = '\nmod source_layout;\npub use source_layout::layout_of;\nuse source_layout::{slot_count, write_source};\n'
    restored_layout = layout_parent.read_text().replace(layout_declarations, '')
    assert restored_layout != layout_parent.read_text()
    restored_layout = restored_layout.replace('fn own_tree(', layout_capture + 'fn own_tree(', 1).rstrip() + '\n\n' + layout_render
    assert restored_layout == layout_before
    layout_prefix = '//! Preserve and emit the whitespace and comments around ES token trees.\n\nuse super::{OwnedPart, OwnedTree, String, TemplatePart, ToString, Tree, Vec};\n\n'
    layout_content = layout_child.read_text()
    assert layout_content.startswith(layout_prefix)
    assert layout_content[len(layout_prefix):].replace('pub(super) fn slot_count(', 'fn slot_count(').replace('pub(super) fn write_source(', 'fn write_source(') == layout_capture + layout_render
    equation_block = equation_before[equation_before.index('#[derive(Debug, Clone, Copy, PartialEq)]\nstruct LinearValue'):equation_before.index('pub fn contains_word_operator(')]
    restored_equation = equation_parent.read_text().replace('mod linear_equation;\n', '').replace('use linear_equation::evaluate_linear_equation;\n', '')
    restored_equation = restored_equation.replace('pub fn contains_word_operator(', equation_block + 'pub fn contains_word_operator(', 1)
    assert restored_equation == equation_before
    equation_prefix = '//! Parse and evaluate the existing single-variable linear equation fallback.\n\nuse super::{ArithmeticError, CalculationEngine, CalculationEvaluation};\n\n'
    equation_content = equation_child.read_text()
    assert equation_content.startswith(equation_prefix)
    signature = 'pub(super) fn evaluate_linear_equation(\n    expression: &str,\n) -> Result<CalculationEvaluation, ArithmeticError> {'
    restored_signature = 'fn evaluate_linear_equation(expression: &str) -> Result<CalculationEvaluation, ArithmeticError> {'
    assert equation_content.count(signature) == 1
    assert equation_block.count(restored_signature) == 1
    assert equation_content[len(equation_prefix):].replace(signature, restored_signature).rstrip() + '\n\n' == equation_block
    changed = [layout_parent, layout_child, equation_parent, equation_child]
    assert all(len(p.read_text().splitlines()) <= 900 for p in changed)
    (scratch / 'inverse-proof.txt').write_text('All final parent bytes and moved type/function/comment bytes reconstruct exactly after reversing declared module imports, child visibility and the public layout_of reexport. The widened evaluate_linear_equation visibility causes rustfmt to wrap only its signature over three lines; the inverse explicitly restores that exact header. No function body or arithmetic/layout token changed.\n')
    original = Path('/tmp/issue-1580-research/extract-layout-and-equation.py').read_text()
    tail = original[original.index('    translator = root'):original.index('\nexcept BaseException:')]
    tail = tail.replace('    assert old_translator.count(old_reference) == 1\n    translator.write_text(old_translator.replace(old_reference, new_reference))', '    assert old_translator.count(new_reference) == 1 or old_translator.count(old_reference) == 1\n    if old_reference in old_translator:\n        translator.write_text(old_translator.replace(old_reference, new_reference))')
    exec(compile('if True:\n' + tail, '<remaining-extraction-checks>', 'exec'), globals())
    (archive / 'extract-layout-and-equation.py').write_text(original)
    (archive / 'resume-layout-and-equation.py').write_bytes(Path(__file__).read_bytes())
    (archive / 'README.md').write_text('Separate unchanged ES source-layout capture/rendering and the linear-equation fallback into private children. Exact parent and child inverses reverse only declared imports, visibility, the layout_of public reexport and the single formatted equation signature wrap. The initial stricter inverse failure is retained alongside the successful corrected proof. Existing JavaScript tests and source/generator currency do not replace pending hosted native qualification. Both parents and children fit the unchanged 900-line warning threshold. No issue-completion or independent-authorship credit is claimed.\n')
except BaseException:
    traceback.print_exc()
    (scratch / 'resume-failure.json').write_text(json.dumps({'passed': False, 'traceback': traceback.format_exc()}) + '\n')
    sys.exit(1)
