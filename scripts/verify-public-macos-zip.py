#!/usr/bin/env python3
"""Preflight the complete ZIP directory before ditto may restore its symlinks."""
import posixpath
import stat
import sys
import zipfile


def validate_archive(archive):
    entries = archive.infolist()
    if not entries or len(entries) > 200000:
        raise ValueError('ZIP member count outside finite budget')
    if sum(entry.file_size for entry in entries) > 8 * 1024**3:
        raise ValueError('ZIP expanded size exceeds eight GiB')
    names, links = set(), {}
    for entry in entries:
        name = entry.filename.rstrip('/')
        if (not name or name.startswith('/') or '\\' in name or
                any(part in ('', '.', '..') for part in name.split('/'))):
            raise ValueError('unsafe ZIP member path')
        if name in names:
            raise ValueError('duplicate ZIP member')
        names.add(name)
        if name.split('/')[0] not in ('formal-ai Desktop.app', '__MACOSX'):
            raise ValueError('unexpected ZIP root')
        mode = entry.external_attr >> 16
        if stat.S_ISLNK(mode):
            if entry.file_size > 4096:
                raise ValueError('oversized symlink')
            target = archive.read(entry).decode('utf-8', errors='strict')
            if not target or target.startswith('/') or '\\' in target:
                raise ValueError('unsafe ZIP symlink')
            links[name] = target
        elif stat.S_IFMT(mode) not in (0, stat.S_IFREG, stat.S_IFDIR):
            raise ValueError('unsupported ZIP special file')
    def resolve(name, seen=()):
        parts = name.split('/')
        for index in range(1, len(parts) + 1):
            prefix = '/'.join(parts[:index])
            if prefix in links:
                if prefix in seen or len(seen) >= 40:
                    raise ValueError('cyclic ZIP symlink')
                target = posixpath.normpath(posixpath.join(
                    posixpath.dirname(prefix), links[prefix], *parts[index:]))
                if target != parts[0] and not target.startswith(parts[0] + '/'):
                    raise ValueError('ZIP symlink escapes its root')
                return resolve(target, seen + (prefix,))
        return name
    for name in names:
        resolve(name)
    if 'formal-ai Desktop.app/Contents/Info.plist' not in names:
        raise ValueError('ZIP has no application Info.plist')


if __name__ == '__main__':
    with zipfile.ZipFile(sys.argv[1]) as archive:
        validate_archive(archive)
