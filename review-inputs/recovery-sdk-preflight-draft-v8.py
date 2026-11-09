"""UNSELECTED read-only SDK snapshot preflight; NEVER imports SDK or calls HF.

Must run in a separately bounded child. Successful preflight is not proof of a
future import: same-owner mutation, path reopening, runtime dependencies and
process custody remain separate reviewed requirements.
"""
import hashlib,json,os,pathlib,re,stat,sys
ANCHOR=pathlib.Path('/home/ubuntu/.zogre-recovery-42689c')
MANIFEST=ANCHOR/'sdk-tree-inventory-v1.json'
MANIFEST_SHA='7b6a8feadbf0d16d1beddc4fa46e448186301f0a843e56f38220abaa08f82e15'
PYTHON_SHA='c2c20b4745d447551221ec3d4e70f92c270c4609fe3df34fc52ea6dd46e92273'
MAX_ENTRIES=10000
MAX_BYTES=128*1024*1024
MAX_DEPTH=32
SDK_TOPLEVEL=('huggingface_hub','httpx2','httpcore2','h11','anyio','idna','tqdm','yaml','_yaml','filelock','fsspec','packaging','hf_xet','typing_extensions','truststore','click')

def verify_tree(rootfd,rows):
    """Compare bounded regular-file/directory tree via held nofollow FDs."""
    if not isinstance(rows,list) or len(rows)>MAX_ENTRIES:raise RuntimeError('MANIFEST_SIZE')
    wanted={}
    for row in rows:
        if not isinstance(row,dict):raise RuntimeError('MANIFEST_SCHEMA')
        path=row.get('path');p=pathlib.PurePosixPath(path) if isinstance(path,str) else None
        if p is None or p.is_absolute() or not p.parts or any(x in ('','.','..') for x in p.parts) or p.as_posix()!=path or path in wanted:raise RuntimeError('MANIFEST_PATH')
        if row.get('kind') not in ('file','directory'):raise RuntimeError('SYMLINK_OR_KIND')
        keys={'path','kind','mode'} if row['kind']=='directory' else {'path','kind','mode','bytes','sha256'}
        if set(row)!=keys or type(row.get('mode')) is not int or not 0<=row['mode']<=0o7777:raise RuntimeError('MANIFEST_SCHEMA')
        if row['kind']=='file' and (type(row.get('bytes')) is not int or not 0<=row['bytes']<=MAX_BYTES or not isinstance(row.get('sha256'),str) or not re.fullmatch('[0-9a-f]{64}',row['sha256'])):raise RuntimeError('MANIFEST_SCHEMA')
        wanted[path]=row
    root_before=os.fstat(rootfd)
    if not stat.S_ISDIR(root_before.st_mode):raise RuntimeError('ROOT_TYPE')
    seen=set();total=0;scanned=0
    def scan(dfd,prefix,depth):
        nonlocal total,scanned
        if depth>MAX_DEPTH:raise RuntimeError('DEPTH_LIMIT')
        with os.scandir(dfd) as iterator:
         for entry in iterator:
            scanned+=1
            if scanned>MAX_ENTRIES:raise RuntimeError('TREE_LIMIT')
            name=entry.name
            if name in ('.','..') or '/' in name:raise RuntimeError('NAME_INVALID')
            path=prefix+'/'+name if prefix else name;row=wanted.get(path)
            if row is None or path in seen:raise RuntimeError('TREE_EXTRA')
            seen.add(path);named=os.stat(name,dir_fd=dfd,follow_symlinks=False)
            if row['kind']=='directory':
                if not stat.S_ISDIR(named.st_mode):raise RuntimeError('DIRECTORY_TYPE')
                fd=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=dfd)
                try:
                    initial=os.fstat(fd)
                    if not stat.S_ISDIR(named.st_mode) or stat.S_IMODE(initial.st_mode)!=row.get('mode') or (initial.st_dev,initial.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('DIRECTORY_MISMATCH')
                    scan(fd,path,depth+1)
                    final=os.fstat(fd);named_end=os.stat(name,dir_fd=dfd,follow_symlinks=False)
                    if (initial.st_dev,initial.st_ino,initial.st_mtime_ns,initial.st_ctime_ns)!=(final.st_dev,final.st_ino,final.st_mtime_ns,final.st_ctime_ns) or (named_end.st_dev,named_end.st_ino)!=(final.st_dev,final.st_ino):raise RuntimeError('DIRECTORY_CHANGED')
                finally:os.close(fd)
            else:
                if not stat.S_ISREG(named.st_mode) or named.st_nlink!=1 or named.st_size>MAX_BYTES:raise RuntimeError('FILE_TYPE_OR_BOUND')
                fd=os.open(name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=dfd)
                try:
                    before=os.fstat(fd)
                    if not stat.S_ISREG(before.st_mode) or before.st_nlink!=1 or not stat.S_ISREG(named.st_mode) or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino) or stat.S_IMODE(before.st_mode)!=row.get('mode'):raise RuntimeError('FILE_IDENTITY')
                    digest=hashlib.sha256();size=0
                    while True:
                        chunk=os.read(fd,65536)
                        if not chunk:break
                        size+=len(chunk);total+=len(chunk)
                        if total>MAX_BYTES:raise RuntimeError('BYTE_LIMIT')
                        digest.update(chunk)
                    after=os.fstat(fd);named_end=os.stat(name,dir_fd=dfd,follow_symlinks=False)
                    if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (named_end.st_dev,named_end.st_ino)!=(after.st_dev,after.st_ino) or size!=row.get('bytes') or digest.hexdigest()!=row.get('sha256'):raise RuntimeError('FILE_CHANGED_OR_HASH')
                finally:os.close(fd)
    scan(rootfd,'',0)
    root_after=os.fstat(rootfd)
    if (root_before.st_dev,root_before.st_ino,root_before.st_mtime_ns,root_before.st_ctime_ns)!=(root_after.st_dev,root_after.st_ino,root_after.st_mtime_ns,root_after.st_ctime_ns):raise RuntimeError('ROOT_CHANGED')
    if seen!=set(wanted):raise RuntimeError('TREE_MISSING')
    return {'entries':len(seen),'bytes':total}

