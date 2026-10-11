# Optional descriptor-relative physical adapter. No pathname fallback.
# Directory descriptors bind objects, not an immutable external namespace.
# External directory relocation is not certified by this adapter.
import errno
import json
import os
import stat
import sys


def create(root, path, content, expected_root=None):
    if (os.open not in os.supports_dir_fd or os.mkdir not in os.supports_dir_fd
            or not hasattr(os, "O_NOFOLLOW") or not hasattr(os, "O_DIRECTORY")):
        raise RuntimeError("UnsupportedDescriptorRelativeCreation")
    if not isinstance(root, str) or not isinstance(path, str) or not isinstance(content, str):
        raise ValueError("InvalidCreationTypes")
    parts = path.split("/")
    if not parts or any(part in ("", ".", "..") or "\x00" in part for part in parts):
        raise ValueError("InvalidRelativeCreationPath")
    payload = content.encode("utf-8", "strict")
    descriptors = []
    directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    try:
        parent = os.open(root, directory_flags)
        descriptors.append(parent)
        identity = os.fstat(parent)
        if not stat.S_ISDIR(identity.st_mode):
            raise ValueError("RootIsNotDirectory")
        if expected_root is not None and expected_root != {"dev": str(identity.st_dev), "ino": str(identity.st_ino)}:
            raise ValueError("RootIdentityMismatch")
        for part in parts[:-1]:
            try:
                child = os.open(part, directory_flags, dir_fd=parent)
            except FileNotFoundError:
                try:
                    os.mkdir(part, mode=0o700, dir_fd=parent)
                except FileExistsError:
                    pass
                child = os.open(part, directory_flags, dir_fd=parent)
            descriptors.append(child)
            parent = child
        target = os.open(parts[-1], os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                         0o600, dir_fd=parent)
        descriptors.append(target)
        if not stat.S_ISREG(os.fstat(target).st_mode):
            raise ValueError("TargetIsNotRegularFile")
        remaining = memoryview(payload)
        while remaining:
            written = os.write(target, remaining)
            if written <= 0:
                raise OSError("IncompleteCreationWrite")
            remaining = remaining[written:]
        os.fsync(target)
        os.fsync(parent)
        return {"success": True, "path": path, "bytes": len(payload)}
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)


try:
    request = json.load(sys.stdin)
    result = create(request["root"], request["path"], request["content"], request.get("root_identity"))
    print(json.dumps(result, ensure_ascii=True))
except Exception as error:
    print(json.dumps({"success": False, "error": type(error).__name__,
                      "code": errno.errorcode.get(getattr(error, "errno", None)), "message": str(error)}))
    sys.exit(1)
