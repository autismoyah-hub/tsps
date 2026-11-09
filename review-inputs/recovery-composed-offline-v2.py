"""UNSELECTED offline composition: fixed fake child, no provider or game I/O.

Dedicated foreground, single-threaded, exclusive child-reaper contract. The
script is not a live adapter, authorization boundary, or collection receipt.
"""
import argparse, hashlib, json, os, pathlib, stat, sys, types

ROOT=pathlib.Path('/tmp/zogre-hf-case')
CONTROL=pathlib.Path('/tmp/zogre-control/rsprox-linux/002/files/process_control.py')
CONTROL_SHA='565e79e713f6881cca014107e3a7d9bf502fc77dc8e3ead152baffed5a53d487'
OBSERVER_SHA='d7a7a8b5416e142e29f3cb47873f006dd82a0eaf9e5824bf8cce004ce5aee1aa'
PINS={'provider-return.json':'caeb899c756da00e736647b9ac415fc131bdbd68eeb06359dd705350db1bd3ee',
      'submission-unknown.json':'3520fdbd3b91f7f665a523093131c3eb92124295d21addc9921f3c78a428a949',
      'case-intent-v16.json':'0407632ba392942b1caa4570e847daaa0c44d634b584fbfa7393c6253d1512b7',
      'selection.json':'cadc7fa79eba914b2fc46fb444dc35f9cee5239e2a07dcd8dde480eb7aff35f8'}
FAKE_SELECTION=b'{"action":"OFFLINE_FAKE_ONLY","job":"6af14d46e6320c66fad7d183","version":1}\n'
FAKE_SELECTION_SHA=hashlib.sha256(FAKE_SELECTION).hexdigest()
FAKES={'ok':'import json; print(json.dumps(json.dumps({"event":"stage-start","name":"build"})),flush=True); print(json.dumps("FAKE SDK CHILD OUTPUT"),flush=True)',
       'nonzero':'import sys; print("FAKE_FAILURE",flush=True); sys.exit(7)',
       'timeout':'import time; print("FAKE_TIMEOUT",flush=True); time.sleep(2)',
       'death':'import time; print("FAKE_CHILD_STARTED",flush=True); time.sleep(2)'}

def write_at(dfd,name,value,inject=False):
    raw=(json.dumps(value,sort_keys=True,separators=(',',':'))+'\n').encode('ascii')
    fd=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=dfd)
    try:
        offset=0
        while offset<len(raw):
            n=os.write(fd,raw[offset:])
            if n<=0:raise RuntimeError('SHORT_WRITE')
            offset+=n
        if inject:raise OSError('synthetic sync failure')
        os.fsync(fd)
    finally:os.close(fd)
    os.fsync(dfd)
    # Immediate bounded readback is evidence of this process's successful path,
    # not proof that a later crash preserved the receipt on physical media.
    fd=os.open(name,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=dfd)
    try:
        actual=b''
        while len(actual)<=len(raw):
            chunk=os.read(fd,min(65536,len(raw)+1-len(actual)))
            if not chunk:break
            actual+=chunk
        if actual!=raw:raise RuntimeError('RECEIPT_READBACK')
    finally:os.close(fd)
    return hashlib.sha256(raw).hexdigest()

def snapshot_at(dfd,name,limit=65536):
    fd=os.open(name,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=dfd)
    try:
        before=os.fstat(fd);named=os.stat(name,dir_fd=dfd,follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode) or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('OUTPUT_IDENTITY')
        digest=hashlib.sha256();size=0;chunks=[]
        while True:
            data=os.read(fd,65536)
            if not data:break
            size+=len(data)
            if size>limit:raise RuntimeError('OUTPUT_LIMIT')
            digest.update(data);chunks.append(data)
        os.fsync(fd);after=os.fstat(fd);named_after=os.stat(name,dir_fd=dfd,follow_symlinks=False)
        if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (named_after.st_dev,named_after.st_ino)!=(after.st_dev,after.st_ino) or size!=after.st_size:raise RuntimeError('OUTPUT_CHANGED')
        return {'bytes':size,'sha256':digest.hexdigest(),'state':'GROUP_CLOSED_SNAPSHOT_NOT_FINAL'},b''.join(chunks)
    finally:os.close(fd)

