"""UNREVIEWED DRAFT: terminal-only bounded same-job collection, never submits."""
import dataclasses,hashlib,importlib.metadata,json,os,pathlib,re,signal,sys,types
from datetime import datetime,timezone
from huggingface_hub import HfApi
ROOT=pathlib.Path(__file__).resolve().parent
OWNER='42689c7a-eddd-44ad-b8bb-04bd0f3c70de';SOURCE='4619ba5838569a68b0f56bb46ab6b2b77c930586'
PLAN='f8013c6ec9a9bc191db918dc8d93c2903cbb1991590887420b97a77cae2b3f9f'
FRAMES='f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76'
COLLECT_LAUNCHER='773dbe1dced4afda72fc78c07a35c7185d4e9d1951d97a04f4797f50088e8cf2'
WRAPPER='fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755'
SDK_API='3871f6516f2bf97b77cf4179daea929b30b4c97d7e87e9158fa742f46cbf4860'
SDK_JOBS='4f6e280e0f1938d3230625384b25acdb360aac2925fe23040a8525c0bc1306e2'
sha=lambda b:hashlib.sha256(b).hexdigest()
def pin(path,expected):
 raw=path.read_bytes()
 if sha(raw)!=expected:raise RuntimeError('HASH_MISMATCH')
 return raw
def save(path,obj):
 fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
 with os.fdopen(fd,'w') as f:json.dump(obj,f,sort_keys=True,indent=2,default=str);f.flush();os.fsync(f.fileno())
 fd=os.open(str(path.parent),os.O_DIRECTORY)
 try:os.fsync(fd)
 finally:os.close(fd)
def bounded(sec,fn):
 def timeout(_sig,_frame):raise TimeoutError('TIMEOUT')
 old=signal.signal(signal.SIGALRM,timeout);signal.alarm(sec)
 try:return fn()
 finally:signal.alarm(0);signal.signal(signal.SIGALRM,old)
