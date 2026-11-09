"""Held nonsecret pre-auth source gate. No real adapter is called by this file."""
import hashlib
import json
import subprocess
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
    selected_path,selected=snapshot(paths['selected'],subject['selectedPlanSHA256'])
    _,expected_selected=caller.prepare_selected(held)
    if selected != expected_selected: raise ValueError('actual selected bytes differ from held transform')
    _,manifest=snapshot(paths['manifest'],subject['manifestSHA256'])
    _,payload=snapshot(paths['payload'],subject['payloadSHA256'])
    _,profile_raw=snapshot(paths['profile'],subject['sourceProfileSHA256'])
    _,inventory_raw=snapshot(paths['inventory'],subject['cacheInventorySHA256'])
    result=gate.guard_before_adapter(held,manifest,payload,json.loads(profile_raw),inventory_raw,caller,lambda:None)
    snapshot(paths['entry'],subject['entrySHA256']);snapshot(paths['adapter'],subject['adapterSHA256']);snapshot(paths['python'],subject['sdkPythonSHA256'])
    helper=module(paths['helper'],subject['childSHA256'],'belona_gate_helper')
    command=helper.authenticated_pilot_command(paths['python'],paths['adapter'],subject['adapterSHA256'],paths['entry'],subject['entrySHA256'],selected_path,paths['sdkRoot'])
    if command[-2:] != ['--plan',str(selected_path)]:raise ValueError('selected plan handoff mismatch')
    return {'sourceProfile':result,'selectedPath':str(selected_path),'selectedSHA256':sha(selected),'command':command}

def launch(subject,paths,run_child=subprocess.run):
    checked=verify_before_auth(subject,paths)
    if HARD_RUNTIME_HOLD:raise RuntimeError('HARD_RUNTIME_HOLD: adapter not invoked')
    snapshot(checked['selectedPath'],checked['selectedSHA256'])
    return checked,run_child(checked['command'],check=True,timeout=300)
