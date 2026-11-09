"""Unselected, bounded SDK record readback; no provider or frame I/O."""
import hashlib, io, json, os, stat
from pathlib import Path
MAX_TOTAL=16*1024*1024
MAX_RECORDS=65536

def verified_records(path, receipt):
    """Return SDK strings only from a complete, verified byte snapshot.

    Content integrity only: byte-identical replacement is not distinguished.
    Caller must pin the original retention result, hold exclusive private
    directory custody, and handle every failure as UNKNOWN/no recollection.
    """
    if not isinstance(receipt,dict) or receipt.get('outcome')!='COMPLETE' or receipt.get('cleanupErrors')!=[] or receipt.get('omittedRecordState')!='NONE':
        raise RuntimeError('INCOMPLETE_RETENTION')
    count=receipt.get('records');size=receipt.get('fileBytes');digest=receipt.get('fileSHA256')
    if type(count) is not int or not 0<=count<=MAX_RECORDS or type(size) is not int or not 0<=size<=MAX_TOTAL or not isinstance(digest,str) or len(digest)!=64 or receipt.get('completeRecordBytes')!=size or receipt.get('completeRecordSHA256')!=digest:
        raise RuntimeError('RECEIPT_INVALID')
    path=Path(path);dfd=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);fd=None
    primary=None; cleanup=[]; values=None
    try:
        parent=os.fstat(dfd);named_parent=os.stat(path.parent,follow_symlinks=False)
        if (parent.st_dev,parent.st_ino)!=(named_parent.st_dev,named_parent.st_ino):raise RuntimeError('PARENT_CHANGED')
        fd=os.open(path.name,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=dfd)
        before=os.fstat(fd);named=os.stat(path.name,dir_fd=dfd,follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode) or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino) or before.st_size!=size:raise RuntimeError('FILE_IDENTITY')
        chunks=[];total=0
        while True:
            b=os.read(fd,65536)
            if not b:break
            total+=len(b)
            if total>MAX_TOTAL:raise RuntimeError('SIZE_LIMIT')
            chunks.append(b)
        raw=b''.join(chunks)
        after=os.fstat(fd);named_after=os.stat(path.name,dir_fd=dfd,follow_symlinks=False);parent_after=os.fstat(dfd);named_parent_after=os.stat(path.parent,follow_symlinks=False)
        if (after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns)!=(before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino) or (parent_after.st_dev,parent_after.st_ino)!=(parent.st_dev,parent.st_ino) or (named_parent_after.st_dev,named_parent_after.st_ino)!=(parent.st_dev,parent.st_ino):raise RuntimeError('READBACK_CHANGED')
        if len(raw)!=size or hashlib.sha256(raw).hexdigest()!=digest:raise RuntimeError('READBACK_MISMATCH')
        if raw and not raw.endswith(b'\n'):raise RuntimeError('RECORD_FRAMING')
        values=[]
        for line in io.BytesIO(raw):
            if len(values)>=count or len(values)>=MAX_RECORDS:raise RuntimeError('RECORD_COUNT_LIMIT')
            value=json.loads(line.decode('ascii'))
            if not isinstance(value,str):raise RuntimeError('RECORD_SCHEMA')
            values.append(value)
        if len(values)!=count:raise RuntimeError('RECORD_SCHEMA')
    except BaseException as exc:
        primary=exc
    finally:
        if fd is not None:
            try:os.close(fd)
            except BaseException as exc:cleanup.append(('FILE_CLOSE_ERROR',exc))
        try:os.close(dfd)
        except BaseException as exc:cleanup.append(('DIRECTORY_CLOSE_ERROR',exc))
    if primary is not None and not isinstance(primary,Exception):raise primary
    for _,exc in cleanup:
        if not isinstance(exc,Exception):raise exc
    if primary is not None or cleanup:
        error=RuntimeError('READBACK_UNKNOWN')
        error.primaryType=type(primary).__name__ if primary is not None else None
        error.cleanupCodes=[code for code,_ in cleanup]
        raise error
    return values
