"""Nonsecret preselection evidence only; does not contact HF or make a choice."""
import datetime,hashlib,importlib.util,json,os,subprocess,sys
from pathlib import Path
root=Path(__file__).resolve().parent
expected_subject='312289635ed66688f2077936e367ef863c3396bc7290a4dbee38d9f554dfedf3'
pre_file=root/'belona-prestart.py'
spec=importlib.util.spec_from_file_location('belona_pre',pre_file);pre=importlib.util.module_from_spec(spec);spec.loader.exec_module(pre)
subject=json.loads((root/'before-subject-v2-draft.json').read_bytes())
if hashlib.sha256(pre.canonical_subject(subject)).hexdigest()!=expected_subject:raise ValueError('subject mismatch')
if os.environ.get('CCC_SESSION_ID')!=pre.OWNER:raise ValueError('wrong owner')
files={'callerSHA256':root/'belona-caller.py','prestartSHA256':pre_file,'payloadSHA256':root/'native-payload-mapref-candidate.cjs',
       'pilotSHA256':Path('/workspace/belona-tooling/cache/hf-accepted-collectors/001/files/pilot.py'),
       'adapterSHA256':Path('/workspace/belona-tooling/cache/hf-author-control/004/files/hf-project-access-v4.py'),
       'childSHA256':Path('/workspace/belona-tooling/cache/hf-author-control/004/files/authenticated-child-snapshot-command.py'),
       'classifierSHA256':root/'classify-tap.py','liveSHA256':root/'belona-live-boundaries.py',
       'entrySHA256':root/'belona-live-entry.py','launcherSHA256':root/'belona-launcher.py','sourceGateSHA256':root/'belona-source-gate.py',
       'sourceProfileSHA256':root/'belona-source-profile.json','cacheInventorySHA256':root/'belona-cache-inventory.json',
       'sdkPythonSHA256':Path('/usr/bin/python3.12'),'sdkManifestSHA256':root/'sdk-manifest.json',
       'cliSHA256':Path('/workspace/runtime/bin/cuttle'),'nodeSHA256':Path('/usr/bin/node'),'cliModuleSHA256':Path('/workspace/runtime/cuttle-cli.mjs')}
for key,path in files.items():pre.pinned(path,subject[key])
pre.verify_toolchain(subject,files['cliSHA256'],files['nodeSHA256'],files['cliModuleSHA256'])
pre.verify_sdk(subject,files['sdkManifestSHA256'],'/workspace/belona-tooling/sdk/site-packages')
config=pre.effective_config()
if config['model']!=subject['model'] or config['reasoningEffort']!=subject['reasoningEffort']:raise ValueError('effective config mismatch')
info=json.loads(subprocess.check_output([str(files['cliSHA256']),'--json','sessions','get',pre.OWNER],timeout=30))
if info.get('id')!=pre.OWNER or info.get('projectId')!=pre.PROJECT or info.get('model')!=subject['model'] or info.get('status')!='running':raise ValueError('session mismatch')
preflight_path=root/'preflight.py'
pre.pinned(preflight_path,pre.PREFLIGHT_SHA)
source=subprocess.run([sys.executable,'-B',str(preflight_path)],capture_output=True,check=True,timeout=120)
result=json.loads(source.stdout)
if result.get('stage')!='SOURCE_PREFLIGHT_ONLY' or result.get('providerContact') is not False or result.get('selection') is not False:raise ValueError('source preflight mismatch')
print(json.dumps({'stage':'PRESELECTION_NONSECRET_EVIDENCE','asOfUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  'subjectSHA256':expected_subject,'effectiveConfig':config,'owner':pre.OWNER,'sessionModel':info['model'],
                  'sdkFilesVerified':1086,'sourcePreflight':result,'sourcePreflightPath':str(preflight_path.resolve()),'sourcePreflightSHA256':pre.PREFLIGHT_SHA,'sourceStdoutSHA256':hashlib.sha256(source.stdout).hexdigest(),
                  'filePins':{key:subject[key] for key in files},'selection':False,'providerContact':False,
                  'reviewCapture':'NOT_ATTEMPTED','scope':'preselection only; not exact reviewed prestart or provider authority'},sort_keys=True))
