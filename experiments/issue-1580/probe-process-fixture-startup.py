#!/usr/bin/env python3
"""Compare finite child-fixture startup without changing production clocks."""
import argparse
import json
from pathlib import Path
import runpy
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('wrapper')
parser.add_argument('mode', choices=['original', 'no-startup', 'clean-home', 'no-stdin'])
options = parser.parse_args()
probe = runpy.run_path(str(Path(__file__).with_name('probe-budget-process-lifetime.py')))
original_popen = subprocess.Popen
with tempfile.TemporaryDirectory(prefix='fixture-startup-home-') as home:
    def launch(command, **kwargs):
        command = list(command)
        if options.mode == 'no-startup':
            index = command.index('-c')
            command[index:index] = ['--noprofile', '--norc']
        if options.mode == 'clean-home':
            kwargs['env'] = dict(kwargs['env'], HOME=home)
        if options.mode == 'no-stdin':
            kwargs['stdin'] = subprocess.DEVNULL
        return original_popen(command, **kwargs)
    subprocess.Popen = launch
    print(json.dumps(probe['observe'](options.wrapper, 'short-success')))
