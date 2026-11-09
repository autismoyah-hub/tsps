"""UNSELECTED Zogre source draft: read-only genuine named-reviewer capture.

No marker, selection or collection authority is created by this file.
"""
import json, os, stat, hashlib, shutil, types, re
from pathlib import Path
from datetime import datetime
REVIEWER='d9664b7d-13e2-464d-81e5-37de82992eb2'
EXECUTOR='42689c7a-eddd-44ad-b8bb-04bd0f3c70de'
CLI='/workspace/runtime/bin/cuttle'
SCRATCH=Path('/home/ubuntu/.zogre-recovery-42689c')
MAX_CAPTURE=8*1024*1024
CONTROL=Path('/tmp/zogre-control/rsprox-linux/002/files/process_control.py')
CONTROL_SHA='565e79e713f6881cca014107e3a7d9bf502fc77dc8e3ead152baffed5a53d487'
RUNTIME_PINS={'/workspace/runtime/bin/cuttle':'8eb64dbd53aff50329cc463f22bb26950a1c686fb8042cc45f1d11fb4886a7c6','/workspace/runtime/cuttle-cli.mjs':'a3fe1b62558fab9381af6eb31d7155459555f92502f7b1788ab0130e20042f46','/usr/bin/node':'53fb205ae78805130177e24bcb459a69a1518c8d98f8965f31d85aae7ea840fc'}
COMMAND=(CLI,'--json','sessions','read',REVIEWER,'--turns','1000')
def strict_json(raw):
 def pairs(items):
  out={}
  for key,value in items:
   if key in out:raise ValueError('duplicate JSON key')
   out[key]=value
  return out
 return json.loads(raw,object_pairs_hook=pairs)

def valid_timestamp(value):
 try:
  if not isinstance(value,str) or not value: return False
  return datetime.fromisoformat(value.replace('Z','+00:00')).tzinfo is not None
 except (ValueError,TypeError): return False
def select_turn(raw,subject,receipt_sha):
 obj=strict_json(raw)
 if not isinstance(obj,dict) or not isinstance(obj.get('turns'),list):raise ValueError('invalid transcript')
 expected={'type':'TSPS_DIRECT_REVIEW','verdict':'EXACT_INPUT_PROFILE_GO','reviewer':REVIEWER,'executor':EXECUTOR,'subject':subject,'receiptSHA256':receipt_sha}
 matches=[]
 for turn in obj['turns']:
  if not isinstance(turn,dict) or turn.get('role')!='assistant' or turn.get('kind')!='text':continue
  text=turn.get('text','')
  try:content=strict_json(text)
  except (TypeError,ValueError):
   if isinstance(text,str) and (text.lstrip().startswith('{') or receipt_sha in text or 'TSPS_DIRECT_REVIEW' in text):raise ValueError('ambiguous marker-like turn')
   continue
  if isinstance(content,dict) and content.get('type')=='TSPS_DIRECT_REVIEW' and content.get('reviewer')==REVIEWER and content.get('executor')==EXECUTOR and content.get('receiptSHA256')==receipt_sha and content!=expected:raise ValueError('conflicting review turn')
  if content==expected and valid_timestamp(turn.get('ts')):matches.append(turn)
 if len(matches)!=1:raise ValueError('exact reviewer verdict absent or ambiguous')
 return matches[0]
def _write_at(dfd,name,value):
 raw=(json.dumps(value,sort_keys=True,separators=(',',':'))+'\n').encode('ascii')
 fd=os.open(name,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=dfd)
 try:
  offset=0
  while offset<len(raw):
   n=os.write(fd,raw[offset:])
   if n<=0:raise ValueError('receipt short write')
   offset+=n
  os.fsync(fd);before=os.fstat(fd);named=os.stat(name,dir_fd=dfd,follow_symlinks=False)
  if not stat.S_ISREG(before.st_mode) or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise ValueError('receipt identity')
  os.lseek(fd,0,os.SEEK_SET);chunks=[];total=0
  while True:
   part=os.read(fd,min(65536,len(raw)+1-total))
   if not part:break
   chunks.append(part);total+=len(part)
   if total>len(raw):raise ValueError('receipt too large')
  after=os.fstat(fd);named_after=os.stat(name,dir_fd=dfd,follow_symlinks=False)
  if b''.join(chunks)!=raw or (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino):raise ValueError('receipt changed')
 finally:os.close(fd)
 os.fsync(dfd)
 return hashlib.sha256(raw).hexdigest()

