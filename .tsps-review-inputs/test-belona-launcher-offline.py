"""Source-derived gate controls using real pinned source and a synthetic adapter."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('live',root/'belona-live-boundaries.py');live=importlib.util.module_from_spec(spec);spec.loader.exec_module(live)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
paths={'prestart':root/'belona-prestart.py','sourceGate':root/'belona-source-gate.py','caller':root/'belona-caller.py',
       'held':root/'belona-before-held-plan.json','manifest':root/'belona-before-manifest.json',
       'payload':root/'belona-native-payload.cjs','profile':root/'belona-source-profile.json','inventory':root/'belona-cache-inventory.json', 'entry':root/'belona-live-entry.py', 'helper':Path(os.environ['BELONA_CHILD_HELPER']), 'adapter':Path(os.environ['BELONA_AUTH_ADAPTER']), 'python':Path(os.environ['BELONA_PYTHON']), 'sdkRoot':Path(os.environ['BELONA_SDK_ROOT']), 'sdkManifest':Path(os.environ['BELONA_SDK_MANIFEST']), 'cli':Path(os.environ['BELONA_CLI']), 'cliModule':Path(os.environ['BELONA_CLI_MODULE']), 'node':Path(os.environ['BELONA_NODE'])}
subject={k:'a'*64 for k in ('pilotSHA256','adapterSHA256','childSHA256','classifierSHA256','liveSHA256','sdkPythonSHA256','sdkManifestSHA256','cliSHA256','requestSHA256')}
subject.update(owner='49676823-be5e-408f-be58-047e80dadbb2',reviewer='ed3a4125-645c-4266-8b86-4e199dd5bc63',model='gpt-6.1-terra',reasoningEffort='medium',sdkVersion='2.4.0',source='247ed387d6a631a00440904a00aa8e1a2e44a962',selectedPlanSHA256='6d739505fa4ca4aa04fb3e273b47b40814256edd4af4b14d176e3d15c30a2b16')
for key,path in [('prestartSHA256','prestart'),('sourceGateSHA256','sourceGate'),('callerSHA256','caller'),('heldPlanSHA256','held'),('manifestSHA256','manifest'),('payloadSHA256','payload'),('sourceProfileSHA256','profile'),('cacheInventorySHA256','inventory'),('entrySHA256','entry'),('childSHA256','helper'),('adapterSHA256','adapter'),('sdkPythonSHA256','python'),('sdkManifestSHA256','sdkManifest'),('cliSHA256','cli'),('cliModuleSHA256','cliModule'),('nodeSHA256','node')]:subject[key]=sha(paths[path])
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
paths['selected'].write_bytes(selected)
# Fresh child rechecks a synthetic private SDK and synthetic CLI before fake auth.
with tempfile.TemporaryDirectory() as t:
 from unittest.mock import patch
 td=Path(t); cli=td/'cuttle';node=td/'node';module=td/'cli.mjs';module.write_text('module');node.write_text('node');node.chmod(0o755)
 cli.write_text('#!/bin/sh\nexec node '+str(module.resolve())+' "$@"\n');cli.chmod(0o755)
 sdk=td/'sdk';metadata=sdk/'huggingface_hub-2.4.0.dist-info'/'METADATA';metadata.parent.mkdir(parents=True);metadata.write_text('Metadata-Version: 2.1\nName: huggingface_hub\nVersion: 2.4.0\n')
 manifest=td/'sdk.json';manifest.write_text(json.dumps({'root':str(sdk),'version':'2.4.0','files':{metadata.relative_to(sdk).as_posix():sha(metadata)}}))
 p2=dict(paths,cli=cli,node=node,cliModule=module,sdkRoot=sdk,sdkManifest=manifest)
 s2=dict(subject,cliSHA256=sha(cli),nodeSHA256=sha(node),cliModuleSHA256=sha(module),sdkManifestSHA256=sha(manifest))
 with patch.dict(os.environ,{'PATH':str(td)+os.pathsep+os.environ.get('PATH','')}):
  command=launcher.complete_child_command(fake,s2,p2)
  good=subprocess.run(command,capture_output=True,text=True);assert good.returncode==0 and good.stdout.strip()=='FULL_FAKE_AUTH',good.stderr
  original=metadata.read_bytes();metadata.write_bytes(b'changed')
  bad=subprocess.run(command,capture_output=True,text=True);assert bad.returncode!=0 and 'FULL_FAKE_AUTH' not in bad.stdout
  metadata.write_bytes(original);original=node.read_bytes();node.write_bytes(b'changed')
  bad=subprocess.run(command,capture_output=True,text=True);assert bad.returncode!=0 and 'FULL_FAKE_AUTH' not in bad.stdout
  node.write_bytes(original);original=module.read_bytes();module.write_bytes(b'changed')
  bad=subprocess.run(command,capture_output=True,text=True);assert bad.returncode!=0 and 'FULL_FAKE_AUTH' not in bad.stdout
  module.write_bytes(original)
  wrong=td/'wrong';wrong.mkdir();(wrong/'node').write_text('bad');(wrong/'node').chmod(0o755)
  with patch.dict(os.environ,{'PATH':str(wrong)+os.pathsep+str(td)+os.pathsep+os.environ.get('PATH','')}):
   bad=subprocess.run(command,capture_output=True,text=True);assert bad.returncode!=0 and 'FULL_FAKE_AUTH' not in bad.stdout
selection_temp.cleanup()
print('complete child source gate checks plan, SDK and CLI identities before fake auth')
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
