"""Source-derived gate controls using real pinned source and a synthetic adapter."""
import hashlib
import importlib.util
from pathlib import Path
import tempfile
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('live',root/'belona-live-boundaries.py');live=importlib.util.module_from_spec(spec);spec.loader.exec_module(live)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
paths={'prestart':root/'belona-prestart.py','sourceGate':root/'belona-source-gate.py','caller':root/'belona-caller.py',
       'held':root/'belona-before-held-plan.json','manifest':root/'belona-before-manifest.json',
       'payload':root/'belona-native-payload.cjs','profile':root/'belona-source-profile.json'}
subject={k:'a'*64 for k in ('pilotSHA256','adapterSHA256','childSHA256','classifierSHA256','liveSHA256','sdkPythonSHA256','sdkManifestSHA256','cliSHA256','requestSHA256')}
subject.update(owner='49676823-be5e-408f-be58-047e80dadbb2',reviewer='ed3a4125-645c-4266-8b86-4e199dd5bc63',model='gpt-6.1-terra',reasoningEffort='medium',sdkVersion='2.4.0',source='247ed387d6a631a00440904a00aa8e1a2e44a962',selectedPlanSHA256='6d739505fa4ca4aa04fb3e273b47b40814256edd4af4b14d176e3d15c30a2b16')
for key,path in [('prestartSHA256','prestart'),('sourceGateSHA256','sourceGate'),('callerSHA256','caller'),('heldPlanSHA256','held'),('manifestSHA256','manifest'),('payloadSHA256','payload'),('sourceProfileSHA256','profile')]:subject[key]=sha(paths[path])
launcher_path=root/'belona-launcher.py';subject['launcherSHA256']=sha(launcher_path)
launcher=live.module_snapshot(launcher_path,subject['launcherSHA256'],'launcher_test')
calls=[]
try:launcher.launch(subject,paths,lambda:calls.append('adapter'))
except RuntimeError as e:assert 'HARD_RUNTIME_HOLD' in str(e)
else:raise AssertionError('hold absent')
assert calls==[]
launcher.HARD_RUNTIME_HOLD=False # synthetic module instance; no real adapter
_,_=launcher.launch(subject,paths,lambda:calls.append('fake-adapter'))
assert calls==['fake-adapter']
with tempfile.TemporaryDirectory() as t:
 changed=Path(t)/'profile';changed.write_bytes(paths['profile'].read_bytes().replace(b'osrs-241',b'osrs-999'))
 altered=dict(paths);altered['profile']=changed;bad=dict(subject);bad['sourceProfileSHA256']=sha(changed);calls.clear()
 try:launcher.launch(bad,altered,lambda:calls.append('adapter'))
 except ValueError:pass
 else:raise AssertionError('mismatch accepted')
 assert not calls
print('source gate runs before fake adapter; mismatch calls zero; real hard hold prevented adapter')
