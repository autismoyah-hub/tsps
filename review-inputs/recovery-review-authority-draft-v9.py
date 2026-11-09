"""UNSELECTED named-reviewer origin caller; no provider or collector calls.

This is a source-review draft. Matching named turns is not collection authority:
outer completion, private custody, other-session instructions, live selection,
process closure and independent review remain mandatory.
"""
import hashlib,json,pathlib,re,types
from datetime import datetime,timezone

OWNER='42689c7a-eddd-44ad-b8bb-04bd0f3c70de'
REVIEWER='d9664b7d-13e2-464d-81e5-37de82992eb2'
JOB='6af14d46e6320c66fad7d183'
NAMESPACE='Autismo-Yah'
SOURCE='4619ba5838569a68b0f56bb46ab6b2b77c930586'
ORIGINAL='cadc7fa79eba914b2fc46fb444dc35f9cee5239e2a07dcd8dde480eb7aff35f8'
REQUEST='dc70c6973733afa4e1463b1acb1c8a5a10805e2f4408b8258cb83b4ee393dfb4'
CAPTURE=pathlib.Path('/tmp/zogre-hf-case/recovery-review-capture-v14.py')
CAPTURE_SHA='6bf3309b38275f4aa725888d307ef18b1a3fb77dcc4800fa90abea1b5f703475'
sha=lambda raw:hashlib.sha256(raw).hexdigest()

def strict_json(raw):
    def pairs(items):
        obj={}
        for key,value in items:
            if key in obj:raise ValueError('DUPLICATE_JSON_KEY')
            obj[key]=value
        return obj
    return json.loads(raw,object_pairs_hook=pairs)

def timestamp(value):
    if not isinstance(value,str):raise RuntimeError('INVALID_TIMESTAMP')
    x=datetime.fromisoformat(value.replace('Z','+00:00'))
    if x.tzinfo is None:raise RuntimeError('INVALID_TIMESTAMP')
    return x.astimezone(timezone.utc)

def expected_subject(selection_raw,selection_sha):
    if not isinstance(selection_raw,bytes) or sha(selection_raw)!=selection_sha:raise RuntimeError('SELECTION_PIN')
    s=strict_json(selection_raw)
    keys={'schema','action','owner','reviewer','collector','jobId','namespace','source','originalSelectionSHA256','requestSHA256','entrySHA256','launcherSHA256','bootstrapSHA256','authoritySHA256','captureSHA256','controllerSHA256','wrapperSHA256','sdkApiSHA256','sdkJobsSHA256','bindingSHA256','recordsSHA256','readbackSHA256','observerSHA256','payloadSHA256','planSHA256','inventorySHA256','reviewReceiptSHA256','selectedUTC'}
    if not isinstance(s,dict) or set(s)!=keys or type(s.get('schema')) is not int or s.get('schema')!=1 or s.get('action')!='COLLECT_EXISTING_ONCE' or s.get('owner')!=OWNER or s.get('reviewer')!=REVIEWER or s.get('collector')!=OWNER or s.get('jobId')!=JOB or s.get('namespace')!=NAMESPACE or s.get('source')!=SOURCE or s.get('originalSelectionSHA256')!=ORIGINAL or s.get('requestSHA256')!=REQUEST:raise RuntimeError('SELECTION_SUBJECT')
    fixed={'captureSHA256':CAPTURE_SHA,'controllerSHA256':'565e79e713f6881cca014107e3a7d9bf502fc77dc8e3ead152baffed5a53d487','wrapperSHA256':'fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755','sdkApiSHA256':'3871f6516f2bf97b77cf4179daea929b30b4c97d7e87e9158fa742f46cbf4860','sdkJobsSHA256':'4f6e280e0f1938d3230625384b25acdb360aac2925fe23040a8525c0bc1306e2','payloadSHA256':'220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4','planSHA256':'f8013c6ec9a9bc191db918dc8d93c2903cbb1991590887420b97a77cae2b3f9f'}
    if any(s.get(k)!=v for k,v in fixed.items()):raise RuntimeError('SELECTION_KNOWN_PINS')
    for name in ('entrySHA256','launcherSHA256','bootstrapSHA256','authoritySHA256','captureSHA256','controllerSHA256','wrapperSHA256','sdkApiSHA256','sdkJobsSHA256','bindingSHA256','recordsSHA256','readbackSHA256','observerSHA256','payloadSHA256','planSHA256','inventorySHA256','reviewReceiptSHA256'):
        if not isinstance(s.get(name),str) or not re.fullmatch('[0-9a-f]{64}',s[name]):raise RuntimeError('SELECTION_HASH_SCHEMA')
    timestamp(s['selectedUTC'])
    subject={'case':'ZOGRE_EXISTING_JOB_COLLECTION_ONLY','jobId':JOB,'namespace':NAMESPACE,'source':SOURCE,
             'originalSelectionSHA256':ORIGINAL,'requestSHA256':REQUEST,'recoverySelectionSHA256':selection_sha,
             'entrySHA256':s['entrySHA256'],'launcherSHA256':s['launcherSHA256'],'bootstrapSHA256':s['bootstrapSHA256']}
    return s,subject