def main():
    arg=argparse.ArgumentParser();arg.add_argument('--root',required=True);arg.add_argument('--mode',choices=FAKES,required=True);arg.add_argument('--fault',choices=('none','guard','intent','receipt','output'),default='none');a=arg.parse_args()
    root=pathlib.Path(a.root)
    if not root.name.startswith('zogre-composed-fake-'):raise RuntimeError('FAKE_ROOT_REQUIRED')
    originals={name:(ROOT/name).read_bytes() for name in PINS}
    for name,pin in PINS.items():
        if hashlib.sha256(originals[name]).hexdigest()!=pin:raise RuntimeError('ORIGINAL_PIN')
    original_return=json.loads(originals['provider-return.json']);original_unknown=json.loads(originals['submission-unknown.json'])
    original_intent=json.loads(originals['case-intent-v16.json']);request=original_intent['request']
    if (original_return.get('stage')!='RETURNED_UNINSPECTED' or original_unknown.get('stage')!='UNKNOWN_RECONCILE_NO_RETRY'
        or original_return.get('jobId')!='6af14d46e6320c66fad7d183' or original_return.get('namespace')!='Autismo-Yah'
        or original_return.get('claimCommit')!='15ecdacb4b53366c5d06ddd79adb1f9c480a0461'
        or original_return.get('selectionSHA256')!=PINS['selection.json']
        or hashlib.sha256(json.dumps(request,sort_keys=True).encode()).hexdigest()!='dc70c6973733afa4e1463b1acb1c8a5a10805e2f4408b8258cb83b4ee393dfb4'):
        raise RuntimeError('ORIGINAL_UNKNOWN')
    control=CONTROL.read_bytes()
    if hashlib.sha256(control).hexdigest()!=CONTROL_SHA:raise RuntimeError('CONTROL_PIN')
    mod=types.ModuleType('pinned_control');exec(compile(control,str(CONTROL),'exec'),mod.__dict__)
    observer_raw=(ROOT/'recovery-native-events-draft-v1.py').read_bytes()
    if hashlib.sha256(observer_raw).hexdigest()!=OBSERVER_SHA:raise RuntimeError('OBSERVER_PIN')
    observer=types.ModuleType('pinned_observer');exec(compile(observer_raw,'verified-observer','exec'),observer.__dict__)
    old_umask=os.umask(0o077);rootfd=None;guardfd=None;intent=False;error=None;closure=None;outputs={};snapshots={};observation=None;prior=[];cancel=None
    try:
        rootfd=os.open(root,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);before=os.fstat(rootfd);named=os.stat(root,follow_symlinks=False)
        if before.st_uid!=os.geteuid() or stat.S_IMODE(before.st_mode)!=0o700 or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('ROOT_IDENTITY')
        if os.listdir(rootfd):raise RuntimeError('PRIOR_ARTIFACT')
        if a.fault=='guard':raise OSError('synthetic guard failure')
        os.mkdir('guard',0o700,dir_fd=rootfd);os.fsync(rootfd)
        guardfd=os.open('guard',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=rootfd)
        opened=os.fstat(guardfd);guard_named=os.stat('guard',dir_fd=rootfd,follow_symlinks=False)
        if opened.st_uid!=os.geteuid() or stat.S_IMODE(opened.st_mode)!=0o700 or (opened.st_dev,opened.st_ino)!=(guard_named.st_dev,guard_named.st_ino):raise RuntimeError('GUARD_IDENTITY')
        intent_hash=write_at(guardfd,'outer-intent.json',{'stage':'FAKE_PENDING_NO_RETRY','fakeSelectionSHA256':FAKE_SELECTION_SHA,'originalSelectionSHA256':PINS['selection.json'],'historicalSubmission':'UNKNOWN','originalLabelsEquality':False},inject=a.fault=='intent');intent=True
        # /proc/self/fd refers to the held descriptor while the foreground
        # controller opens files; it is not passed to the spawned child as fd.
        outroot='/proc/self/fd/'+str(guardfd)
        closure=mod.run_group([sys.executable,'-c',FAKES[a.mode]],cwd=outroot,env=os.environ.copy(),stdout_path=outroot+'/stdout.txt',stderr_path=outroot+'/stderr.txt',timeout_seconds=0.25 if a.mode=='timeout' else 5,grace_seconds=0.1,output_limit=65536,file_limit=1048576)
        group_closed=(closure.get('exit_code')==0 and not closure.get('timed_out') and not closure.get('drain_incomplete') and not closure.get('drain_errors') and not closure.get('output_truncated') and not closure.get('group_remaining'))
        if group_closed:
            if a.fault=='output':raise OSError('synthetic output failure')
            for n in ('stdout.txt','stderr.txt'):outputs[n],snapshots[n]=snapshot_at(guardfd,n)
            fake_lines=snapshots['stdout.txt'].decode('ascii').splitlines()
            fake_records=[json.loads(line) for line in fake_lines]
            if not all(isinstance(x,str) for x in fake_records):raise RuntimeError('FAKE_SDK_SCHEMA')
            observation=observer.observe(fake_records)
        # Never read partial output as a complete snapshot when closure fails.
        gnow=os.fstat(guardfd);gnamed=os.stat('guard',dir_fd=rootfd,follow_symlinks=False);rnow=os.fstat(rootfd);rnamed=os.stat(root,follow_symlinks=False)
        if (gnow.st_dev,gnow.st_ino)!=(opened.st_dev,opened.st_ino) or (gnamed.st_dev,gnamed.st_ino)!=(opened.st_dev,opened.st_ino) or (rnow.st_dev,rnow.st_ino)!=(before.st_dev,before.st_ino) or (rnamed.st_dev,rnamed.st_ino)!=(before.st_dev,before.st_ino):raise RuntimeError('CUSTODY_CHANGED')
    except BaseException as exc:
        error=type(exc).__name__
        if not isinstance(exc,Exception):cancel=exc
    finally:
        if guardfd is not None:
            try:
                complete=(error is None and closure is not None and closure.get('exit_code')==0 and not closure.get('timed_out') and not closure.get('group_remaining') and not closure.get('drain_incomplete') and not closure.get('drain_errors') and not closure.get('output_truncated') and len(outputs)==2)
                write_at(guardfd,'outer-result.json',{'stage':'FAKE_GROUP_ONLY_UNREVIEWED' if complete else 'FAKE_UNKNOWN_NO_RETRY','originalSubmission':'UNKNOWN','providerStage':'NOT_QUERIED','originalLabelsEquality':False,'intentPersisted':intent,'fakeSelectionSHA256':FAKE_SELECTION_SHA,'controllerObservation':closure,'outputSnapshots':outputs,'fakeNativeObservations':observation,'errorType':error,'detachedDescendants':'UNKNOWN','signalPolicy':'no installed handlers; default process termination may preempt cleanup; first caught BaseException re-raised after attempted cleanup, repeated signals may preempt receipt','scope':'OFFLINE FAKE ONLY; group observation not payload or detached closure'},inject=a.fault=='receipt')
            except BaseException as exc:
                prior.append(type(exc).__name__)
                if not isinstance(exc,Exception) and cancel is None:cancel=exc
            try:os.close(guardfd)
            except BaseException as exc:
                prior.append(type(exc).__name__)
                if not isinstance(exc,Exception) and cancel is None:cancel=exc
        if rootfd is not None:
            try:os.close(rootfd)
            except BaseException as exc:
                prior.append(type(exc).__name__)
                if not isinstance(exc,Exception) and cancel is None:cancel=exc
        os.umask(old_umask)
    if cancel is not None:raise cancel
    if error is not None or prior:raise RuntimeError('FAKE_UNKNOWN_NO_RETRY')
    return 0 if closure is not None and closure.get('exit_code')==0 and not closure.get('timed_out') and len(outputs)==2 else 1
if __name__=='__main__':sys.exit(main())
