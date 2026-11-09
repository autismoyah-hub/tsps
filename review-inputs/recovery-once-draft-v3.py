"""UNSELECTED cooperative case-wide collection guard; no network calls.

This is a local guard among callers using the same verified case directory. It
cannot prevent a bypass, another root, or a provider-side duplicate.
"""
import fnmatch
import os
import stat
from pathlib import Path

GUARD='case-collection-once'
PATTERNS=('collection-intent.json','collection-readback.json','collection-unknown.json',
          'status-*.json','logs-*.txt','records-*.jsonl','decoded-*',
          'recovery-collection-*','recovery-launch-stdout.txt','recovery-launch-stderr.txt',
          'recovery-launch-closure.json','recovery-collection*.json','collection-partial*',
          'collect-launch-stdout.txt','collect-launch-stderr.txt','collect-launch-closure.json',
          'collect-controller-*','recovery-controller-*','recovery-attempt-*','collection-*','partial-*')

def prior(names):
    return any(name == GUARD or any(fnmatch.fnmatchcase(name, pattern) for pattern in PATTERNS) for name in names)

def acquire(root):
    """Caller handles every exception as UNKNOWN and never deletes the guard.

    Caller must establish stable private ancestor custody, check named identity,
    and use an independent durable controller receipt for crashes/timeouts.
    Existing artifacts always refuse; post-mkdir failures retain the guard.
    """
    path=Path(root);fd=None;guardfd=None
    try:
        fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        original=os.fstat(fd);named=os.stat(path,follow_symlinks=False)
        if not stat.S_ISDIR(original.st_mode) or original.st_uid!=os.geteuid() or stat.S_IMODE(original.st_mode)!=0o700 or (original.st_dev,original.st_ino)!=(named.st_dev,named.st_ino): raise RuntimeError('ROOT_IDENTITY')
        if prior(os.listdir(fd)): raise RuntimeError('PRIOR_OR_AMBIGUOUS_ARTIFACT')
        os.mkdir(GUARD,0o700,dir_fd=fd)
        os.fsync(fd)
        guardfd=os.open(GUARD,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=fd)
        g=os.fstat(guardfd)
        if g.st_uid!=os.geteuid() or stat.S_IMODE(g.st_mode)!=0o700: raise RuntimeError('GUARD_IDENTITY')
        if any(any(fnmatch.fnmatchcase(n,p) for p in PATTERNS) for n in os.listdir(fd)): raise RuntimeError('POST_GUARD_ARTIFACT')
        g_now=os.fstat(guardfd);g_named=os.stat(GUARD,dir_fd=fd,follow_symlinks=False)
        if (g_now.st_dev,g_now.st_ino)!=(g.st_dev,g.st_ino) or (g_named.st_dev,g_named.st_ino)!=(g_now.st_dev,g_now.st_ino): raise RuntimeError('GUARD_REPLACED')
        now=os.fstat(fd);named_now=os.stat(path,follow_symlinks=False)
        if (now.st_dev,now.st_ino)!=(original.st_dev,original.st_ino) or (named_now.st_dev,named_now.st_ino)!=(now.st_dev,now.st_ino): raise RuntimeError('ROOT_CHANGED')
        os.close(fd);fd=None
        held=guardfd;guardfd=None
        return {'stage':'CLAIMED_LOCAL_COOPERATIVE_GUARD','guard':GUARD,'rootDevice':now.st_dev,'rootInode':now.st_ino,'guardDevice':g.st_dev,'guardInode':g.st_ino,'guardFD':held}
    finally:
        if guardfd is not None: os.close(guardfd)
        if fd is not None: os.close(fd)
