"""UNREVIEWED DRAFT. Sole owner, same selected plan and already submitted job only."""
import hashlib, importlib.metadata, json, os, pathlib, re, runpy, signal, sys
from datetime import datetime,timezone
from huggingface_hub import HfApi
ROOT=pathlib.Path(__file__).resolve().parent
OWNER='42689c7a-eddd-44ad-b8bb-04bd0f3c70de'
PLAN='f8013c6ec9a9bc191db918dc8d93c2903cbb1991590887420b97a77cae2b3f9f'
PILOT='2d7c5e44bb37f3043311962f8c05deee90e83ab6fa5ee5e3b499df238ed98200'
FRAMES='f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76'
WRAPPER='fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755'
sha=lambda b:hashlib.sha256(b).hexdigest()
def pin(path,expected):
 raw=path.read_bytes()
 if sha(raw)!=expected:raise RuntimeError('HASH_MISMATCH')
 return raw
def save_exclusive(path,obj):
 fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
 with os.fdopen(fd,'w') as f:json.dump(obj,f,sort_keys=True,indent=2);f.flush();os.fsync(f.fileno())
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
 if globals().get('__verified_wrapper_sha256__')!=WRAPPER:raise RuntimeError('WRAPPER_MISMATCH')
 selected_raw=(ROOT/'selection.json').read_bytes();selected=json.loads(selected_raw)
 if selected.get('owner')!=OWNER or selected.get('collector')!=OWNER or selected.get('collectorEntrySHA256')!=globals().get('__verified_entry_sha256__'):raise RuntimeError('COLLECTOR_SELECTION_MISMATCH')
 plan_raw=pin(ROOT/'pilot-selected-draft.json',PLAN);pin(ROOT/'pilot.py',PILOT);pin(ROOT/'frame_transport.py',FRAMES)
 submission_raw=(ROOT/'submission.json').read_bytes();submission=json.loads(submission_raw)
 if submission.get('collector')!=OWNER or submission.get('selectionSHA256')!=sha(selected_raw) or submission.get('namespace')!='Autismo-Yah' or submission.get('plan')!=json.loads(plan_raw):raise RuntimeError('RECEIPT_MISMATCH')
 job_id=submission.get('job',{}).get('id')
 if not isinstance(job_id,str) or not re.fullmatch('[0-9a-f]{24}',job_id):raise RuntimeError('JOB_ID_INVALID')
 if list(ROOT.glob('status-*.json')) or list(ROOT.glob('logs-*.txt')) or (ROOT/'collection-intent.json').exists():raise RuntimeError('PRIOR_COLLECTION_RECONCILE')
 api=HfApi();identity=bounded(30,lambda:api.whoami())
 if identity.get('name')!='Autismo-Yah':raise RuntimeError('NAMESPACE_MISMATCH')
 info=bounded(30,lambda:api.inspect_job(namespace='Autismo-Yah',job_id=job_id))
 state=getattr(info.status.stage,'value',info.status.stage)
 if state not in ('COMPLETED','ERROR','CANCELED','CANCELLED','DELETED'):raise RuntimeError('NOT_TERMINAL_NO_COLLECTION')
 save_exclusive(ROOT/'collection-intent.json',{'stage':'COLLECTION_PENDING','owner':OWNER,'collector':OWNER,'jobId':job_id,'selectionSHA256':sha(selected_raw),'submissionSHA256':sha(submission_raw),'planSHA256':PLAN,'terminal':state,'utc':datetime.now(timezone.utc).isoformat(),'noAutomaticRecollection':True})
 # Existing pilot reads exactly this plan and submission; no submit path is invoked.
 sys.argv=[str(ROOT/'pilot.py'),'--plan',str(ROOT/'pilot-selected-draft.json'),'--collect']
 try:runpy.run_path(str(ROOT/'pilot.py'),run_name='__main__')
 except BaseException as exc:
  save_exclusive(ROOT/'collection-unknown.json',{'stage':'UNKNOWN_RECONCILE_SAME_JOB','jobId':job_id,'exceptionType':type(exc).__name__});raise
 files=sorted(list(ROOT.glob('status-*.json'))+list(ROOT.glob('logs-*.txt')))
 if len(files)!=2:raise RuntimeError('RAW_COLLECTION_INCOMPLETE')
 for file in files:
  with file.open('rb') as f:os.fsync(f.fileno())
 fd=os.open(str(ROOT),os.O_DIRECTORY)
 try:os.fsync(fd)
 finally:os.close(fd)
 save_exclusive(ROOT/'collection-readback.json',{'stage':'COLLECTED_UNREVIEWED','jobId':job_id,'terminal':state,'files':{p.name:sha(p.read_bytes()) for p in files},'scope':'same-job status/logs; frame completeness and process closure require independent review'})
if __name__=='__main__':main()
