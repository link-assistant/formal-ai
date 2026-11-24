from pathlib import Path
source = Path('/tmp/issue-1580-research/inventory-link-record-warning-checkpoint.py').read_text()
source = source.replace('link-record-warning-inventory', 'world-algorithm-warning-inventory')
source = source.replace("assert not any(row['file'] == 'rust/src/link_store.rs' for row in data['warnings'])", "assert not any(row['file'] in ['rust/src/link_store.rs', 'rust/src/codegen/es.rs', 'rust/src/calc/solve.rs', 'rust/src/world_model.rs', 'rust/src/algorithmization.rs'] for row in data['warnings'])")
source = source.replace("'linkStoreWarningResolved': True", "'allFiveSelectedParentWarningsResolved': True")
exec(compile(source, __file__, 'exec'))
