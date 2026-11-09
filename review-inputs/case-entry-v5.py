"""UNREVIEWED DRAFT; never run without independent exact-source review.
One baseline attempt; a retained create-only Git ref is the cross-root claim.
"""
import dataclasses, hashlib, importlib.metadata, json, os, pathlib, re, shlex, signal, subprocess, sys, tempfile, urllib.request
from datetime import datetime, timezone
from huggingface_hub import HfApi
ROOT=pathlib.Path(__file__).resolve().parent
SOURCE='4619ba5838569a68b0f56bb46ab6b2b77c930586'
PLAN='f8013c6ec9a9bc191db918dc8d93c2903cbb1991590887420b97a77cae2b3f9f'
PROGRAM='b94244afa80bf08ff4124e3c7788e64f56146ec9fd448864ddce51ed426bf5c3'
PAYLOAD='220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4'
MANIFEST='d3a5e8b0136e7af419c221962990fbb6aec582911cfc4b4be0f2673a98561e5f'
WRAPPER='fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755'
LAUNCHER='00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'
CASE_LAUNCHER='a218c70278437cbf79c94e7f229a65bee5c3c6f1bfb26b875990a9d2da7d25aa'
COLLECT_ENTRY='0f3ad3d7c1a68cd6f9aa4c28f50f3c849c9b322333777398fb367e656691f22c'
FRAMES='f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76'
RETAINER='bdcb7f104ebfa8c26d2aa68c1dd31cf7780f12174eb92d3e98ffb928797a9071'
COLLECTOR='2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200'
SDK_API='3871f6516f2bf97b77cf4179daea929b30b4c97d7e87e9158fa742f46cbf4860'
SDK_JOBS='4f6e280e0f1938d3230625384b25acdb360aac2925fe23040a8525c0bc1306e2'
CASE='zogre-uglug-baseline-42689c'
PROFILE='8eb0af645befdd9c250e8716ab9adabfeeb65fc2044a33852e0d3eaad2aacb06'
OWNER='42689c7a-eddd-44ad-b8bb-04bd0f3c70de'
PARENT='ad80bdbf-4243-4fee-9928-88fd6836505c'
REVIEWER='d9664b7d-13e2-464d-81e5-37de82992eb2'
REPO='https://github.com/autismoyah-hub/tsps.git'
REF='refs/heads/case-intents/zogre-uglug-baseline-42689c'
sha=lambda b: hashlib.sha256(b).hexdigest()
def fail(message): raise RuntimeError(message)
def pin(file,expected):
 raw=pathlib.Path(file).read_bytes()
 if sha(raw)!=expected: fail('HASH_MISMATCH:'+str(file))
 return raw
