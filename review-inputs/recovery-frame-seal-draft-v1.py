"""UNSELECTED bounded frame output content seal; no provider access."""
import hashlib, json, os, re, stat
from pathlib import Path
MAX_STREAMS=128
MAX_TREE_BYTES=16*1024*1024
MAX_MANIFEST_BYTES=1024*1024

def seal(directory, report):
    """Verify/frame-fsync output under exclusive private custody.

    This proves only bytes matching a COMPLETE frame report, never required
    stages, gameplay assertions, provider terminal, or process closure.
    Caller must protect stable private ancestors and durably retain the result.
    """
    if not isinstance(report,dict) or report.get('version')!=1 or report.get('complete') is not True or report.get('errors')!=[]:raise RuntimeError('FRAME_REPORT_INCOMPLETE')
    streams=report.get('streams')
    if not isinstance(streams,list) or not 1<=len(streams)<=MAX_STREAMS:raise RuntimeError('STREAM_COUNT')
    expected={};identities=set()
    for row in streams:
        if not isinstance(row,dict) or row.get('complete') is not True or row.get('errors')!=[]:raise RuntimeError('STREAM_INCOMPLETE')
        name=row.get('file');stage=row.get('stage');stream=row.get('stream');size=row.get('bytes');digest=row.get('sha256')
        if not isinstance(name,str) or not re.fullmatch(r'[0-9]{3}-(stdout|stderr)\.bin',name) or stream not in ('stdout','stderr') or not name.endswith('-'+stream+'.bin') or not isinstance(stage,str) or not 1<=len(stage)<=128 or type(size) is not int or not 0<=size<=MAX_TREE_BYTES or not isinstance(digest,str) or not re.fullmatch('[0-9a-f]{64}',digest) or name in expected or (stage,stream) in identities:raise RuntimeError('STREAM_SCHEMA')
        expected[name]=(size,digest);identities.add((stage,stream))
    if sum(size for size,_ in expected.values())>MAX_TREE_BYTES:raise RuntimeError('TREE_LIMIT')
    path=Path(directory);dfd=os.open(path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);errors=[];sealed={};primary=None;cancel=[]
    try:
        original=os.fstat(dfd);named=os.stat(path,follow_symlinks=False)
        if not stat.S_ISDIR(original.st_mode) or (original.st_dev,original.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('DIRECTORY_IDENTITY')
        if set(os.listdir(dfd))!=set(expected)|{'manifest.json'}:raise RuntimeError('TREE_CONTENTS')
        for name in sorted(expected.keys()|{'manifest.json'}):
            fd=None;item_error=None
            try:
                fd=os.open(name,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=dfd);before=os.fstat(fd);named_file=os.stat(name,dir_fd=dfd,follow_symlinks=False)
                if not stat.S_ISREG(before.st_mode) or before.st_nlink!=1 or (before.st_dev,before.st_ino)!=(named_file.st_dev,named_file.st_ino):raise RuntimeError('FILE_IDENTITY')
                limit=MAX_MANIFEST_BYTES if name=='manifest.json' else MAX_TREE_BYTES
                if before.st_size>limit:raise RuntimeError('FILE_LIMIT')
                chunks=[];size=0;digest=hashlib.sha256()
                while True:
                    data=os.read(fd,65536)
                    if not data:break
                    size+=len(data)
                    if size>limit:raise RuntimeError('FILE_LIMIT')
                    digest.update(data)
                    if name=='manifest.json':chunks.append(data)
                os.fsync(fd);after=os.fstat(fd);named_after=os.stat(name,dir_fd=dfd,follow_symlinks=False)
                if (after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns)!=(before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino):raise RuntimeError('FILE_CHANGED')
                h=digest.hexdigest()
                if name=='manifest.json':
                    if json.loads(b''.join(chunks))!=report:raise RuntimeError('MANIFEST_MISMATCH')
                elif (size,h)!=expected[name]:raise RuntimeError('STREAM_MISMATCH')
                sealed[name]={'bytes':size,'sha256':h}
            except BaseException as exc:item_error=exc
            finally:
                if fd is not None:
                    try:os.close(fd)
                    except BaseException as exc:
                        errors.append('FILE_CLOSE_ERROR')
                        if not isinstance(exc,Exception):cancel.append(exc)
            if item_error is not None:raise item_error
        if set(os.listdir(dfd))!=set(expected)|{'manifest.json'}:raise RuntimeError('TREE_CHANGED')
        final=os.fstat(dfd);named_final=os.stat(path,follow_symlinks=False)
        if (final.st_dev,final.st_ino)!=(original.st_dev,original.st_ino) or (named_final.st_dev,named_final.st_ino)!=(original.st_dev,original.st_ino):raise RuntimeError('DIRECTORY_CHANGED')
    except BaseException as exc:primary=exc
    finally:
        try:os.fsync(dfd)
        except BaseException as exc:
            errors.append('DIRECTORY_FSYNC_ERROR')
            if not isinstance(exc,Exception):cancel.append(exc)
        try:os.close(dfd)
        except BaseException as exc:
            errors.append('DIRECTORY_CLOSE_ERROR')
            if not isinstance(exc,Exception):cancel.append(exc)
    if primary is not None and not isinstance(primary,Exception):raise primary
    if cancel:raise cancel[0]
    if primary is not None or errors:
        failure=RuntimeError('FRAME_SEAL_UNKNOWN');failure.primaryType=type(primary).__name__ if primary else None;failure.cleanupCodes=errors;raise failure
    return {'stage':'FRAME_BYTES_SEALED_UNREVIEWED','files':sealed,'scope':'declared per-stream bytes only'}
