"""UNSELECTED SDK loader design; fixed refusal before any SDK import or HF API.

This is source-review material, not a trusted import environment. A separately
reviewed launcher must snapshot-execute this file with -I -S -B and bind its
actual bytes. The refusal is intentional and cannot be removed without new
source/selection review.
"""
import hashlib, importlib, importlib.machinery, json, os, pathlib, stat, sys, types
PREFLIGHT=pathlib.Path('/tmp/zogre-hf-case/recovery-sdk-preflight-draft-v8.py')
PREFLIGHT_SHA='ad4fd8a9fa8a18c39e4f96b9a2de8ed28baef78ea397d97c64168ea267275dc0'
ANCHOR=pathlib.Path('/home/ubuntu/.zogre-recovery-42689c')
MANIFEST_SHA='7b6a8feadbf0d16d1beddc4fa46e448186301f0a843e56f38220abaa08f82e15'
STDLIB=('/usr/lib/python312.zip','/usr/lib/python3.12','/usr/lib/python3.12/lib-dynload')

def _not_ready():
    raise RuntimeError('SDK_IMPORT_AND_PROVIDER_DISABLED_PENDING_REVIEW')

def validate_origins(modules, roots, manifest):
    """Validate loaded *candidate* module origins; not execution provenance."""
    expected={row['path']:row for row in manifest['entries'] if row.get('kind')=='file'}
    snapshot=pathlib.Path(roots['snapshot']).resolve(strict=True)
    checked=[]
    for name,module in tuple(modules.items()):
        if not isinstance(name,str) or not any(name==top or name.startswith(top+'.') for top in roots['sdkTopLevel']):continue
        origin=getattr(module,'__file__',None)
        if not isinstance(origin,str):raise RuntimeError('MODULE_ORIGIN_UNKNOWN')
        unresolved=pathlib.Path(origin)
        if stat.S_ISLNK(unresolved.lstat().st_mode):raise RuntimeError('MODULE_SYMLINK')
        path=unresolved.resolve(strict=True)
        try:rel=path.relative_to(snapshot).as_posix()
        except ValueError:raise RuntimeError('MODULE_OUTSIDE_SNAPSHOT')
        row=expected.get(rel)
        if not isinstance(row,dict):raise RuntimeError('MODULE_NOT_IN_MANIFEST')
        expected_size=row.get('bytes')
        if type(expected_size) is not int or not 0<=expected_size<=128*1024*1024:raise RuntimeError('MODULE_BOUND')
        named=path.lstat()
        if not stat.S_ISREG(named.st_mode) or named.st_nlink!=1:raise RuntimeError('MODULE_TYPE')
        fd=os.open(path,os.O_RDONLY|os.O_NONBLOCK|os.O_NOFOLLOW)
        try:
            before=os.fstat(fd);digest=hashlib.sha256();size=0
            if not stat.S_ISREG(before.st_mode) or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('MODULE_IDENTITY')
            while True:
                data=os.read(fd,65536)
                if not data:break
                size+=len(data)
                if size>expected_size:raise RuntimeError('MODULE_BOUND')
                digest.update(data)
            after=os.fstat(fd);named_after=path.lstat()
            if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino) or size!=expected_size or digest.hexdigest()!=row.get('sha256'):raise RuntimeError('MODULE_BYTES_MISMATCH')
        finally:os.close(fd)
        checked.append(name)
    if 'huggingface_hub' not in checked or 'huggingface_hub.hf_api' not in checked or 'huggingface_hub._jobs_api' not in checked:raise RuntimeError('REQUIRED_MODULES_MISSING')
    return sorted(checked)

def candidate_load():
    """Unselectable sequence; no SDK import until explicit refusal is removed."""
    raw=PREFLIGHT.read_bytes()
    if hashlib.sha256(raw).hexdigest()!=PREFLIGHT_SHA:raise RuntimeError('PREFLIGHT_PIN')
    pre=types.ModuleType('verified_sdk_preflight');exec(compile(raw,str(PREFLIGHT),'exec'),pre.__dict__)
    before=pre.preflight()
    afd=os.open(ANCHOR,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);sfd=None;old_path=list(sys.path)
    try:
        sfd=os.open('sdk-snapshot-v1',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=afd)
        a=os.fstat(afd);s=os.fstat(sfd);named=os.stat('sdk-snapshot-v1',dir_fd=afd,follow_symlinks=False)
        if {'device':a.st_dev,'inode':a.st_ino}!=before['anchorIdentity'] or {'device':s.st_dev,'inode':s.st_ino}!=before['snapshotIdentity'] or (s.st_dev,s.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('SNAPSHOT_IDENTITY_CHANGED')
        # The candidate restricts path before import, but stdlib/interpreter,
        # import hooks, same-owner mutation and reopening are not yet proven.
        if sys.meta_path!=[importlib.machinery.BuiltinImporter,importlib.machinery.FrozenImporter,importlib.machinery.PathFinder]:raise RuntimeError('IMPORT_HOOKS_UNREVIEWED')
        sys.path[:]=['/proc/self/fd/'+str(sfd),*STDLIB]
        _not_ready()  # mandatory: no SDK import or provider call in this draft
        importlib.import_module('huggingface_hub')
        importlib.import_module('huggingface_hub.hf_api')
        importlib.import_module('huggingface_hub._jobs_api')
        manifest_raw=(ANCHOR/'sdk-tree-inventory-v1.json').read_bytes()
        if hashlib.sha256(manifest_raw).hexdigest()!=MANIFEST_SHA:raise RuntimeError('MANIFEST_PIN')
        manifest=json.loads(manifest_raw)
        checked=validate_origins(sys.modules,{'snapshot':str(ANCHOR/'sdk-snapshot-v1'),'sdkTopLevel':pre.SDK_TOPLEVEL},manifest)
        post=pre.verify_tree(sfd,manifest['entries'])
        return {'stage':'IMPORTED_UNVERIFIED_PENDING_OUTER','modules':checked,'tree':post,'scope':'no provider action or trusted execution claim'}
    finally:
        sys.path[:]=old_path
        if sfd is not None:
            try:os.close(sfd)
            finally:os.close(afd)
        else:os.close(afd)
if __name__=='__main__':_not_ready()