def run(args,**kwargs): return subprocess.run(args,check=True,capture_output=True,text=True,timeout=40,**kwargs).stdout
def atomic_json(file,value):
 data=(json.dumps(value,sort_keys=True,indent=2)+'\n').encode()
 fd=os.open(file,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
 with os.fdopen(fd,'wb') as f: f.write(data); f.flush(); os.fsync(f.fileno())
 dfd=os.open(str(pathlib.Path(file).parent),os.O_DIRECTORY)
 try: os.fsync(dfd)
 finally: os.close(dfd)
def parse_utc(value):
 try: return datetime.fromisoformat(value.replace('Z','+00:00')).astimezone(timezone.utc)
 except Exception: fail('INVALID_TIMESTAMP')
def read_turns(session, selected):
 data=json.loads(run(['cuttle','sessions','read','--turns','1000',session]))
 turns=data.get('turns',[])
 if not turns or not turns[0].get('ts') or parse_utc(turns[0]['ts'])>selected: fail('TRANSCRIPT_WINDOW_INCOMPLETE:'+session)
 return [t for t in turns if t.get('ts') and parse_utc(t['ts'])>selected]
def known_hold(text):
 # Remove quoted lines; actual directives remain conservative and fail closed.
 lines=[x for x in text.splitlines() if not x.lstrip().startswith('>')]
 body='\n'.join(lines).lower()
 if re.search(r'\b(stop|pause|cancel|revoke|halt|hold)\b',body): return True
 return bool(re.search(r'\b(do not|don.t|must not|never)\b.{0,80}\b(submit|launch|execute|start|run|select|claim)\b',body,re.S))
def current_state(selection,selection_sha,own_sha,program_sha):
 for session in (OWNER,PARENT,REVIEWER):
  obj=json.loads(run(['cuttle','--json','sessions','get',session]))
  if obj.get('id')!=session or obj.get('status')!='running' or obj.get('projectId')!='4cbb8d38-9dae-47c7-9736-c0020aea1c2a': fail('SESSION_IDENTITY_OR_STATUS:'+session)
  if session==OWNER and (obj.get('parentSessionId')!=PARENT or obj.get('transport')!='acp'): fail('OWNER_PARENT_OR_TRANSPORT')
 selected=parse_utc(selection['selectedUTC']);age=(datetime.now(timezone.utc)-selected).total_seconds()
 if age<0 or age>900: fail('SELECTION_TIME_INVALID_OR_STALE')
 if sha((ROOT/'selection.json').read_bytes())!=selection_sha: fail('SELECTION_CHANGED')
 own=read_turns(OWNER,selected);parent=read_turns(PARENT,selected);peer=read_turns(REVIEWER,selected)
 for session,turns in ((OWNER,own),(PARENT,parent)):
  for turn in turns:
   if turn.get('role')=='user' and known_hold(turn.get('text') or ''): fail('KNOWN_STOP_OR_REVOCATION:'+session)
 marker=f"UGLUG_CASE_EXECUTOR_GO owner={OWNER} reviewer={REVIEWER} source={SOURCE} entry={own_sha} plan={PLAN} program={program_sha} profile={PROFILE} request={selection['requestSHA256']} selection={selection_sha} wrapper={WRAPPER} launcher={CASE_LAUNCHER} helper={LAUNCHER} collectorEntry={COLLECT_ENTRY} collector={OWNER}"
 # Correlate peer's exact tool-send action and its result with delivery in owner's transcript.
 delivered=[t for t in own if t.get('role')=='user' and REVIEWER in (t.get('text') or '') and (t.get('text') or '').split('\n',1)[-1]==marker]
 matches=[]
 for i,t in enumerate(peer):
  if t.get('role')!='assistant' or t.get('kind')!='tool': continue
  tool=t.get('tool') or {}; command=tool.get('name') or ''
  try: argv=shlex.split(command)
  except ValueError: continue
  if argv!=['cuttle','sessions','send',OWNER,marker]: continue
  results=[q for q in peer[i+1:] if q.get('kind')=='tool_result' and (q.get('tool') or {}).get('id')==tool.get('id')]
  if results and delivered and any(parse_utc(q['ts'])>=parse_utc(t['ts']) for q in delivered): matches.append(t)
 if not matches: fail('NO_CORRELATED_REVIEW_SEND_AND_DELIVERY')
 attested=matches[-1]['ts']
 if any(t.get('role')=='assistant' and t.get('kind')=='text' and parse_utc(t['ts'])>parse_utc(attested) and 'UGLUG_CASE_EXECUTOR_REVOKE' in (t.get('text') or '') for t in peer): fail('REVIEW_WITHDRAWN')
 return {'reviewMarker':marker,'reviewUTC':attested,'checkedUTC':datetime.now(timezone.utc).isoformat(),'provenance':'correlated Cuttle peer tool action and recipient delivery; not signature or atomic revocation fence'}
def check_config():
 req=urllib.request.Request(os.environ['CCC_BASE_URL']+'/api/acp/sessions/'+OWNER+'/config',headers={'X-CCC-Control-Token':os.environ['CCC_TOKEN']})
 with urllib.request.urlopen(req,timeout=15) as r: obj=json.load(r)
 if obj.get('ok') is not True: fail('CONFIG_UNAVAILABLE')
 options={v['id']:v.get('currentValue') for v in obj['configOptions']}
 if any(options.get(k)!=v for k,v in {'model':'gpt-6.1-terra','reasoning_effort':'medium','mode':'agent-full-access','collaboration_mode':'default'}.items()): fail('EFFECTIVE_CONFIG_CHANGED')
def bounded(seconds,func):
 def timeout(_signum,_frame): raise TimeoutError('BOUNDED_CALL_TIMEOUT')
 old=signal.signal(signal.SIGALRM,timeout);signal.alarm(seconds)
 try: return func()
 finally: signal.alarm(0);signal.signal(signal.SIGALRM,old)
def no_duplicate(api,namespace):
 def scan():
  for job in api.list_jobs(namespace=namespace,labels={'project':'tsps'},timeout=20):
   labels=job.labels or {}
   if labels.get('source')==SOURCE or labels.get('case')==CASE: fail('PRIOR_PROVIDER_JOB_RECONCILE')
 bounded(45,scan)

def main():
 if sys.flags.optimize or sys.version_info[:3]!=(3,12,3) or sys.argv[1:]!=['--submit-once']: fail('EXPLICIT_SUBMIT_REQUIRED')
 if os.environ.get('CCC_SESSION_ID')!=OWNER: fail('WRONG_SESSION')
 selection_path=ROOT/'selection.json'; selection_raw=selection_path.read_bytes();selection_sha=sha(selection_raw);selection=json.loads(selection_raw)
 own_sha=globals().get('__verified_entry_sha256__')
 if globals().get('__verified_wrapper_sha256__')!=WRAPPER or globals().get('__verified_launcher_sha256__')!=CASE_LAUNCHER: fail('LAUNCHER_OR_WRAPPER_SNAPSHOT_MISMATCH')
 if not own_sha or selection.get('entrySHA256')!=own_sha or selection.get('owner')!=OWNER or selection.get('collector')!=OWNER or selection.get('reviewer')!=REVIEWER or selection.get('collectorEntrySHA256')!=COLLECT_ENTRY or selection.get('launcherSHA256')!=CASE_LAUNCHER or selection.get('action')!='SELECT_ONCE' or selection.get('planSHA256')!=PLAN or selection.get('programSHA256')!=PROGRAM: fail('SELECTION_INVALID')
 if importlib.metadata.version('huggingface_hub')!='2.4.0': fail('SDK_VERSION')
 pin('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/hf_api.py',SDK_API);pin('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/_jobs_api.py',SDK_JOBS)
 import huggingface_hub
 if pathlib.Path(huggingface_hub.__file__).resolve()!=pathlib.Path('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/__init__.py'): fail('SDK_IMPORT_PATH_MISMATCH')
 pin(ROOT/'collect-case-v1.py',COLLECT_ENTRY);pin(ROOT/'pilot.py',COLLECTOR);pin(ROOT/'frame_transport.py',FRAMES);pin(ROOT/'retain-job-result.py',RETAINER)
 check_config(); current_state(selection,selection_sha,own_sha,PROGRAM)
 plan=json.loads(pin(ROOT/'pilot-selected-draft.json',PLAN)); payload=pin(ROOT/'native-payload.cjs',PAYLOAD).decode()
 basis=json.loads(pin(ROOT/'profile-basis.json',PROFILE))
 for key in ['image','flavor','timeout','attempts','expectedCacheFiles']:
  if plan[key]!=basis[key]: fail('PROFILE_MISMATCH:'+key)
 if plan['repository']!='autismoyah-hub/tsps' or plan['source']!=SOURCE or plan['tests']!=['zogre-uglug-sale.test.cjs'] or plan['expectedTestCounts']!={'zogre-uglug-sale.test.cjs':16} or plan.get('executionHold') is not None: fail('PLAN_SUBJECT_MISMATCH')
 ref=plan['expectedInputsRef']
 expected={'schema':1,'repository':'autismoyah-hub/tsps','artifactCommit':'34a9d304fc9ad8c87bf2b9f11592cf8893590e43','path':'input-manifests/zogre-uglug-baseline.json','sha256':MANIFEST,'bytes':1096258,'source':SOURCE,'count':5055}
 if ref!=expected or payload.count('__PLAN__')!=1: fail('INPUT_SPEC_MISMATCH')
 program=payload.replace('__PLAN__',json.dumps(plan)); program_bytes=program.encode()
 if len(program_bytes)+1>=131072 or sha(program_bytes)!=PROGRAM: fail('PROGRAM_MISMATCH')
 url=f"https://raw.githubusercontent.com/{ref['repository']}/{ref['artifactCommit']}/{ref['path']}"
 with urllib.request.urlopen(url,timeout=40) as response: raw=response.read(ref['bytes']+1)
 if len(raw)!=ref['bytes'] or sha(raw)!=MANIFEST: fail('REMOTE_MANIFEST_MISMATCH')
 doc=json.loads(raw)
 if doc['source']!=SOURCE or doc['tests']!=plan['tests'] or doc['expectedTestCounts']!=plan['expectedTestCounts'] or len(doc['expectedInputs'])!=5055 or set(doc['expectedInputs'])!=set(doc['sourceBlobs']): fail('MANIFEST_SUBJECT_MISMATCH')
 branch=run(['git','ls-remote',REPO,'refs/heads/wip/zogre-uglug-fixture-42689c']).split()
 if len(branch)!=2 or branch[0]!=SOURCE: fail('SOURCE_REF_CHANGED')
 with urllib.request.urlopen(urllib.request.Request(f'https://codeload.github.com/autismoyah-hub/tsps/tar.gz/{SOURCE}',method='HEAD'),timeout=40) as r:
  if r.status!=200: fail('SOURCE_ARCHIVE_UNAVAILABLE')
 review=current_state(selection,selection_sha,own_sha,PROGRAM)
 request={'name':plan['name'],'image':plan['image'],'flavor':plan['flavor'],'timeout':plan['timeout'],'attempts':1,'command':['node','-e',program], 'labels':{'project':'tsps','source':SOURCE,'purpose':'overflow-qualification','case':CASE}}
 if sha(json.dumps(request,sort_keys=True).encode())!=selection.get('requestSHA256'): fail('REQUEST_BINDING_MISMATCH')
 if any((ROOT/name).exists() for name in ('case-intent-v2.json','case-intent-v4.json','case-intent-v5.json','provider-return.json','submission.json','submission-unknown.json')): fail('LOCAL_PRIOR_INTENT_RECONCILE')
 api=HfApi(); namespace=bounded(30,lambda:api.whoami())['name']
 if namespace!='Autismo-Yah': fail('WRONG_NAMESPACE')
 # Any prior source-matching job, regardless of state, requires reconciliation.
 no_duplicate(api,namespace)
 intent={'stage':'SUBMISSION_PENDING','owner':OWNER,'reviewer':REVIEWER,'collector':OWNER,'source':SOURCE,'planSHA256':PLAN,'payloadSHA256':PAYLOAD,'programSHA256':PROGRAM,'manifestSHA256':MANIFEST,'entrySHA256':own_sha,'profileSHA256':PROFILE,'namespace':namespace,'request':request,'review':review,'selectionSHA256':selection_sha,'wrapperSHA256':WRAPPER,'launcherSHA256':CASE_LAUNCHER,'collectorEntrySHA256':COLLECT_ENTRY,'collectorSHA256':COLLECTOR,'framesSHA256':FRAMES,'retainerSHA256':RETAINER,'sdkVersion':'2.4.0','attempts':1,'noAutomaticRetry':True,'createdUTC':datetime.now(timezone.utc).isoformat()}
 # Remote immutable claim is created before provider write. Never delete, reset, or retry on ambiguity.
 with tempfile.TemporaryDirectory(prefix='uglug-intent-') as tmp:
  env=os.environ.copy();env['GIT_INDEX_FILE']=str(pathlib.Path(tmp)/'index')
  run(['git','read-tree','--empty'],env=env)
  raw=(json.dumps(intent,sort_keys=True,indent=2)+'\n').encode();obj=subprocess.run(['git','hash-object','-w','--stdin'],input=raw,check=True,capture_output=True,timeout=40).stdout.decode().strip()
  run(['git','update-index','--add','--cacheinfo',f'100644,{obj},intent.json'],env=env)
  tree=run(['git','write-tree'],env=env).strip()
  commit=subprocess.run(['git','commit-tree',tree],input=b'Zogre Uglug baseline sole intent\n',check=True,capture_output=True,timeout=40).stdout.decode().strip()
  if not re.fullmatch('[0-9a-f]{40}',commit): fail('CLAIM_COMMIT_INVALID')
  atomic_json(ROOT/'case-intent-v5.json',dict(intent,claimCommit=commit,claimRef=REF))
  # Any failure or timeout is ambiguous; persistent local intent forbids automatic retry.
  check_config();current_state(selection,selection_sha,own_sha,PROGRAM);no_duplicate(api,namespace)
  run(['git','-c','credential.helper=!gh auth git-credential','push',f'--force-with-lease={REF}:',REPO,f'{commit}:{REF}'])
  rows=run(['git','ls-remote',REPO,REF]).split()
  if rows!=[commit,REF]: fail('CLAIM_READBACK_AMBIGUOUS')
  check_config();current_state(selection,selection_sha,own_sha,PROGRAM);no_duplicate(api,namespace)
  # A timeout or exception must leave case-intent-v5.json and remote ref intact.
  try:
   info=bounded(60,lambda:api.run_job(namespace=namespace,**request))
   job_id=getattr(info,'id',None)
   if not isinstance(job_id,str) or not re.fullmatch('[0-9a-f]{24}',job_id): fail('RETURNED_JOB_ID_INVALID')
   atomic_json(ROOT/'provider-return.json',{'stage':'RETURNED_UNINSPECTED','jobId':job_id,'namespace':namespace,'claimCommit':commit,'selectionSHA256':selection_sha,'collector':OWNER,'requestSHA256':sha(json.dumps(request,sort_keys=True).encode())})
   if getattr(getattr(info,'owner',None),'name',None)!=namespace: fail('RETURNED_OWNER_MISMATCH')
   inspected=bounded(30,lambda:api.inspect_job(job_id=job_id,namespace=namespace))
   if inspected.id!=job_id or inspected.owner.name!=namespace or inspected.docker_image!=request['image'] or inspected.command!=request['command'] or inspected.flavor!=request['flavor'] or inspected.labels!=request['labels'] or inspected.retry!=0: fail('INSPECTED_REQUEST_MISMATCH')
   job=dataclasses.asdict(info)
   # stringify datetimes as accepted collector does
   compatible=json.loads(json.dumps({'stage':'SUBMITTED','namespace':namespace,'request':request,'payloadSHA256':PROGRAM,'plan':plan,'job':job,'claimCommit':commit,'collector':OWNER,'selectionSHA256':selection_sha},default=str))
   atomic_json(ROOT/'submission.json',compatible)
   print(json.dumps({'stage':'SUBMITTED','jobId':job_id,'claimCommit':commit,'collector':OWNER}))
  except BaseException as e:
   atomic_json(ROOT/'submission-unknown.json',{'stage':'UNKNOWN_RECONCILE_NO_RETRY','exceptionType':type(e).__name__,'claimCommit':commit,'selectionSHA256':selection_sha,'collector':OWNER})
   raise
if __name__=='__main__': main()
