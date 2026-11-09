"""Source-derived gate controls using real pinned source and a synthetic adapter."""
import hashlib
import importlib.util
import os
from pathlib import Path
import tempfile
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('live',root/'belona-live-boundaries.py');live=importlib.util.module_from_spec(spec);spec.loader.exec_module(live)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
paths={'prestart':root/'belona-prestart.py','sourceGate':root/'belona-source-gate.py','caller':root/'belona-caller.py',
       'held':root/'belona-before-held-plan.json','manifest':root/'belona-before-manifest.json',
       'payload':root/'belona-native-payload.cjs','profile':root/'belona-source-profile.json','inventory':root/'belona-cache-inventory.json', 'entry':root/'belona-live-entry.py', 'helper':Path(os.environ['BELONA_CHILD_HELPER']), 'adapter':Path(os.environ['BELONA_AUTH_ADAPTER']), 'python':Path(os.environ['BELONA_PYTHON']), 'sdkRoot':Path(os.environ['BELONA_SDK_ROOT'])}
subject={k:'a'*64 for k in ('pilotSHA256','adapterSHA256','childSHA256','classifierSHA256','liveSHA256','sdkPythonSHA256','sdkManifestSHA256','cliSHA256','requestSHA256')}
subject.update(owner='49676823-be5e-408f-be58-047e80dadbb2',reviewer='ed3a4125-645c-4266-8b86-4e199dd5bc63',model='gpt-6.1-terra',reasoningEffort='medium',sdkVersion='2.4.0',source='247ed387d6a631a00440904a00aa8e1a2e44a962',selectedPlanSHA256='6d739505fa4ca4aa04fb3e273b47b40814256edd4af4b14d176e3d15c30a2b16')
for key,path in [('prestartSHA256','prestart'),('sourceGateSHA256','sourceGate'),('callerSHA256','caller'),('heldPlanSHA256','held'),('manifestSHA256','manifest'),('payloadSHA256','payload'),('sourceProfileSHA256','profile'),('cacheInventorySHA256','inventory'),('entrySHA256','entry'),('childSHA256','helper'),('adapterSHA256','adapter'),('sdkPythonSHA256','python')]:subject[key]=sha(paths[path])
launcher_path=root/'belona-launcher.py';subject['launcherSHA256']=sha(launcher_path)
launcher=live.module_snapshot(launcher_path,subject['launcherSHA256'],'launcher_test')
caller=live.module_snapshot(paths['caller'],subject['callerSHA256'],'launcher_caller_test')
_,selected=caller.prepare_selected(paths['held'].read_bytes())
selection_temp=tempfile.TemporaryDirectory(); paths['selected']=Path(selection_temp.name)/'synthetic-selected.json';paths['selected'].write_bytes(selected)
calls=[]
try:launcher.launch(subject,paths,lambda *args,**kwargs:calls.append('adapter'))
except RuntimeError as e:assert 'HARD_RUNTIME_HOLD' in str(e)
else:raise AssertionError('hold absent')
assert calls==[]
launcher.HARD_RUNTIME_HOLD=False # synthetic module instance; no real adapter
_,_=launcher.launch(subject,paths,lambda *args,**kwargs:calls.append(args[0]))
assert len(calls)==1 and calls[0][1:3]==['-B','-c'] and calls[0][-2:]==['--plan',str(paths['selected'].resolve())]
paths['selected'].write_bytes(selected+b' ');calls.clear()
try:launcher.launch(subject,paths,lambda *args,**kwargs:calls.append('adapter'))
except ValueError:pass
else:raise AssertionError('changed selected accepted')
assert not calls
paths['selected'].write_bytes(selected)
with tempfile.TemporaryDirectory() as t:
 changed=Path(t)/'profile';changed.write_bytes(paths['profile'].read_bytes().replace(b'osrs-241',b'osrs-999'))
 altered=dict(paths);altered['profile']=changed;bad=dict(subject);bad['sourceProfileSHA256']=sha(changed);calls.clear()
 try:launcher.launch(bad,altered,lambda *args,**kwargs:calls.append('adapter'))
 except ValueError:pass
 else:raise AssertionError('mismatch accepted')
 assert not calls
import subprocess
import sys
fake=[sys.executable,'-c','import builtins; print("FULL_FAKE_AUTH" if hasattr(builtins,"__BELONA_PREAUTH_EVIDENCE__") else "MISSING")']
full=launcher.complete_child_command(fake,subject,paths)
run=subprocess.run(full,capture_output=True,text=True)
assert run.returncode==0 and run.stdout.strip()=='FULL_FAKE_AUTH',run.stderr
paths['selected'].write_bytes(selected+b' ')
run=subprocess.run(full,capture_output=True,text=True)
assert run.returncode!=0 and 'FULL_FAKE_AUTH' not in run.stdout
selection_temp.cleanup()
print('complete child source gate reaches fake auth only for pinned selected plan')
print('source gate and actual selected bytes checked before fake adapter; mismatches call zero; real hard hold prevented adapter')
# Child-side plan guard exercises a fake bootstrap, never the authentication adapter.
import subprocess
import sys
with tempfile.TemporaryDirectory() as tmp:
    plan=Path(tmp)/'selected.json';plan.write_bytes(b'synthetic-plan')
    expected=sha(plan); fake=[sys.executable,'-c','print("FAKE_ADAPTER_REACHED")']
    command=launcher.guarded_child_command(fake,plan,expected)
    assert command[1:3]==['-B','-c']
    good=subprocess.run(command,capture_output=True,text=True)
    assert good.returncode==0 and good.stdout.strip()=='FAKE_ADAPTER_REACHED'
    plan.write_bytes(b'mutated')
    bad=subprocess.run(command,capture_output=True,text=True)
    assert bad.returncode!=0 and 'SELECTED_PLAN_CHANGED_BEFORE_AUTH' in bad.stderr
    assert 'FAKE_ADAPTER_REACHED' not in bad.stdout
print('child plan mutation refused before synthetic adapter bootstrap')
