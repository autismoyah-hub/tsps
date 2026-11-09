"""Belona admission primitives. Source-only draft: no provider submission or selection CLI."""
import hashlib
import json
import os
from pathlib import Path

OWNER = '49676823-be5e-408f-be58-047e80dadbb2'
REVIEWER = 'ed3a4125-645c-4266-8b86-4e199dd5bc63'
HOLD = 'HELD_NOT_SELECTED_CASE_PROFILE_REVIEW_REQUIRED'
IMAGE = 'node@sha256:b04ce4ae4e95b522112c2e5c52f781471a5cbc3b594527bcddedee9bc48c03a0'
PAYLOAD_SHA = '68debf7f7a7392c3ffd92cfa185a09dcff9ef0e00490ca86efb69709988c1378'
PILOT_SHA = '2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200'
AUTH_SHA = 'fab4bc92dc8601143ce61e910f8121774c22a0d19be6d2617bba9449f7131326'
CHILD_SHA = '00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'
CASES = {
 'before': {'held': 'e50f6ec1e341799fd48bd3e8d39897717f68d8b657afa9297e3b7a968118fabf', 'source':'247ed387d6a631a00440904a00aa8e1a2e44a962', 'manifest':'a44a97f57d259f84a8cc2ad3adc6ce9dbd65ec8013e7def9258702c78a2aa305','count':4912},
 'after': {'held': '7b141df4d1b9966654795c6fcff01efc11c0c153bbf7806519df1ca60560157e', 'source':'88b5e5793039c1f246c2a4096206664e50c51003', 'manifest':'f2797ca4c62cd14e4c039f766d17163d49f4a28ed1548e3af87616aafa4e7206','count':4913},
}

def sha(raw): return hashlib.sha256(raw).hexdigest()

def prepare_selected(held_raw):
    """Pure exact transformation, with no creation of an owner selection."""
    side = next((key for key, value in CASES.items() if sha(held_raw) == value['held']), None)
    if side is None: raise ValueError('held bytes not pinned')
    plan = json.loads(held_raw)
    expected = CASES[side]
    fields = {'source':expected['source'],'name':'belona-'+side+'-held','image':IMAGE,
              'flavor':'cpu-upgrade','timeout':1800,'attempts':1,'cacheId':2746,
              'cacheTarget':'osrs-241_2026-10-20','tests':['mineral-exchange.test.cjs'],
              'expectedTestCounts':{'mineral-exchange.test.cjs':3},
              'payloadFile':'native-payload-mapref-candidate.cjs','executionHold':HOLD}
    if any(plan.get(k) != v for k,v in fields.items()): raise ValueError('held field mismatch')
    ref = plan.get('expectedInputsRef', {})
    if ref.get('sha256') != expected['manifest'] or ref.get('count') != expected['count'] or ref.get('source') != expected['source']:
        raise ValueError('manifest mismatch')
    selected = dict(plan)
    del selected['executionHold']
    raw = (json.dumps(selected, ensure_ascii=True, indent=2)+'\n').encode()
    decoded = json.loads(raw)
    if decoded != selected or set(plan)-set(decoded) != {'executionHold'} or set(decoded)-set(plan) or any(decoded[k] != plan[k] for k in decoded):
        raise ValueError('selected transformation differs')
    return side, raw

def create_selected(path, raw):
    """Caller must obtain separate genuine choice and review before this is used."""
    path = Path(path)
    if not path.parent.is_dir(): raise ValueError('missing parent')
    with path.open('xb') as stream:
        stream.write(raw); stream.flush(); os.fsync(stream.fileno())
    fd = os.open(path.parent, os.O_RDONLY | getattr(os, 'O_DIRECTORY', 0))
    try: os.fsync(fd)
    finally: os.close(fd)
    if path.read_bytes() != raw: raise ValueError('selected readback mismatch')

SELECTED_SHA = {'before':'6d739505fa4ca4aa04fb3e273b47b40814256edd4af4b14d176e3d15c30a2b16',
                'after':'92dcba657ee0599f5e38c7927e1bcb1296301125f435f002f828bcc109b1ec00'}