def main():
 if sys.argv[1:]!=['--collect-once'] or os.environ.get('CCC_SESSION_ID')!=OWNER or sys.version_info[:3]!=(3,12,3):raise RuntimeError('WRONG_EXECUTOR')
 if globals().get('__verified_wrapper_sha256__')!=WRAPPER or globals().get('__verified_launcher_sha256__')!=COLLECT_LAUNCHER:raise RuntimeError('WRAPPER_MISMATCH')
 import huggingface_hub
 if importlib.metadata.version('huggingface_hub')!='2.4.0' or pathlib.Path(huggingface_hub.__file__).resolve()!=pathlib.Path('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/__init__.py'):raise RuntimeError('SDK_IDENTITY')
 pin(pathlib.Path('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/hf_api.py'),SDK_API);pin(pathlib.Path('/tmp/zogre-hf-sdk/site-packages/huggingface_hub/_jobs_api.py'),SDK_JOBS)
 selected_raw=(ROOT/'selection.json').read_bytes();selected=json.loads(selected_raw)
 if selected.get('owner')!=OWNER or selected.get('collector')!=OWNER or selected.get('collectorLauncherSHA256')!=COLLECT_LAUNCHER or selected.get('collectorEntrySHA256')!=globals().get('__verified_entry_sha256__'):raise RuntimeError('COLLECTOR_SELECTION_MISMATCH')
 entry_raw=pin(ROOT/'case-entry-v16.py',selected['entrySHA256']);authority=types.ModuleType('verified_case_authority');authority.__file__=str(ROOT/'case-entry-v16.py');exec(compile(entry_raw,authority.__file__,'exec'),authority.__dict__)
 authority.check_config();authority.current_state(selected,sha(selected_raw),selected['entrySHA256'],selected['programSHA256'],collection=True)
 plan_raw=pin(ROOT/'pilot-selected-draft.json',PLAN);plan=json.loads(plan_raw)
 frames_raw=pin(ROOT/'frame_transport.py',FRAMES);frame_mod=types.ModuleType('verified_frame_transport');exec(compile(frames_raw,'verified_frame_transport.py','exec'),frame_mod.__dict__)
 submission_raw=(ROOT/'submission.json').read_bytes();submission=json.loads(submission_raw);intent_raw=(ROOT/'case-intent-v16.json').read_bytes();intent=json.loads(intent_raw)
 if submission.get('collector')!=OWNER or submission.get('selectionSHA256')!=sha(selected_raw) or submission.get('namespace')!='Autismo-Yah' or submission.get('plan')!=plan or submission.get('claimCommit')!=intent.get('claimCommit') or submission.get('request')!=intent.get('request') or intent.get('source')!=SOURCE:raise RuntimeError('RECEIPT_MISMATCH')
 job_id=submission.get('job',{}).get('id')
 if not isinstance(job_id,str) or not re.fullmatch('[0-9a-f]{24}',job_id):raise RuntimeError('JOB_ID_INVALID')
 if list(ROOT.glob('status-*.json')) or list(ROOT.glob('logs-*.txt')) or (ROOT/'collection-intent.json').exists():raise RuntimeError('PRIOR_COLLECTION_RECONCILE')
 api=HfApi();identity=bounded(30,lambda:api.whoami())
 if identity.get('name')!='Autismo-Yah':raise RuntimeError('NAMESPACE_MISMATCH')
 info=bounded(30,lambda:api.inspect_job(namespace='Autismo-Yah',job_id=job_id));req=intent['request']
 if info.id!=job_id or info.owner.name!='Autismo-Yah' or info.docker_image!=req['image'] or info.command!=req['command'] or info.flavor!=req['flavor'] or info.labels!=req['labels'] or info.retry!=0:raise RuntimeError('INSPECTED_REQUEST_MISMATCH')
 state=getattr(info.status.stage,'value',info.status.stage)
 if state not in ('COMPLETED','ERROR','CANCELED','CANCELLED','DELETED'):raise RuntimeError('NOT_TERMINAL_NO_COLLECTION')
 authority.check_config();authority.current_state(selected,sha(selected_raw),selected['entrySHA256'],selected['programSHA256'],collection=True)
 save(ROOT/'collection-intent.json',{'stage':'COLLECTION_PENDING','owner':OWNER,'collector':OWNER,'jobId':job_id,'selectionSHA256':sha(selected_raw),'submissionSHA256':sha(submission_raw),'intentSHA256':sha(intent_raw),'planSHA256':PLAN,'terminal':state,'utc':datetime.now(timezone.utc).isoformat(),'noAutomaticRecollection':True})
 stamp=datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ');status=ROOT/f'status-{stamp}.json';logs=ROOT/f'logs-{stamp}.txt';records=ROOT/f'records-{stamp}.jsonl';decoded=ROOT/f'decoded-{stamp}'
 try:
  save(status,dataclasses.asdict(info));lines=[];total=0
  def fetch():
   nonlocal total
   with logs.open('xb') as f, records.open('xb') as g:
    for line in api.fetch_job_logs(namespace='Autismo-Yah',job_id=job_id,follow=False):
     data=(line+'\n').encode();total+=len(data)
     if len(data)>1048576 or total>16777216:raise RuntimeError('LOG_BOUND_EXCEEDED')
     f.write(data);g.write((json.dumps(line,ensure_ascii=True)+'\n').encode());lines.append(line)
    f.flush();os.fsync(f.fileno());g.flush();os.fsync(g.fileno())
  bounded(120,fetch)
  result=bounded(60,lambda:frame_mod.reconstruct_child_outputs(lines,decoded))
  for p in decoded.rglob('*') if decoded.exists() else []:
   if p.is_file():
    with p.open('rb') as f:os.fsync(f.fileno())
  fd=os.open(str(ROOT),os.O_DIRECTORY)
  try:os.fsync(fd)
  finally:os.close(fd)
  save(ROOT/'collection-readback.json',{'stage':'COLLECTED_UNREVIEWED','jobId':job_id,'terminal':state,'statusSHA256':sha(status.read_bytes()),'logsSHA256':sha(logs.read_bytes()),'recordBoundariesSHA256':sha(records.read_bytes()),'frameResult':result,'scope':'bounded same-job logs; frame correctness, process closure and provider terminal independently reviewed'})
 except BaseException as exc:
  save(ROOT/'collection-unknown.json',{'stage':'UNKNOWN_RECONCILE_SAME_JOB','jobId':job_id,'exceptionType':type(exc).__name__});raise
if __name__=='__main__':main()
