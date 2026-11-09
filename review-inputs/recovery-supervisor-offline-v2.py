"""UNSELECTED offline supervisor for one pinned fake outer controller.

No real provider or GAME path. Missing/partial/unverified receipts are UNKNOWN;
this does not establish durable storage after a crash or detached closure. No
signal handlers are installed; default termination may preempt all cleanup.
The first caught cancellation is propagated after attempted receipt/cleanup;
further asynchronous signals can prevent the receipt and remain UNKNOWN.
"""
import argparse,base64,hashlib,json,os,pathlib,stat,subprocess,sys
ROOT=pathlib.Path('/tmp/zogre-hf-case')
OUTER=ROOT/'recovery-composed-offline-v2.py'
OUTER_SHA='a5ae72eb4b8c7b5ffa934e56de95a080bcd31718cba6ff3efe30b8af670af7e2'
BOOT="import base64,hashlib,sys; b=base64.b64decode(sys.argv[1],validate=True); h=sys.argv[2]; hashlib.sha256(b).hexdigest()==h or sys.exit(91); sys.argv=[sys.argv[0]]+sys.argv[3:];exec(compile(b,'verified-composed-fake','exec'),{'__name__':'__main__'})"

def read_at(fd,name,limit):
    f=os.open(name,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=fd)
    try:
        before=os.fstat(f);named=os.stat(name,dir_fd=fd,follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode) or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino) or before.st_size>limit:raise RuntimeError('READ_IDENTITY')
        data=b''
        while len(data)<=limit:
            part=os.read(f,min(65536,limit+1-len(data)))
            if not part:break
            data+=part
        after=os.fstat(f);named_after=os.stat(name,dir_fd=fd,follow_symlinks=False)
        if len(data)>limit or (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino):raise RuntimeError('READ_CHANGED')
        return data
    finally:os.close(f)

