"""Read the literal package version with Python's standard TOML parser."""
import json
import sys
import tomllib


def package_version(content):
    package = tomllib.loads(content).get('package')
    version = package.get('version') if isinstance(package, dict) else None
    if not isinstance(version, str) or not version:
        raise ValueError('A literal package version is required')
    return version


if __name__ == '__main__':
    try:
        version = package_version(sys.stdin.buffer.read().decode('utf-8'))
    except (ValueError, UnicodeError):
        print('readable package version required; refusing unknown manifest', file=sys.stderr)
        sys.exit(1)
    print(json.dumps(version))
