"""UNSELECTED OFFLINE DESIGN: injected adapter only; no real provider client.

Not a runnable production collector. Authenticated authority/claim inspection,
outer exclusive process control, bounded calls and independent durable failure
receipts must be supplied and reviewed by a separate real adapter/controller.
"""
import hashlib, json, os, pathlib, types

ROOT=pathlib.Path('/tmp/zogre-hf-case')
PINS={
 'authority':('recovery-authority-draft-v2.py','3214ba793361c149a4d089c4c13d1ce5ac923e60574fa3af473be660d61055e9'),
 'once':('recovery-once-draft-v2.py','3dae1fe3d65afc398f9d31905dfc45f68a77df04b7528fdc24b8759dcc25abd8'),
 'binding':('recovery-binding-draft-v1.py','722073938c846568ff073098702b72c2fd083b3d55081af050decd4b9b0daab6'),
 'records':('recovery-records-draft-v8.py','38ae7aae856550c5c324294efa81890c4a36ef6ae481d3db613bd84cdefb72c1'),
 'readback':('recovery-record-readback-v3.py','ebfa10796f79fd6e606d286885b160a2a160877fefef0110c1f2ea5e5ab8ee5d'),
 'observer':('recovery-native-events-draft-v1.py','d7a7a8b5416e142e29f3cb47873f006dd82a0eaf9e5824bf8cce004ce5aee1aa'),
}

def snapshots(root=ROOT):
    modules={}
    for name,(filename,digest) in PINS.items():
        path=root/filename;raw=path.read_bytes()
        if hashlib.sha256(raw).hexdigest()!=digest:raise RuntimeError('SNAPSHOT_MISMATCH')
        mod=types.ModuleType('verified_'+name);exec(compile(raw,str(path),'exec'),mod.__dict__);modules[name]=mod
    return modules

def durable(path, document):
    raw=(json.dumps(document,sort_keys=True,indent=2)+'\n').encode('ascii')
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    with os.fdopen(fd,'wb') as f:f.write(raw);f.flush();os.fsync(f.fileno())
    dfd=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:os.fsync(dfd)
    finally:os.close(dfd)

def run_fake_only(adapter, case_root, retained, selection_raw, now, modules):
    """Exercise ordering with a FAKE adapter. Never invoke on a real case.

    The adapter is deliberately not supplied here. Its fake evidence flags,
    current time, job projection and closure are not trusted authorization.
    Durable failure receipts can themselves fail; outer custody is required.
    """
    if adapter.kind!='OFFLINE_FAKE_ONLY':raise RuntimeError('NO_REAL_PROVIDER_ADAPTER')
    case_root=pathlib.Path(case_root);guard=None;result=None
    evidence=adapter.authority()
    if modules['authority'].validate(evidence,selection_raw,now)!='CORRELATED_REVIEW_ONLY':raise RuntimeError('AUTHORITY_UNKNOWN')
    if adapter.claim()!='15ecdacb4b53366c5d06ddd79adb1f9c480a0461':raise RuntimeError('CLAIM_UNKNOWN')
    observed=adapter.inspect()
    if modules['binding'].validate(retained,observed)!='BOUND_TERMINAL':raise RuntimeError('BINDING_UNKNOWN')
    # Repeat checks immediately before claim; fake evidence is not real proof.
    if modules['authority'].validate(adapter.authority(),selection_raw,now)!='CORRELATED_REVIEW_ONLY':raise RuntimeError('AUTHORITY_UNKNOWN')
    if adapter.claim()!='15ecdacb4b53366c5d06ddd79adb1f9c480a0461':raise RuntimeError('CLAIM_UNKNOWN')
    try:
        guard=modules['once'].acquire(case_root)
        work=case_root/modules['once'].GUARD
        fd=os.open(work,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:
            s=os.fstat(fd)
            if (s.st_dev,s.st_ino)!=(guard['guardDevice'],guard['guardInode']):raise RuntimeError('GUARD_CHANGED')
        finally:os.close(fd)
        durable(work/'collection-intent.json',{'kind':'COLLECTION_ONLY_FAKE_CONTROL','jobId':'6af14d46e6320c66fad7d183','stage':'PENDING_NO_RETRY','selectionSHA256':hashlib.sha256(selection_raw).hexdigest(),'historicalSubmission':'UNKNOWN','originalLabelsEquality':False,'providerStage':'ERROR'})
        # An interruption after acquisition leaves the persistent guard behind.
        adapter.before_fetch()
        result=modules['records'].retain_records(adapter.fetch(),work/'sdk-records.jsonl')
        durable(work/'records-result.json',result)
        if result.get('outcome')!='COMPLETE' or result.get('cleanupErrors') or result.get('omittedRecordState')!='NONE':raise RuntimeError('RECORDS_UNKNOWN')
        strings=modules['readback'].verified_records(work/'sdk-records.jsonl',result)
        observation=modules['observer'].observe(strings)
        durable(work/'native-observations.json',observation)
        closure=adapter.closure()
        # Fake closure is recorded as such; it does not prove child/provider closure.
        durable(work/'collection-readback.json',{'stage':'FAKE_CONTROL_ONLY','sdkRecordSHA256':result['fileSHA256'],'sdkRecords':result['records'],'nativeInterpretation':observation['interpretation'],'providerStage':'ERROR','historicalSubmission':'UNKNOWN','closure':closure,'scope':'SDK-decoded records; no gameplay/provider/process closure claim'})
        return {'stage':'FAKE_CONTROL_ONLY','guard':guard,'records':result}
    except BaseException:
        # Fake test hook models an independent controller; it is not durable or
        # independent until the real integrated controller is built and reviewed.
        try:adapter.record_unknown({'stage':'UNKNOWN_NO_RETRY','partialDetails':'UNAVAILABLE_OR_RETAINED_FILES','guardAcquired':guard is not None})
        except BaseException:pass
        raise
