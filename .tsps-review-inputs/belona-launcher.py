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

CHILD_PLAN_GUARD = """import hashlib,sys
from pathlib import Path
plan_path=sys.argv.pop(1)
expected=sys.argv.pop(1)
bootstrap=sys.argv.pop(1)
if hashlib.sha256(Path(plan_path).read_bytes()).hexdigest()!=expected:raise SystemExit('SELECTED_PLAN_CHANGED_BEFORE_AUTH')
exec(compile(bootstrap,'<verified-auth-bootstrap>','exec'))
"""

def guarded_child_command(command,selected_path,selected_sha):
    if len(command)<3 or command[1]!='-c':raise ValueError('unexpected authenticated child command')
    if not isinstance(selected_sha,str) or len(selected_sha)!=64:raise ValueError('selected hash malformed')
    guarded=[command[0],'-B','-c',CHILD_PLAN_GUARD,str(selected_path),selected_sha,*command[2:]]
    if any('\0' in arg or len(arg.encode())+1>=131072 for arg in guarded):raise ValueError('child argv bound')
    return guarded

CHILD_FULL_SOURCE_GUARD = """import builtins,hashlib,json,sys,types
from pathlib import Path
launcher_path=sys.argv.pop(1)
launcher_sha=sys.argv.pop(1)
subject=json.loads(sys.argv.pop(1))
paths=json.loads(sys.argv.pop(1))
bootstrap=sys.argv.pop(1)
raw=Path(launcher_path).read_bytes()
if hashlib.sha256(raw).hexdigest()!=launcher_sha or subject.get('launcherSHA256')!=launcher_sha:raise SystemExit('LAUNCHER_CHANGED_BEFORE_AUTH')
module=types.ModuleType('belona_child_verified_launcher');module.__file__=launcher_path
module.__dict__['__BELONA_SNAPSHOT_SHA__']=launcher_sha
exec(compile(raw,launcher_path,'exec'),module.__dict__)
checked=module.verify_before_auth(subject,paths)
if checked['selectedSHA256']!=subject['selectedPlanSHA256']:raise SystemExit('PLAN_CHANGED_BEFORE_AUTH')
subject_raw=json.dumps(subject,sort_keys=True,separators=(',',':')).encode()
builtins.__BELONA_PREAUTH_EVIDENCE__={'subjectSHA256':hashlib.sha256(subject_raw).hexdigest(),'selectedPlanSHA256':checked['selectedSHA256']}
exec(compile(bootstrap,'<verified-auth-bootstrap>','exec'))
"""

def complete_child_command(command,subject,paths):
    if len(command)<3 or command[1]!='-c':raise ValueError('unexpected authenticated child command')
    launcher_path=str(Path(__file__).resolve())
    path_strings={key:str(value) for key,value in paths.items()}
    guarded=[command[0],'-B','-c',CHILD_FULL_SOURCE_GUARD,launcher_path,subject['launcherSHA256'],
             json.dumps(subject,sort_keys=True,separators=(',',':')),
             json.dumps(path_strings,sort_keys=True,separators=(',',':')),*command[2:]]
    if any('\0' in arg or len(arg.encode())+1>=131072 for arg in guarded):raise ValueError('child argv bound')
    return guarded

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
    command=complete_child_command(command,subject,paths)
    return {'sourceProfile':result,'selectedPath':str(selected_path),'selectedSHA256':sha(selected),'command':command}

def launch(subject,paths,run_child=subprocess.run):
    checked=verify_before_auth(subject,paths)
    if HARD_RUNTIME_HOLD:raise RuntimeError('HARD_RUNTIME_HOLD: adapter not invoked')
    snapshot(checked['selectedPath'],checked['selectedSHA256'])
    return checked,run_child(checked['command'],check=True,timeout=300)