def validate_preselection_receipt(raw,selection):
    """Acyclic author-prepared receipt; named reviewer attests only later.

    The receipt intentionally contains neither selection digest nor marker.
    It binds immutable proposed inputs and can therefore be hashed first.
    """
    if not isinstance(raw,bytes) or sha(raw)!=selection['reviewReceiptSHA256']:raise RuntimeError('RECEIPT_PIN')
    receipt=strict_json(raw)
    copied={'reviewer','executor','jobId','source','originalSelectionSHA256','requestSHA256','entrySHA256','launcherSHA256','bootstrapSHA256','authoritySHA256','captureSHA256','controllerSHA256','wrapperSHA256','sdkApiSHA256','sdkJobsSHA256','bindingSHA256','recordsSHA256','readbackSHA256','observerSHA256','payloadSHA256','planSHA256','inventorySHA256'}
    if not isinstance(receipt,dict) or set(receipt)!=copied|{'schema','kind','receiptNonce'} or type(receipt.get('schema')) is not int or receipt.get('schema')!=1 or receipt.get('kind')!='ZOGRE_PRESELECTION_SOURCE_RECEIPT':raise RuntimeError('RECEIPT_SCHEMA')
    if receipt.get('reviewer')!=REVIEWER or receipt.get('executor')!=OWNER:raise RuntimeError('RECEIPT_IDENTITY')
    if not isinstance(receipt.get('receiptNonce'),str) or not re.fullmatch('[0-9a-f]{32}',receipt['receiptNonce']):raise RuntimeError('RECEIPT_NONCE')
    for name in copied-{'reviewer','executor'}:
        if receipt.get(name)!=selection.get(name):raise RuntimeError('RECEIPT_SUBJECT_MISMATCH')
    return receipt

ARTIFACT_KEYS=('entrySHA256','launcherSHA256','bootstrapSHA256','authoritySHA256','captureSHA256','controllerSHA256',
               'wrapperSHA256','sdkApiSHA256','sdkJobsSHA256','bindingSHA256','recordsSHA256',
               'readbackSHA256','observerSHA256','payloadSHA256','planSHA256','inventorySHA256')

def verify_artifact_snapshots(selection,artifacts):
    """Check supplied bytes only; caller must attest actual executed snapshots."""
    if not isinstance(artifacts,dict) or set(artifacts)!=set(ARTIFACT_KEYS):raise RuntimeError('ARTIFACT_SET')
    total=0
    for name in ARTIFACT_KEYS:
        raw=artifacts[name]
        if not isinstance(raw,bytes) or len(raw)>32*1024*1024:raise RuntimeError('ARTIFACT_BOUND')
        total+=len(raw)
        if total>64*1024*1024 or sha(raw)!=selection[name]:raise RuntimeError('ARTIFACT_PIN')
    return 'SNAPSHOT_BYTES_MATCH_EXECUTION_UNPROVEN'