def reject_loaded_modules(modules):
    for name in modules:
        if not isinstance(name,str):raise RuntimeError('MODULE_NAME_INVALID')
        if any(name==top or name.startswith(top+'.') for top in SDK_TOPLEVEL):raise RuntimeError('SDK_DEPENDENCY_ALREADY_IMPORTED')

def read_manifest(afd):
    name='sdk-tree-inventory-v1.json'
    named=os.stat(name,dir_fd=afd,follow_symlinks=False)
    if not stat.S_ISREG(named.st_mode) or named.st_nlink!=1 or named.st_size>2*1024*1024:raise RuntimeError('MANIFEST_TYPE_OR_BOUND')
    mfd=os.open(name,os.O_RDONLY|os.O_NONBLOCK|os.O_NOFOLLOW,dir_fd=afd)
    try:
        before=os.fstat(mfd)
        if not stat.S_ISREG(before.st_mode) or before.st_nlink!=1 or before.st_size>2*1024*1024 or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('MANIFEST_IDENTITY')
        chunks=[];total=0
        while True:
            chunk=os.read(mfd,65536)
            if not chunk:break
            total+=len(chunk)
            if total>2*1024*1024:raise RuntimeError('MANIFEST_BOUND')
            chunks.append(chunk)
        raw=b''.join(chunks);after=os.fstat(mfd);named_end=os.stat(name,dir_fd=afd,follow_symlinks=False)
        if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (named_end.st_dev,named_end.st_ino)!=(after.st_dev,after.st_ino) or total!=before.st_size:raise RuntimeError('MANIFEST_CHANGED')
        return raw
    finally:os.close(mfd)

def preflight():
    """Identity check only. Caller must not infer trust or import authority."""
    if sys.version_info[:3]!=(3,12,3) or not sys.flags.isolated or not sys.flags.no_site or not sys.flags.dont_write_bytecode:raise RuntimeError('PYTHON_FLAGS')
    if pathlib.Path(sys.executable).resolve()!=pathlib.Path('/usr/bin/python3.12') or hashlib.sha256(pathlib.Path(sys.executable).read_bytes()).hexdigest()!=PYTHON_SHA:raise RuntimeError('PYTHON_PIN')
    reject_loaded_modules(tuple(sys.modules))
    afd=os.open(ANCHOR,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);sfd=None
    try:
        anchor=os.fstat(afd);named=os.stat(ANCHOR,follow_symlinks=False)
        if anchor.st_uid!=os.geteuid() or stat.S_IMODE(anchor.st_mode)!=0o700 or (anchor.st_dev,anchor.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('ANCHOR_IDENTITY')
        raw=read_manifest(afd)
        if hashlib.sha256(raw).hexdigest()!=MANIFEST_SHA:raise RuntimeError('MANIFEST_PIN')
        manifest=json.loads(raw)
        if not isinstance(manifest,dict) or type(manifest.get('schema')) is not int or manifest['schema']!=1 or manifest.get('root')!='/tmp/zogre-hf-sdk/site-packages':raise RuntimeError('MANIFEST_SCHEMA')
        sfd=os.open('sdk-snapshot-v1',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=afd)
        snap=os.fstat(sfd);named_snap=os.stat('sdk-snapshot-v1',dir_fd=afd,follow_symlinks=False)
        if (snap.st_dev,snap.st_ino)!=(named_snap.st_dev,named_snap.st_ino):raise RuntimeError('SNAPSHOT_IDENTITY')
        summary=verify_tree(sfd,manifest['entries'])
        end=os.fstat(sfd);named_end=os.stat('sdk-snapshot-v1',dir_fd=afd,follow_symlinks=False);anchor_end=os.fstat(afd);anchor_named_end=os.stat(ANCHOR,follow_symlinks=False)
        if (end.st_dev,end.st_ino,end.st_mtime_ns,end.st_ctime_ns)!=(snap.st_dev,snap.st_ino,snap.st_mtime_ns,snap.st_ctime_ns) or (named_end.st_dev,named_end.st_ino)!=(end.st_dev,end.st_ino) or (anchor_end.st_dev,anchor_end.st_ino)!=(anchor.st_dev,anchor.st_ino) or (anchor_named_end.st_dev,anchor_named_end.st_ino)!=(anchor.st_dev,anchor.st_ino):raise RuntimeError('SNAPSHOT_CHANGED')
        return {'stage':'PREFLIGHT_ONLY_NO_IMPORT','summary':summary,'manifestSHA256':MANIFEST_SHA,'anchorIdentity':{'device':anchor.st_dev,'inode':anchor.st_ino},'snapshotIdentity':{'device':snap.st_dev,'inode':snap.st_ino},'scope':'point-in-time bytes only; future import/immutability unproved'}
    finally:
        if sfd is not None:
            try:os.close(sfd)
            finally:os.close(afd)
        else:os.close(afd)

if __name__=='__main__':
    # This source draft is never an execution or provider authorization.
    raise RuntimeError('UNSELECTED_SDK_PREFLIGHT_REVIEW_ONLY')