def durable_at(fd,name,obj,fail=False):
    data=(json.dumps(obj,sort_keys=True,separators=(',',':'))+'\n').encode('ascii')
    f=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=fd)
    try:
        n=os.write(f,data)
        if n!=len(data):raise RuntimeError('SHORT_WRITE')
        if fail:raise OSError('synthetic receipt failure')
        os.fsync(f)
    finally:os.close(f)
    os.fsync(fd)
    if read_at(fd,name,65536)!=data:raise RuntimeError('RECEIPT_READBACK')

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--root',required=True);ap.add_argument('--mode',choices=('ok','nonzero','timeout','death'),required=True);ap.add_argument('--outer-fault',choices=('none','guard','intent','receipt','output'),default='none');ap.add_argument('--supervisor-fault',choices=('none','receipt'),default='none');args=ap.parse_args()
    root=pathlib.Path(args.root)
    if not root.name.startswith('zogre-supervised-fake-'):raise RuntimeError('FAKE_ROOT_REQUIRED')
    outer=OUTER.read_bytes()
    if hashlib.sha256(outer).hexdigest()!=OUTER_SHA:raise RuntimeError('OUTER_PIN')
    rootfd=os.open(root,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);gfd=None;result=None;error=None;cancel=None;receipt_error=None;child_exit=None;timed_out=False
    try:
        rootinfo=os.fstat(rootfd);named=os.stat(root,follow_symlinks=False)
        if rootinfo.st_uid!=os.geteuid() or stat.S_IMODE(rootinfo.st_mode)!=0o700 or (rootinfo.st_dev,rootinfo.st_ino)!=(named.st_dev,named.st_ino) or os.listdir(rootfd):raise RuntimeError('ROOT_OR_PRIOR_ARTIFACT')
        # Outer currently requires a different fake prefix. A private child
        # subdirectory is allocated before start and remains in this root.
        os.mkdir('zogre-composed-fake-inner',0o700,dir_fd=rootfd);os.fsync(rootfd)
        inner=str(root/'zogre-composed-fake-inner')
        argv=[sys.executable,'-c',BOOT,base64.b64encode(outer).decode(),OUTER_SHA,'--root',inner,'--mode',args.mode,'--fault',args.outer_fault]
        try:
            child=subprocess.run(argv,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=8)
            child_exit=child.returncode
        except subprocess.TimeoutExpired:
            timed_out=True
        # Never interpret a child receipt on nonzero/timeout: it may be partial,
        # unsynced, or from a still-running detached descendant.
        if not timed_out and child_exit==0:
            if hashlib.sha256(FAKE_SELECTION()).hexdigest()!='5d74490817b8ae3330c80283328a587f53dc71815b194303be73cf09bbd35f06':raise RuntimeError('SELECTION_PIN')
            innerfd=os.open('zogre-composed-fake-inner',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=rootfd)
            try:gfd=os.open('guard',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=innerfd)
            finally:os.close(innerfd)
            intent_raw=read_at(gfd,'outer-intent.json',65536);result_raw=read_at(gfd,'outer-result.json',65536)
            intent=json.loads(intent_raw);result=json.loads(result_raw)
            if intent.get('stage')!='FAKE_PENDING_NO_RETRY' or intent.get('fakeSelectionSHA256')!='5d74490817b8ae3330c80283328a587f53dc71815b194303be73cf09bbd35f06' or result.get('stage')!='FAKE_GROUP_ONLY_UNREVIEWED' or result.get('historicalSubmission',result.get('originalSubmission'))!='UNKNOWN' or result.get('providerStage')!='NOT_QUERIED' or result.get('detachedDescendants')!='UNKNOWN':raise RuntimeError('RESULT_SCHEMA')
            snapshots=result.get('outputSnapshots')
            if not isinstance(snapshots,dict) or set(snapshots)!={'stdout.txt','stderr.txt'}:raise RuntimeError('OUTPUT_SCHEMA')
            for name,entry in snapshots.items():
                data=read_at(gfd,name,65536)
                if entry.get('bytes')!=len(data) or entry.get('sha256')!=hashlib.sha256(data).hexdigest() or entry.get('state')!='GROUP_CLOSED_SNAPSHOT_NOT_FINAL':raise RuntimeError('OUTPUT_MISMATCH')
            result={'stage':'FAKE_VALIDATED_GROUP_ONLY','outerResultSHA256':hashlib.sha256(result_raw).hexdigest(),'outerIntentSHA256':hashlib.sha256(intent_raw).hexdigest(),'fakeSelectionSHA256':intent['fakeSelectionSHA256'],'originalSubmission':'UNKNOWN','providerStage':'NOT_QUERIED','detachedDescendants':'UNKNOWN','scope':'OFFLINE FAKE ONLY; no provider or complete closure claim'}
        else:
            result={'stage':'FAKE_UNKNOWN_NO_RETRY','outerExit':child_exit,'outerTimeout':timed_out,'originalSubmission':'UNKNOWN','providerStage':'NOT_QUERIED','detachedDescendants':'UNKNOWN'}
    except BaseException as exc:
        error=exc
        if not isinstance(exc,Exception):cancel=exc
        result={'stage':'FAKE_UNKNOWN_NO_RETRY','errorType':type(exc).__name__,'originalSubmission':'UNKNOWN','providerStage':'NOT_QUERIED','detachedDescendants':'UNKNOWN'}
    finally:
        if gfd is not None:
            try:os.close(gfd)
            except BaseException as exc:
                result={'stage':'FAKE_UNKNOWN_NO_RETRY','errorType':'GUARD_CLOSE','detachedDescendants':'UNKNOWN'}
                if not isinstance(exc,Exception) and cancel is None:cancel=exc
    try:
        last=os.fstat(rootfd);named_last=os.stat(root,follow_symlinks=False)
        if (last.st_dev,last.st_ino)!=(rootinfo.st_dev,rootinfo.st_ino) or (named_last.st_dev,named_last.st_ino)!=(last.st_dev,last.st_ino):raise RuntimeError('ROOT_CHANGED')
    except BaseException as exc:
        result={'stage':'FAKE_UNKNOWN_NO_RETRY','errorType':'ROOT_IDENTITY','detachedDescendants':'UNKNOWN'}
        if not isinstance(exc,Exception) and cancel is None:cancel=exc
    try:
        durable_at(rootfd,'supervisor-result.json',result,fail=args.supervisor_fault=='receipt')
    except BaseException as exc:
        receipt_error=exc
        if not isinstance(exc,Exception) and cancel is None:cancel=exc
    finally:
        try:os.close(rootfd)
        except BaseException as exc:
            if receipt_error is None:receipt_error=exc
            if not isinstance(exc,Exception) and cancel is None:cancel=exc
    if cancel is not None:raise cancel
    if receipt_error is not None:raise receipt_error
    return 0 if result['stage']=='FAKE_VALIDATED_GROUP_ONLY' else 1

def FAKE_SELECTION():
    return b'{"action":"OFFLINE_FAKE_ONLY","job":"6af14d46e6320c66fad7d183","version":1}\n'
if __name__=='__main__':sys.exit(main())
