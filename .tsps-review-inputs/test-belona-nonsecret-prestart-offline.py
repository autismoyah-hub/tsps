"""Exercise actual nonsecret_prestart with synthetic SDK/session and real source gate."""
import hashlib,importlib.util,json,os,shutil,subprocess,sys,tempfile
from pathlib import Path
from unittest.mock import patch
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('prestart',root/'belona-prestart.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
with tempfile.TemporaryDirectory() as tmp:
 d=Path(tmp);sdk=d/'sdk';metadata=sdk/'huggingface_hub-2.4.0.dist-info'/'METADATA';metadata.parent.mkdir(parents=True);metadata.write_text('Metadata-Version: 2.1\nName: huggingface_hub\nVersion: 2.4.0\n')
 sdkdoc={'root':str(sdk),'version':'2.4.0','files':{str(metadata.relative_to(sdk)):sha(metadata)}};sdkmanifest=d/'sdk.json';sdkmanifest.write_text(json.dumps(sdkdoc))
 cli=d/'fake-cli';cli.write_text('synthetic only');prefdir=d/'pref';prefdir.mkdir();preflight=prefdir/'preflight.py';shutil.copy(root/'belona-preflight.py',preflight)
 for side in ('before','after'):
  shutil.copy(root/('belona-'+side+'-held-plan.json'),prefdir/(side+'-held-plan.json'));shutil.copy(root/('belona-'+side+'-manifest.json'),prefdir/(side+'-manifest.json'))
 files={'payloadSHA256':root/'belona-native-payload.cjs','pilotSHA256':d/'dummy-pilot','adapterSHA256':d/'dummy-adapter','childSHA256':d/'dummy-child','classifierSHA256':root/'belona-classify-tap.py','liveSHA256':root/'belona-live-boundaries.py','entrySHA256':root/'belona-live-entry.py','launcherSHA256':root/'belona-launcher.py','sourceGateSHA256':root/'belona-source-gate.py','sourceProfileSHA256':root/'belona-source-profile.json','cacheInventorySHA256':root/'belona-cache-inventory.json','sdkPythonSHA256':Path(sys.executable),'prestartSHA256':root/'belona-prestart.py'}
 for k in ('pilotSHA256','adapterSHA256','childSHA256'):(files[k]).write_text('synthetic '+k)
 subject={key:sha(value) for key,value in files.items()};subject.update(owner=m.OWNER,reviewer=m.REVIEWER,model='gpt-6.1-terra',reasoningEffort='medium',sdkVersion='2.4.0',source='247ed387d6a631a00440904a00aa8e1a2e44a962',heldPlanSHA256='e50f6ec1e341799fd48bd3e8d39897717f68d8b657afa9297e3b7a968118fabf',selectedPlanSHA256='6d739505fa4ca4aa04fb3e273b47b40814256edd4af4b14d176e3d15c30a2b16',manifestSHA256='a44a97f57d259f84a8cc2ad3adc6ce9dbd65ec8013e7def9258702c78a2aa305',callerSHA256=sha(root/'belona-caller.py'),sdkManifestSHA256=sha(sdkmanifest),cliSHA256=sha(cli),requestSHA256='a'*64)
 digest=m.sha(m.canonical_subject(subject));turn={'role':'assistant','kind':'message','text':'BELONA_PROFILE_GO '+digest,'ts':'synthetic'}
 review={'type':'DIRECT_SESSION_CAPTURE','reviewer':m.REVIEWER,'owner':m.OWNER,'subject':subject,'subjectSHA256':digest,'turn':turn,'rawTranscriptSHA256':'b'*64,'cliSHA256':sha(cli)};review_raw=json.dumps(review).encode()
 original_run=subprocess.run;original_check=subprocess.check_output;seen=[]
 def run(args,*a,**kw):
  if args[0]==str(cli.resolve()):return subprocess.CompletedProcess(args,0,json.dumps({'turns':[turn]}).encode(),b'')
  return original_run(args,*a,**kw)
 def check(args,*a,**kw):
  if args[0]==str(cli.resolve()):return json.dumps({'id':m.OWNER,'projectId':m.PROJECT,'model':'gpt-6.1-terra','status':'running'}).encode()
  if args[:2]==['git','show']:seen.append(args[2])
  return original_check(args,*a,**kw)
 with patch.dict(os.environ,{'CCC_SESSION_ID':m.OWNER}),patch.object(m,'effective_config',return_value={'model':'gpt-6.1-terra','reasoningEffort':'medium','configSHA256':'c'*64}),patch.object(subprocess,'run',side_effect=run),patch.object(subprocess,'check_output',side_effect=check):
  out=m.nonsecret_prestart(subject,review_raw,cli,sha(cli),preflight,root/'belona-caller.py',files,sdkmanifest,root/'belona-before-held-plan.json',root/'belona-before-manifest.json')
 assert out['stage']=='NONSECRET_PRESTART_ONLY' and any(x.endswith(':server/target.txt') for x in seen) and any(x.endswith(':server/scripts/ensure-cache.ts') for x in seen)
print('actual nonsecret_prestart passed synthetic SDK/session with real git source gate and source preflight; no provider or selection')
