#!/usr/bin/env python3
"""Validate the loopback HTTP fixture only, without building or running Rust."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

spec = importlib.util.spec_from_file_location('registry_cli_fixture', Path(__file__).with_name('cli_regressions.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
actual_run = subprocess.run


def without_cargo_build(args, **kwargs):
    if args[:2] == ['cargo', 'build']:
        return subprocess.CompletedProcess(args, 0, stdout='', stderr='')
    return actual_run(args, **kwargs)


suite = unittest.TestSuite([module.RegistryRegressions(
    'test_fixture_can_return_wrong_identity_for_an_unpublished_candidate'),
    module.RegistryRegressions('test_fixture_serves_registry_overrides_as_exact_bytes'),
    module.RegistryRegressions('test_repository_contains_the_writer_status_staging_inputs')])
with patch.object(module.subprocess, 'run', side_effect=without_cargo_build):
    result = unittest.TextTestRunner(verbosity=2).run(suite)
print('HTTP fixture check only: no Rust compilation or release CLI execution.')
raise SystemExit(0 if result.wasSuccessful() else 1)