def literal_request(selected_raw, payload_raw):
    if sha(selected_raw) not in SELECTED_SHA.values(): raise ValueError('selected bytes not pinned')
    if sha(payload_raw) != PAYLOAD_SHA: raise ValueError('payload hash mismatch')
    payload = payload_raw.decode('utf-8', errors='strict')
    if payload.count('__PLAN__') != 1: raise ValueError('payload placeholder mismatch')
    plan = json.loads(selected_raw)
    if plan.get('executionHold') is not None: raise ValueError('plan held')
    rendered = payload.replace('__PLAN__', json.dumps(plan))
    request = {key: plan[key] for key in ('name','image','flavor','timeout','attempts')}
    request.update(command=['node','-e',rendered], labels={'project':'tsps','source':plan['source'],'purpose':'overflow-qualification'})
    if any('\0' in part or len(part.encode())+1 >= 131072 for part in request['command']): raise ValueError('argv bound')
    return request, sha(rendered.encode())


def reject_duplicates(records, source, name):
    """Reject incomplete scans and any previous matching source OR name."""
    if not isinstance(records, list): raise ValueError('duplicate scan incomplete')
    for row in records:
        if not isinstance(row, dict) or not all(isinstance(row.get(k), str) and row[k] for k in ('source','name','id')):
            raise ValueError('incomplete job or intent record')
        if row['source'] == source or row['name'] == name:
            raise ValueError('matching source or name already exists')

def require_inspection(inspected, namespace, job_id, request):
    if not isinstance(inspected, dict) or inspected.get('id') != job_id or inspected.get('namespace') != namespace:
        raise ValueError('wrong job or namespace')
    for key in ('command','image','flavor','timeout','attempts','labels'):
        if inspected.get(key) != request[key]: raise ValueError('inspected request mismatch: '+key)
    status = inspected.get('status')
    if not isinstance(status, dict) or status.get('stage') not in ('COMPLETED','ERROR','CANCELED','CANCELLED'):
        raise ValueError('job not terminal')

def retain_log_prefix(path, iterator):
    """An exception is propagated only after preserving and syncing the received prefix."""
    with Path(path).open('xb') as out:
        try:
            for line in iterator:
                if not isinstance(line, str): raise ValueError('unexpected log type')
                out.write(line.encode('utf-8')+b'\n'); out.flush(); os.fsync(out.fileno())
        finally:
            out.flush(); os.fsync(out.fileno())

def admission_sequence(prestart, duplicate_scan, write_intent, submit_once, source, name):
    """Abstract ordering primitive; live authenticated identity/review binding is not implemented."""
    prestart()
    reject_duplicates(duplicate_scan(), source, name)
    prestart()
    write_intent()
    return submit_once()

def validate_saved_request(selected_raw, payload_raw, saved_request):
    request, digest = literal_request(selected_raw, payload_raw)
    if saved_request != request: raise ValueError('saved request differs from literal request')
    return request, digest

def guarded_collect(selected_raw, payload_raw, saved_request, inspected, namespace, job_id, write_inspection, fetch_logs, raw_path):
    """Record inspection first; mismatch refuses before the log iterator is opened."""
    request, digest = validate_saved_request(selected_raw, payload_raw, saved_request)
    write_inspection(inspected)
    require_inspection(inspected, namespace, job_id, request)
    retain_log_prefix(raw_path, fetch_logs())
    return digest

def claim_intent(path, intent):
    """Create exactly once; an existing/ambiguous intent must be reconciled, never retried."""
    path = Path(path)
    raw = (json.dumps(intent, sort_keys=True, separators=(',',':'))+'\n').encode()
    with path.open('xb') as out:
        out.write(raw); out.flush(); os.fsync(out.fileno())
    fd = os.open(path.parent, os.O_RDONLY | getattr(os, 'O_DIRECTORY', 0))
    try: os.fsync(fd)
    finally: os.close(fd)
    if path.read_bytes() != raw: raise ValueError('intent readback mismatch')
