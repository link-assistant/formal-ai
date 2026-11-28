from pathlib import Path
source = Path('/tmp/issue-1580-research/inventory-link-record-warning-checkpoint.py').read_text()
source = source.replace('link-record-warning-inventory', 'layout-equation-warning-inventory')
source = source.replace("assert not any(row['file'] == 'rust/src/link_store.rs' for row in data['warnings'])", "assert not any(row['file'] in ['rust/src/link_store.rs', 'rust/src/es_meta.rs', 'rust/src/calculation.rs'] for row in data['warnings'])")
source = source.replace("'linkStoreWarningResolved': True", "'linkStoreWarningResolved': True, 'sourceLayoutWarningResolved': True, 'linearEquationWarningResolved': True")
exec(compile(source, '<fresh-source-warning-inventory>', 'exec'), globals())