def review_window(raw,selected,selected_turn,now):
    """Strict prospective timestamp window; does not prove server pagination."""
    doc=strict_json(raw);turns=doc.get('turns')
    if not isinstance(turns,list) or not 1<=len(turns)<=1000:raise RuntimeError('WINDOW_INVALID')
    times=[]
    for turn in turns:
        if not isinstance(turn,dict):raise RuntimeError('WINDOW_INVALID')
        times.append(timestamp(turn.get('ts')))
    if times!=sorted(times) or times[0]>selected:raise RuntimeError('WINDOW_INCOMPLETE')
    matches=[i for i,turn in enumerate(turns) if turn==selected_turn]
    if len(matches)!=1:raise RuntimeError('MARKER_AMBIGUOUS')
    marker_index=matches[0];marker_time=times[marker_index]
    if selected>marker_time or marker_time>now or (now-selected).total_seconds()>900:raise RuntimeError('MARKER_STALE')
    # Authenticated turn order, not strict timestamp inequality: later turns
    # sharing the marker timestamp must also be checked. This is a conservative
    # explicit sentinel check, not complete semantic interpretation.
    for turn in turns[marker_index+1:]:
        text=turn.get('text') or ''
        tool=turn.get('tool') or {}
        if not isinstance(text,str) or not isinstance(tool,dict):raise RuntimeError('WINDOW_INVALID')
        if 'UGLUG_RECOVERY_REVOKE' in (text+' '+str(tool.get('name') or '')):raise RuntimeError('REVIEW_REVOKED')
    return {'markerUTC':selected_turn['ts'],'windowFirstUTC':turns[0]['ts'],'turns':len(turns)}

def validate_named_origin(selection_raw,selection_sha,receipt_raw,retained_raw,retained_sha,retained_turn,artifacts,fresh_attempt_id):
    """Snapshot-exec reviewed helper then independently re-acquire named session.

    The caller must pin this executed snapshot and original immutable selection,
    commit both attempt IDs before calls, verify receipt/source artifact hashes,
    and separately check other-session instructions and independent closure.
    This return is never a provider-action grant.
    """
    selection,subject=expected_subject(selection_raw,selection_sha)
    if globals().get('__verified_authority_sha256__')!=selection['authoritySHA256']:raise RuntimeError('EXECUTED_AUTHORITY_SNAPSHOT_UNBOUND')
    validate_preselection_receipt(receipt_raw,selection)
    verify_artifact_snapshots(selection,artifacts)
    if not isinstance(retained_raw,bytes) or sha(retained_raw)!=retained_sha:raise RuntimeError('RETAINED_PIN')
    raw=artifacts['captureSHA256']
    if sha(raw)!=CAPTURE_SHA:raise RuntimeError('CAPTURE_PIN')
    adapter=types.ModuleType('verified_named_capture');exec(compile(raw,str(CAPTURE),'exec'),adapter.__dict__)
    if tuple(adapter.COMMAND)!=(adapter.CLI,'--json','sessions','read',REVIEWER,'--turns','1000'):raise RuntimeError('COMMAND_PIN')
    kept=adapter.select_turn(retained_raw,subject,selection['reviewReceiptSHA256'])
    if kept!=retained_turn:raise RuntimeError('RETAINED_TURN_MISMATCH')
    # Production path deliberately uses pinned helper's actual named-session
    # acquire, never an injected transcript or synthetic runner.
    fresh_raw,fresh_turn,metadata=adapter.acquire(subject,selection['reviewReceiptSHA256'],fresh_attempt_id)
    if fresh_turn!=kept:raise RuntimeError('FRESH_TURN_MISMATCH')
    now=datetime.now(timezone.utc);window=review_window(fresh_raw,timestamp(selection['selectedUTC']),fresh_turn,now)
    return {'stage':'NAMED_TURN_MATCH_PENDING_OUTER_AND_INSTRUCTION_REVIEW','reviewer':REVIEWER,
            'selectionSHA256':selection_sha,'receiptSHA256':selection['reviewReceiptSHA256'],
            'retainedSHA256':retained_sha,'freshSHA256':sha(fresh_raw),'window':window,
            'captureMetadata':metadata,'scope':'origin match only; pagination, custody, outer closure and other instructions remain unproved'}
