"""Held nonsecret pre-auth source gate. No real adapter is called by this file."""
import hashlib
import json
from pathlib import Path
import types

HARD_RUNTIME_HOLD = True

def sha(raw):return hashlib.sha256(raw).hexdigest()
def snapshot(path,digest):
    path=Path(path).resolve(strict=True);raw=path.read_bytes()
    if sha(raw)!=digest:raise ValueError('source snapshot mismatch')
    return path,raw

def module(path,digest,name):
    path,raw=snapshot(path,digest);result=types.ModuleType(name);result.__file__=str(path)
    result.__dict__['__BELONA_SNAPSHOT_SHA__']=sha(raw)
    exec(compile(raw,str(path),'exec'),result.__dict__)
    return result

def verify_before_auth(subject,paths):
    """Called outside the authenticated adapter; checks source before any adapter work."""
    if globals().get('__BELONA_SNAPSHOT_SHA__') != subject.get('launcherSHA256'):
        raise ValueError('launcher snapshot provenance missing')
    snapshot(__file__,subject['launcherSHA256'])
    prestart=module(paths['prestart'],subject['prestartSHA256'],'belona_gate_prestart')
    prestart.canonical_subject(subject)
    gate=module(paths['sourceGate'],subject['sourceGateSHA256'],'belona_gate_source')
    caller=module(paths['caller'],subject['callerSHA256'],'belona_gate_caller')
    _,held=snapshot(paths['held'],subject['heldPlanSHA256'])
    _,manifest=snapshot(paths['manifest'],subject['manifestSHA256'])
    _,payload=snapshot(paths['payload'],subject['payloadSHA256'])
    _,profile_raw=snapshot(paths['profile'],subject['sourceProfileSHA256'])
    return gate.guard_before_adapter(held,manifest,payload,json.loads(profile_raw),caller,lambda:None)

def launch(subject,paths,adapter):
    checked=verify_before_auth(subject,paths)
    if HARD_RUNTIME_HOLD:raise RuntimeError('HARD_RUNTIME_HOLD: adapter not invoked')
    return checked,adapter()
