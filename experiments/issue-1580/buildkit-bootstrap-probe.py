#!/usr/bin/env python3
"""Observe real hosted BuildKit daemons; inject only an exact canonical pull failure."""
from pathlib import Path
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def reference():
    action = Path(__file__).resolve().parents[2] / '.github/actions/setup-buildx-resilient/action.yml'
    raw = action.read_bytes()
    matches = re.findall(rb"^    default: '(moby/buildkit@sha256:[a-f0-9]{64})'$", raw, re.M)
    if len(matches) != 1:
        raise ValueError('exactly one immutable default BuildKit reference is required')
    return matches[0].decode(), raw


def prepare(directory):
    image, action = reference()
    directory.mkdir(parents=True, exist_ok=False)
    (directory / 'action.yml').write_bytes(action)
    write_json(directory / 'context.json', {'canonical': image, 'actionSha256': hashlib.sha256(action).hexdigest(),
        'head': os.environ.get('GITHUB_SHA'), 'workflowRef': os.environ.get('GITHUB_WORKFLOW_REF'),
        'workflowSha': os.environ.get('GITHUB_WORKFLOW_SHA'), 'runId': os.environ.get('GITHUB_RUN_ID')})


def install_fault(directory, real_docker=None):
    context = json.loads((directory / 'context.json').read_text())
    real_docker = real_docker or shutil.which('docker')
    if not real_docker or not Path(real_docker).is_absolute():
        raise ValueError('real Docker executable must be an absolute path')
    binary = directory / 'bin'
    binary.mkdir(exist_ok=False)
    write_json(directory / 'fault.json', {'canonical': context['canonical'], 'docker': real_docker,
        'trace': str(directory / 'injected-failures.jsonl')})
    script = '''#!/usr/bin/env python3
import json, os, sys
configuration = json.loads(open(CONFIGURATION).read())
args = sys.argv[1:]
if args and args[0] == 'pull' and configuration['canonical'] in args[1:]:
    with open(configuration['trace'], 'a') as stream:
        stream.write(json.dumps({'kind': 'deliberate-canonical-pull-failure', 'image': configuration['canonical']}) + '\\n')
    print('Error response from daemon: canonical registry unavailable in deliberate fallback control', file=sys.stderr)
    sys.exit(1)
os.execv(configuration['docker'], [configuration['docker'], *args])
'''.replace('CONFIGURATION', repr(str(directory / 'fault.json')))
    path = binary / 'docker'
    path.write_text(script)
    path.chmod(0o755)
    return binary


def command(directory, label, argv):
    result = subprocess.run(argv, capture_output=True)
    (directory / (label + '.stdout')).write_bytes(result.stdout)
    (directory / (label + '.stderr')).write_bytes(result.stderr)
    write_json(directory / (label + '.result.json'), {'argv': argv, 'exitCode': result.returncode})
    if result.returncode != 0:
        raise RuntimeError('actual Docker observation failed: ' + label)
    return result.stdout.decode()


def verify(directory, phase, image, builder):
    canonical = json.loads((directory / 'context.json').read_text())['canonical']
    mirror = 'mirror.gcr.io/' + canonical
    if image not in [canonical, mirror] or not re.fullmatch(r'[a-zA-Z0-9_.-]{1,128}', builder):
        raise ValueError('unexpected selected image or builder identity')
    if phase == 'mirror':
        if image != mirror:
            raise ValueError('deliberate canonical outage did not select the mirror digest')
        failures = [json.loads(line) for line in (directory / 'injected-failures.jsonl').read_text().splitlines()]
        if not failures or any(row != {'kind': 'deliberate-canonical-pull-failure', 'image': canonical} for row in failures):
            raise ValueError('canonical pull failure was not exercised exactly')
    elif phase != 'initial':
        raise ValueError('unknown bootstrap observation phase')
    inspection = command(directory, phase + '-builder', ['docker', 'buildx', 'inspect', builder, '--bootstrap'])
    for expected in [r'^Driver:\s+docker-container\s*$', r'^Status:\s+running\s*$', r'^BuildKit(?: version)?:\s+v?\d']:
        if not re.search(expected, inspection, re.M):
            raise ValueError('actual initialized builder observation missing: ' + expected)
    images = json.loads(command(directory, phase + '-image', ['docker', 'image', 'inspect', image]))
    if len(images) != 1:
        raise ValueError('exactly one selected Docker image is required')
    metadata = images[0]
    normalize = lambda value: value.removeprefix('docker.io/')
    if image not in [normalize(value) for value in metadata.get('RepoDigests', [])]:
        raise ValueError('selected index digest absent from actual Docker image metadata')
    containers = json.loads(command(directory, phase + '-container', ['docker', 'container', 'inspect', 'buildx_buildkit_' + builder + '0']))
    if len(containers) != 1 or not containers[0]['State']['Running'] or containers[0]['Image'] != metadata['Id']:
        raise ValueError('running builder daemon does not use the verified image ID')
    if normalize(containers[0]['Config']['Image']) != image:
        raise ValueError('running builder image reference differs from the selected digest')
    write_json(directory / (phase + '-verified.json'), {'actualDockerBootVerified': True, 'phase': phase,
        'selectedImage': image, 'canonical': canonical, 'builder': builder, 'imageId': metadata['Id'],
        'containerId': containers[0]['Id'], 'independentAuthorshipCredit': 0})
    print(phase + ': actual Docker builder is running with the selected immutable image')


def self_test():
    # This local control never starts or downloads Docker.
    with tempfile.TemporaryDirectory(prefix='formal-ai-buildkit-injector-') as temporary:
        directory = Path(temporary)
        image = 'moby/buildkit@sha256:' + 'a' * 64
        write_json(directory / 'context.json', {'canonical': image})
        delegated = directory / 'delegated.jsonl'
        real = directory / 'real-docker'
        real.write_text('#!/usr/bin/env python3\nimport json, sys\nwith open(' + repr(str(delegated)) + ", 'a') as stream: stream.write(json.dumps(sys.argv[1:]) + '\\n')\n")
        real.chmod(0o755)
        shim = install_fault(directory, str(real)) / 'docker'
        assert subprocess.run([str(shim), 'pull', image], capture_output=True).returncode == 1
        forwarded = [['pull', 'mirror.gcr.io/' + image], ['pull', 'moby/buildkit@sha256:' + 'b' * 64],
                     ['buildx', 'inspect', 'actual-builder', '--bootstrap'], ['version']]
        for args in forwarded:
            assert subprocess.run([str(shim), *args], capture_output=True).returncode == 0
        assert [json.loads(line) for line in delegated.read_text().splitlines()] == forwarded
        assert len((directory / 'injected-failures.jsonl').read_text().splitlines()) == 1
    print('fault injector rejects only the exact canonical pull; all other commands delegate unchanged')


if __name__ == '__main__':
    if len(sys.argv) == 2 and sys.argv[1] == 'self-test':
        self_test()
    elif len(sys.argv) == 3 and sys.argv[1] == 'prepare':
        prepare(Path(sys.argv[2]).resolve())
    elif len(sys.argv) == 3 and sys.argv[1] == 'install-fault':
        binary = install_fault(Path(sys.argv[2]).resolve())
        with open(os.environ['GITHUB_PATH'], 'a') as stream:
            stream.write(str(binary) + '\n')
    elif len(sys.argv) == 6 and sys.argv[1] == 'verify':
        verify(Path(sys.argv[2]).resolve(), *sys.argv[3:])
    else:
        raise SystemExit('expected self-test, prepare DIRECTORY, install-fault DIRECTORY or verify DIRECTORY PHASE IMAGE BUILDER')
