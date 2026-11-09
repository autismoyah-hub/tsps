"""UNSELECTED OFFLINE DESIGN: injected adapter only; no real provider client.

Not a runnable production collector. Authenticated authority/claim inspection,
outer exclusive process control, bounded calls and independent durable failure
receipts must be supplied and reviewed by a separate real adapter/controller.
"""
import hashlib, json, os, pathlib, types

ROOT=pathlib.Path('/tmp/zogre-hf-case')
PINS={
 'authority':('recovery-authority-draft-v2.py','3214ba793361c149a4d089c4c13d1ce5ac923e60574fa3af473be660d61055e9'),
 'once':('recovery-once-draft-v3.py','ecbcd5509377efbfdec568e82ebefccea4ab06780584eaa1d083ca4d55537ff6'),
 'binding':('recovery-binding-draft-v1.py','722073938c846568ff073098702b72c2fd083b3d55081af050decd4b9b0daab6'),
 'records':('recovery-records-draft-v9.py','7dab240ba10bcc47a1d8a5d9f258448eb79e402dd099b19b3c5007d7a71524e4'),
 'readback':('recovery-record-readback-v4.py','c0ac95483877f2fd4b0a0b34323a0686087a25740f9117ba11d69118303e0d9a'),
 'observer':('recovery-native-events-draft-v1.py','d7a7a8b5416e142e29f3cb47873f006dd82a0eaf9e5824bf8cce004ce5aee1aa'),
}

def snapshots(root=ROOT):
    modules={}
    for name,(filename,digest) in PINS.items():
        path=root/filename;raw=path.read_bytes()
        if hashlib.sha256(raw).hexdigest()!=digest:raise RuntimeError('SNAPSHOT_MISMATCH')
        mod=types.ModuleType('verified_'+name);exec(compile(raw,str(path),'exec'),mod.__dict__);modules[name]=mod
    return modules

def durable_at(dirfd, name, document):
    if pathlib.Path(name).name != name: raise RuntimeError('RELATIVE_NAME_REQUIRED')
    raw=(json.dumps(document,sort_keys=True,indent=2)+'\n').encode('ascii')
    fd=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=dirfd)
    with os.fdopen(fd,'wb') as f:f.write(raw);f.flush();os.fsync(f.fileno())
    os.fsync(dirfd)

def verify_guard(root, guard, guardfd):
    rootfd=os.open(root,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:
        r=os.fstat(rootfd);g=os.fstat(guardfd);named=os.stat(guard['guard'],dir_fd=rootfd,follow_symlinks=False)
        if (r.st_dev,r.st_ino)!=(guard['rootDevice'],guard['rootInode']) or (g.st_dev,g.st_ino)!=(guard['guardDevice'],guard['guardInode']) or (named.st_dev,named.st_ino)!=(g.st_dev,g.st_ino):raise RuntimeError('GUARD_REPLACED')
    finally:os.close(rootfd)

def run_fake_only(adapter, case_root, retained, selection_raw, modules):
    """Exercise ordering with a FAKE adapter. Never invoke on a real case.

    The adapter is deliberately not supplied here. Its fake evidence flags,
    current time, job projection and closure are not trusted authorization.
    Durable failure receipts can themselves fail; outer custody is required.
    """
    if adapter.kind!='OFFLINE_FAKE_ONLY':raise RuntimeError('NO_REAL_PROVIDER_ADAPTER')
    case_root=pathlib.Path(case_root);guard=None;guardfd=None;result=None;guard_state='NOT_ACQUIRED'
    try:
        evidence=adapter.authority()
        if modules['authority'].validate(evidence,selection_raw,adapter.now())!='CORRELATED_REVIEW_ONLY':raise RuntimeError('AUTHORITY_UNKNOWN')
        if adapter.claim()!='15ecdacb4b53366c5d06ddd79adb1f9c480a0461':raise RuntimeError('CLAIM_UNKNOWN')
        observed=adapter.inspect()
        if modules['binding'].validate(retained,observed)!='BOUND_TERMINAL':raise RuntimeError('BINDING_UNKNOWN')
        # Repeat checks immediately before claim; fake evidence is not real proof.
        if modules['authority'].validate(adapter.authority(),selection_raw,adapter.now())!='CORRELATED_REVIEW_ONLY':raise RuntimeError('AUTHORITY_UNKNOWN')
        if adapter.claim()!='15ecdacb4b53366c5d06ddd79adb1f9c480a0461':raise RuntimeError('CLAIM_UNKNOWN')
        # Once acquisition starts, an exception can occur after mkdir/fsync.
        guard_state='UNKNOWN'
        guard=modules['once'].acquire(case_root)
        guard_state='GUARDED'
        guardfd=guard['guardFD']
        verify_guard(case_root,guard,guardfd)
        durable_at(guardfd,'collection-intent.json',{'kind':'COLLECTION_ONLY_FAKE_CONTROL','jobId':'6af14d46e6320c66fad7d183','stage':'PENDING_NO_RETRY','selectionSHA256':hashlib.sha256(selection_raw).hexdigest(),'historicalSubmission':'UNKNOWN','originalLabelsEquality':False,'providerStage':'ERROR'})
        # Fresh clock, instructions and claim are checked again after acquisition.
        if modules['authority'].validate(adapter.authority(),selection_raw,adapter.now())!='CORRELATED_REVIEW_ONLY':raise RuntimeError('AUTHORITY_UNKNOWN')
        if adapter.claim()!='15ecdacb4b53366c5d06ddd79adb1f9c480a0461':raise RuntimeError('CLAIM_UNKNOWN')
        # An interruption after acquisition leaves the persistent guard behind.
        verify_guard(case_root,guard,guardfd)
        adapter.before_fetch()
        verify_guard(case_root,guard,guardfd)
        result=modules['records'].retain_records(adapter.fetch(),'sdk-records.jsonl',directory_fd=guardfd)
        durable_at(guardfd,'records-result.json',result)
        if result.get('outcome')!='COMPLETE' or result.get('cleanupErrors') or result.get('omittedRecordState')!='NONE':raise RuntimeError('RECORDS_UNKNOWN')
        strings=modules['readback'].verified_records('sdk-records.jsonl',result,directory_fd=guardfd)
        observation=modules['observer'].observe(strings)
        durable_at(guardfd,'native-observations.json',observation)
        verify_guard(case_root,guard,guardfd)
        closure=adapter.closure()
        # Fake closure is recorded as such; it does not prove child/provider closure.
        durable_at(guardfd,'collection-readback.json',{'stage':'FAKE_CONTROL_ONLY','sdkRecordSHA256':result['fileSHA256'],'sdkRecords':result['records'],'nativeInterpretation':observation['interpretation'],'providerStage':'ERROR','historicalSubmission':'UNKNOWN','closure':closure,'scope':'SDK-decoded records; no gameplay/provider/process closure claim'})
        return {'stage':'FAKE_CONTROL_ONLY','guard':{k:v for k,v in guard.items() if k!='guardFD'},'records':result}
    except BaseException:
        # Fake test hook models an independent controller; it is not durable or
        # independent until the real integrated controller is built and reviewed.
        try:adapter.record_unknown({'stage':'UNKNOWN_NO_RETRY','partialDetails':'UNAVAILABLE_OR_RETAINED_FILES','guardState':guard_state})
        except BaseException:pass
        raise
    finally:
        if guardfd is not None:
            try:os.close(guardfd)
            except BaseException:
                try:adapter.record_unknown({'stage':'UNKNOWN_NO_RETRY','guardState':guard_state,'failure':'GUARD_CLOSE_FAILED'})
                except BaseException:pass
                raise
