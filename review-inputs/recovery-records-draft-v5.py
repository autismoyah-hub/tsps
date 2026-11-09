"""Unselected draft: retain SDK strings as bounded JSONL, with no provider access."""
import hashlib
import json
import os
import stat
from pathlib import Path

MAX_RECORD = 1024 * 1024
MAX_TOTAL = 16 * 1024 * 1024


def retain_records(records, destination):
    """Caller owns a verified private directory and a durable exclusive intent.

    Reject records before writing when encoded JSONL exceeds the limit. The
    caller must durably persist this result and treat any failure as UNKNOWN.
    BaseException cancellation propagates after the finally block.
    """
    path = Path(destination)
    fd = None
    dfd = None
    dir_identity = None
    digest = hashlib.sha256()
    count = total = 0
    outcome, reason = 'UNKNOWN', 'CREATE_FAILED'
    disk_sha = None
    disk_size = None
    identity = None
    cleanup_errors = []
    pending_cancel = None
    try:
        dfd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        dir_identity = os.fstat(dfd)
        named_dir = os.stat(path.parent, follow_symlinks=False)
        if (dir_identity.st_dev, dir_identity.st_ino) != (named_dir.st_dev, named_dir.st_ino): raise RuntimeError('directory identity')
        fd = os.open(path.name, os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=dfd)
        identity = os.fstat(fd)
        if not stat.S_ISREG(identity.st_mode) or stat.S_IMODE(identity.st_mode) != 0o600:
            reason = 'FILE_IDENTITY'; return_result = False
        else:
            return_result = True
        if return_result:
            reason = 'ITERATION_ERROR'
            iterator = iter(records)
            while True:
                try:
                    value = next(iterator)
                except StopIteration:
                    outcome, reason = 'COMPLETE', 'NONE'
                    break
                except TimeoutError:
                    reason = 'TIMEOUT'; break
                except Exception:
                    reason = 'ITERATION_ERROR'; break
                if not isinstance(value, str):
                    reason = 'NON_STRING_RECORD'; break
                # JSON encoding of at most MAX_RECORD characters is at most
                # twelve times MAX_RECORD plus quotes/newline: finite allocation.
                if len(value) > MAX_RECORD - 3:
                    reason = 'RECORD_LIMIT'; break
                try:
                    data = (json.dumps(value, ensure_ascii=True, separators=(',', ':')) + '\n').encode('ascii')
                except TimeoutError:
                    reason = 'TIMEOUT'; break
                except Exception:
                    reason = 'ENCODE_ERROR'; break
                if len(data) > MAX_RECORD:
                    reason = 'RECORD_LIMIT'; break
                if total + len(data) > MAX_TOTAL:
                    reason = 'TOTAL_LIMIT'; break
                try:
                    n = os.write(fd, data)
                except TimeoutError:
                    reason = 'TIMEOUT'; break
                except Exception:
                    reason = 'WRITE_ERROR'; break
                if n != len(data):
                    reason = 'SHORT_WRITE'; break
                digest.update(data); total += len(data); count += 1
    except TimeoutError:
        outcome, reason = 'UNKNOWN', 'TIMEOUT'
    except Exception:
        outcome, reason = 'UNKNOWN', 'CREATE_OR_ITERATOR_ERROR'
    except BaseException as exc:
        outcome, reason = 'UNKNOWN', 'CANCELLED'
        pending_cancel = exc
    finally:
        if fd is not None:
            try:
                os.fsync(fd)
            except BaseException as exc:
                cleanup_errors.append('FSYNC_ERROR')
                if not isinstance(exc, Exception) and pending_cancel is None: pending_cancel = exc
            try:
                if identity is None: raise RuntimeError('unbound identity')
                before = os.fstat(fd); named = os.stat(path.name, dir_fd=dfd, follow_symlinks=False)
                if not stat.S_ISREG(before.st_mode) or (before.st_dev, before.st_ino) != (identity.st_dev, identity.st_ino) or (named.st_dev, named.st_ino) != (before.st_dev, before.st_ino):
                    raise RuntimeError('identity')
                os.lseek(fd, 0, os.SEEK_SET); actual = hashlib.sha256(); disk_size = 0
                while True:
                    chunk = os.read(fd, 65536)
                    if not chunk: break
                    disk_size += len(chunk)
                    if disk_size > MAX_TOTAL + MAX_RECORD: raise RuntimeError('bound')
                    actual.update(chunk)
                disk_sha = actual.hexdigest()
                after = os.fstat(fd); named_after = os.stat(path.name, dir_fd=dfd, follow_symlinks=False)
                if (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns) != (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) or (named_after.st_dev, named_after.st_ino) != (after.st_dev, after.st_ino):
                    raise RuntimeError('concurrent change')
                if outcome == 'COMPLETE' and (disk_size != total or disk_sha != digest.hexdigest()):
                    cleanup_errors.append('READBACK_MISMATCH')
            except BaseException as exc:
                cleanup_errors.append('READBACK_OR_IDENTITY_ERROR')
                if not isinstance(exc, Exception) and pending_cancel is None: pending_cancel = exc
            try:
                os.close(fd)
            except BaseException as exc:
                cleanup_errors.append('CLOSE_ERROR')
                if not isinstance(exc, Exception) and pending_cancel is None: pending_cancel = exc
        if dfd is not None:
            try:
                current_dir = os.fstat(dfd); named_dir = os.stat(path.parent, follow_symlinks=False)
                if dir_identity is None or (current_dir.st_dev, current_dir.st_ino) != (dir_identity.st_dev, dir_identity.st_ino) or (named_dir.st_dev, named_dir.st_ino) != (current_dir.st_dev, current_dir.st_ino):
                    raise RuntimeError('directory replaced')
            except BaseException as exc:
                cleanup_errors.append('DIRECTORY_IDENTITY_ERROR')
                if not isinstance(exc, Exception) and pending_cancel is None: pending_cancel = exc
            try:
                os.fsync(dfd)
            except BaseException as exc:
                cleanup_errors.append('DIRECTORY_FSYNC_ERROR')
                if not isinstance(exc, Exception) and pending_cancel is None: pending_cancel = exc
            try:
                os.close(dfd)
            except BaseException as exc:
                cleanup_errors.append('DIRECTORY_CLOSE_ERROR')
                if not isinstance(exc, Exception) and pending_cancel is None: pending_cancel = exc
    if cleanup_errors: outcome = 'UNKNOWN'
    if pending_cancel is not None:
        # The controller retains an independent UNKNOWN receipt and partial files.
        raise pending_cancel
    return {'outcome': outcome, 'reason': reason, 'cleanupErrors': cleanup_errors, 'records': count,
            'completeRecordBytes': total, 'completeRecordSHA256': digest.hexdigest(),
            'fileBytes': disk_size, 'fileSHA256': disk_sha,
            'encoding': 'ASCII JSONL of SDK-decoded strings',
            'maxRecordBytes': MAX_RECORD, 'maxTotalBytes': MAX_TOTAL}
