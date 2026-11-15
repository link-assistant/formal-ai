#!/usr/bin/env python3
"""Host-only independent literal observations. Never infer executable use."""
import hashlib
import json
import re
import sys
try:
    import tomllib
except ImportError:
    tomllib = None


def unique_object(pairs):
    result = {}
    for name, value in pairs:
        if name in result:
            raise ValueError('DuplicateJsonKey')
        result[name] = value
    return result


def invalid_constant(_value):
    raise ValueError('NonfiniteJsonNumber')


def checked_json(source):
    return json.loads(source, object_pairs_hook=unique_object, parse_constant=invalid_constant)


CARGO_SECTIONS = ('dependencies', 'dev-dependencies', 'build-dependencies')
STRING_OPTIONS = {'package', 'path', 'git', 'branch', 'tag', 'rev', 'registry'}
BOOLEAN_OPTIONS = {'default-features', 'optional', 'workspace'}


def declaration(value):
    if isinstance(value, str):
        return {'version': value}
    if not isinstance(value, dict):
        raise ValueError('DependencyLiteralRequired')
    options = {}
    for name, item in value.items():
        if name == 'version' or name in STRING_OPTIONS:
            if not isinstance(item, str):
                raise ValueError('DependencyStringRequired')
        elif name in BOOLEAN_OPTIONS:
            if not isinstance(item, bool):
                raise ValueError('DependencyBooleanRequired')
        elif name == 'features':
            if not isinstance(item, list) or not all(isinstance(entry, str) for entry in item):
                raise ValueError('DependencyFeaturesRequired')
        else:
            raise ValueError('UnsupportedDependencyProperty')
        if name != 'version':
            options[name] = item
    if 'version' not in value and not any(name in value for name in ('path', 'git')) and value.get('workspace') is not True:
        raise ValueError('DependencyOriginRequired')
    result = {'version': value.get('version')}
    if options:
        result['options'] = options
    return result


def manifest_facts(owner, source):
    identity = hashlib.sha256(source.encode('utf-8')).hexdigest()
    base = {'manifest': owner, 'sourceIdentity': identity}
    facts = []
    if owner.endswith('.json'):
        document = checked_json(source)
        if not isinstance(document, dict):
            raise ValueError('ManifestObjectRequired')
        for section in ('dependencies', 'devDependencies', 'optionalDependencies', 'peerDependencies'):
            if section not in document:
                continue
            if not isinstance(document[section], dict):
                raise ValueError('DependencySectionRequired')
            for name, version in document[section].items():
                if not isinstance(version, str):
                    raise ValueError('DependencyVersionRequired')
                facts.append(dict(base, kind='manifest-declaration', section=section, name=name, version=version))
        return identity, facts
    if not owner.endswith('.toml') or tomllib is None:
        raise ValueError('IndependentTomlParserUnavailable')
    document = tomllib.loads(source)

    def dependencies(table, target=None):
        if not isinstance(table, dict):
            raise ValueError('DependencyTableRequired')
        for section in CARGO_SECTIONS:
            if section not in table:
                continue
            values = table[section]
            if not isinstance(values, dict):
                raise ValueError('DependencySectionRequired')
            for name, value in values.items():
                row = dict(base, kind='manifest-declaration', section=section, name=name, **declaration(value))
                if target is not None:
                    row['target'] = target
                facts.append(row)

    dependencies(document)
    if 'target' in document:
        if not isinstance(document['target'], dict):
            raise ValueError('TargetTableRequired')
        for target, table in document['target'].items():
            dependencies(table, target)
    if 'features' in document:
        if not isinstance(document['features'], dict):
            raise ValueError('FeatureTableRequired')
        for name, enables in document['features'].items():
            if not isinstance(enables, list) or not all(isinstance(item, str) for item in enables):
                raise ValueError('FeatureStringArrayRequired')
            facts.append(dict(base, kind='cargo-feature', section='features', name=name, enables=enables))
    return identity, facts


def main():
    request = sys.stdin.buffer.read(2 * 1024 * 1024 + 1)
    if len(request) > 2 * 1024 * 1024:
        raise ValueError('ObservationInputBudget')
    files = checked_json(request.decode('utf-8'))
    if not isinstance(files, list) or len(files) > 64:
        raise ValueError('ObservationFileBudget')
    facts, observations, seen = [], [], set()
    for file in files:
        owner, source = file['path'], file['content']
        if not owner.endswith(('.json', '.toml')):
            continue
        if owner in seen:
            raise ValueError('DuplicateObservationPath')
        seen.add(owner)
        identity = hashlib.sha256(source.encode('utf-8')).hexdigest()
        if len(source.encode('utf-8')) > 262144:
            observations.append({'path': owner, 'sourceIdentity': identity, 'status': 'unknown', 'reason': 'SourceBudget'})
            continue
        try:
            if file.get('claim') is True:
                claim = checked_json(source)
                observations.append({'path': owner, 'sourceIdentity': identity, 'status': 'observed', 'value': claim})
                continue
            identity, rows = manifest_facts(owner, source)
            facts.extend(rows)
            observations.append({'path': owner, 'sourceIdentity': identity, 'status': 'observed',
                                 'parser': 'python-json-unique-keys' if owner.endswith('.json') else 'python-tomllib',
                                 'scope': 'literal-declarations-only'})
        except (ValueError, TypeError, RecursionError, UnicodeError) as error:
            observations.append({'path': owner, 'sourceIdentity': identity, 'status': 'unknown', 'reason': str(error) if re.fullmatch(r'[A-Za-z][A-Za-z0-9]*', str(error)) else type(error).__name__})
    print(json.dumps({'facts': facts, 'observations': observations}, ensure_ascii=True, allow_nan=False, separators=(',', ':')))


if __name__ == '__main__':
    main()