def _bounded_read(scratch,attempt_id,subject_sha,receipt_sha,runner=None):
 # Dedicated foreground, single-threaded exclusive child-reaper contract.
 # Caller precommits this unique attempt ID; never reuse on ambiguity.
 if not isinstance(attempt_id,str) or not re.fullmatch('[0-9a-f]{32}',attempt_id):raise ValueError('attempt id invalid')
 d=os.open(scratch,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);a=None
 try:
  st=os.fstat(d);named=os.stat(scratch,follow_symlinks=False)
  if st.st_uid!=os.geteuid() or stat.S_IMODE(st.st_mode)!=0o700 or (st.st_dev,st.st_ino)!=(named.st_dev,named.st_ino):raise ValueError('scratch not private')
  if os.path.realpath(shutil.which('node',path='/usr/bin:/bin') or '')!='/usr/bin/node':raise ValueError('node path mismatch')
  for path,digest in RUNTIME_PINS.items():
   if hashlib.sha256(Path(path).read_bytes()).hexdigest()!=digest:raise ValueError('CLI runtime pin mismatch')
  control=CONTROL.read_bytes()
  if hashlib.sha256(control).hexdigest()!=CONTROL_SHA:raise ValueError('controller pin mismatch')
  module=types.ModuleType('capture_process_control');exec(compile(control,str(CONTROL),'exec'),module.__dict__)
  # Only offline tests may inject runner; production acquire supplies none.
  operate=runner if runner is not None else module.run_group
  attempt='review-capture-'+attempt_id
  os.mkdir(attempt,0o700,dir_fd=d);os.fsync(d)
  a=os.open(attempt,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=d)
  ast=os.fstat(a);anamed=os.stat(attempt,dir_fd=d,follow_symlinks=False)
  if ast.st_uid!=os.geteuid() or stat.S_IMODE(ast.st_mode)!=0o700 or (ast.st_dev,ast.st_ino)!=(anamed.st_dev,anamed.st_ino):raise ValueError('capture attempt identity')
  _write_at(a,'capture-intent.json',{'stage':'PENDING_UNKNOWN','attemptId':attempt_id,'reviewer':REVIEWER,'subjectSHA256':subject_sha,'receiptSHA256':receipt_sha,'noAutomaticRetry':True})
  prefix='/proc/self/fd/'+str(a)
  child_env={k:os.environ[k] for k in ('CCC_BASE_URL','CCC_TOKEN','CCC_TOKEN_FILE','HOME','LANG','TZ') if k in os.environ};child_env['PATH']='/usr/bin:/bin'
  old_umask=os.umask(0o077)
  try:
   result=operate(list(COMMAND),cwd='/proc/self/fd/'+str(d),env=child_env,stdout_path=prefix+'/stdout',stderr_path=prefix+'/stderr',timeout_seconds=40,grace_seconds=2,output_limit=MAX_CAPTURE,file_limit=MAX_CAPTURE)
  finally:os.umask(old_umask)
  if not isinstance(result,dict) or type(result.get('exit_code')) is not int or result.get('exit_code')!=0 or result.get('timed_out') is not False or result.get('output_truncated') is not False or result.get('drain_incomplete') is not False or result.get('drain_errors')!=[] or result.get('group_remaining') is not False or result.get('group_state')!='absent_after_reap_observed' or result.get('detached_descendant_closure')!='unknown':raise ValueError('capture process unknown')
  counts=result.get('output_bytes')
  if not isinstance(counts,dict) or any(type(counts.get(k)) is not int or counts[k]<0 or counts[k]>=MAX_CAPTURE for k in ('stdout','stderr')):raise ValueError('capture bound uncertain')
  paths={};digests={}
  for key in ('stdout','stderr'):
   fd=os.open(key,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=a)
   try:
    before=os.fstat(fd);named_file=os.stat(key,dir_fd=a,follow_symlinks=False)
    if not stat.S_ISREG(before.st_mode) or stat.S_IMODE(before.st_mode)!=0o600 or (before.st_dev,before.st_ino)!=(named_file.st_dev,named_file.st_ino) or before.st_size!=counts[key]:raise ValueError('capture file mismatch')
    os.fsync(fd);chunks=[];total=0
    while True:
     part=os.read(fd,min(65536,MAX_CAPTURE-total))
     if not part:break
     chunks.append(part);total+=len(part)
     if total>=MAX_CAPTURE:raise ValueError('capture bound uncertain')
    raw=b''.join(chunks);after=os.fstat(fd);named_after=os.stat(key,dir_fd=a,follow_symlinks=False)
    if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino) or len(raw)!=counts[key]:raise ValueError('capture changed')
    paths[key]=raw;digests[key]=hashlib.sha256(raw).hexdigest()
   finally:os.close(fd)
  aend=os.fstat(a);anamed_end=os.stat(attempt,dir_fd=d,follow_symlinks=False)
  end=os.fstat(d);named_end=os.stat(scratch,follow_symlinks=False)
  if (aend.st_dev,aend.st_ino)!=(ast.st_dev,ast.st_ino) or (anamed_end.st_dev,anamed_end.st_ino)!=(ast.st_dev,ast.st_ino) or (end.st_dev,end.st_ino)!=(st.st_dev,st.st_ino) or (named_end.st_dev,named_end.st_ino)!=(st.st_dev,st.st_ino):raise ValueError('capture directory replaced')
  _write_at(a,'capture-result.json',{'stage':'CAPTURE_BYTES_UNREVIEWED','attemptId':attempt_id,'stdoutSHA256':digests['stdout'],'stderrSHA256':digests['stderr'],'closure':result,'detachedDescendants':'UNKNOWN'})
  return paths['stdout'],result,attempt
 finally:
  if a is not None:
   try:os.close(a)
   finally:os.close(d)
  else:os.close(d)

def acquire(subject,receipt_sha,attempt_id):
 if CLI!='/workspace/runtime/bin/cuttle' or COMMAND!=(CLI,'--json','sessions','read','d9664b7d-13e2-464d-81e5-37de82992eb2','--turns','1000'):raise ValueError('named session command changed')
 if not isinstance(receipt_sha,str) or not re.fullmatch('[0-9a-f]{64}',receipt_sha):raise ValueError('receipt digest invalid')
 subject_sha=hashlib.sha256(json.dumps(subject,sort_keys=True,separators=(',',':')).encode()).hexdigest()
 raw,closure,attempt=_bounded_read(SCRATCH,attempt_id,subject_sha,receipt_sha)
 return raw,select_turn(raw,subject,receipt_sha),{'closure':closure,'captureDirectory':attempt}
