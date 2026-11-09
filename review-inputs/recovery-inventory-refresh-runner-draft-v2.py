"""UNSELECTED read-only inventory refresh plan. No SDK import or provider call.

Dedicated foreground/single-thread/exclusive child reaper. Any attempt directory
without an independently validated result is UNKNOWN; never delete or retry.
"""
import base64,hashlib,json,os,pathlib,stat,sys,types
ANCHOR=pathlib.Path('/home/ubuntu/.zogre-recovery-42689c')
SOURCE=pathlib.Path('/tmp/zogre-hf-case/recovery-sdk-tree-inventory-v2.py')
SOURCE_SHA='4fbf6ec2abdff1e49912e00482c859b0d55ff71a1a66ef3c0c98218d49af4afe'
CONTROL=pathlib.Path('/tmp/zogre-control/rsprox-linux/002/files/process_control.py')
CONTROL_SHA='565e79e713f6881cca014107e3a7d9bf502fc77dc8e3ead152baffed5a53d487'
OLD_SHA='7b6a8feadbf0d16d1beddc4fa46e448186301f0a843e56f38220abaa08f82e15'
BOOT="import base64,hashlib,sys; b=base64.b64decode(sys.argv[1],validate=True); h=sys.argv[2]; hashlib.sha256(b).hexdigest()==h or sys.exit(91); exec(compile(b,'verified-sdk-inventory','exec'),{'__name__':'__main__'})"

def put(dfd,name,obj):
 raw=(json.dumps(obj,sort_keys=True,separators=(',',':'))+'\n').encode('ascii')
 fd=os.open(name,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=dfd)
 try:
  i=0
  while i<len(raw):
   n=os.write(fd,raw[i:])
   if n<=0:raise RuntimeError('SHORT_WRITE')
   i+=n
  os.fsync(fd);before=os.fstat(fd);named=os.stat(name,dir_fd=dfd,follow_symlinks=False)
  if not stat.S_ISREG(before.st_mode) or before.st_nlink!=1 or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('RECEIPT_IDENTITY')
  os.lseek(fd,0,os.SEEK_SET);check=os.read(fd,len(raw)+1);after=os.fstat(fd);named_after=os.stat(name,dir_fd=dfd,follow_symlinks=False)
  if check!=raw or (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino):raise RuntimeError('RECEIPT_READBACK')
 finally:os.close(fd)
 os.fsync(dfd)

def read(dfd,name,limit,require_private=False):
 named=os.stat(name,dir_fd=dfd,follow_symlinks=False)
 if not stat.S_ISREG(named.st_mode) or named.st_nlink!=1 or named.st_size>=limit:raise RuntimeError('INPUT_TYPE_OR_BOUND')
 fd=os.open(name,os.O_RDONLY|os.O_NONBLOCK|os.O_NOFOLLOW,dir_fd=dfd)
 try:
  before=os.fstat(fd)
  if not stat.S_ISREG(before.st_mode) or before.st_nlink!=1 or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino) or before.st_size>=limit or (require_private and stat.S_IMODE(before.st_mode)!=0o600):raise RuntimeError('INPUT_IDENTITY')
  os.fsync(fd);chunks=[];size=0
  while True:
   data=os.read(fd,65536)
   if not data:break
   size+=len(data)
   if size>=limit:raise RuntimeError('OUTPUT_BOUND')
   chunks.append(data)
  after=os.fstat(fd);named_after=os.stat(name,dir_fd=dfd,follow_symlinks=False)
  if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino) or size!=before.st_size:raise RuntimeError('INPUT_CHANGED')
  return b''.join(chunks)
 finally:os.close(fd)

def read_path(path,limit):
 parent=path.parent;fd=os.open(parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
 try:
  before=os.fstat(fd);named=os.stat(parent,follow_symlinks=False)
  if before.st_uid!=os.geteuid() or stat.S_IMODE(before.st_mode)&0o022 or (before.st_dev,before.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('INPUT_PARENT_IDENTITY')
  raw=read(fd,path.name,limit)
  after=os.fstat(fd);named_after=os.stat(parent,follow_symlinks=False)
  if (before.st_dev,before.st_ino)!=(after.st_dev,after.st_ino) or (after.st_dev,after.st_ino)!=(named_after.st_dev,named_after.st_ino):raise RuntimeError('INPUT_PARENT_CHANGED')
  return raw
 finally:os.close(fd)

def main():
 if sys.argv[1:]!=['--readonly-once'] or sys.version_info[:3]!=(3,12,3):raise RuntimeError('ACTION_OR_PYTHON')
 if pathlib.Path(sys.executable).resolve()!=pathlib.Path('/usr/bin/python3.12') or hashlib.sha256(pathlib.Path(sys.executable).read_bytes()).hexdigest()!='c2c20b4745d447551221ec3d4e70f92c270c4609fe3df34fc52ea6dd46e92273':raise RuntimeError('PYTHON_PIN')
 source=read_path(SOURCE,1024*1024);control=read_path(CONTROL,1024*1024);old=read_path(ANCHOR/'sdk-tree-inventory-v1.json',2*1024*1024)
 if hashlib.sha256(source).hexdigest()!=SOURCE_SHA or hashlib.sha256(control).hexdigest()!=CONTROL_SHA or hashlib.sha256(old).hexdigest()!=OLD_SHA:raise RuntimeError('INPUT_PIN')
 module=types.ModuleType('verified_controller');exec(compile(control,str(CONTROL),'exec'),module.__dict__)
 afd=os.open(ANCHOR,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);attempt=None
 try:
  root=os.fstat(afd);named=os.stat(ANCHOR,follow_symlinks=False)
  if root.st_uid!=os.geteuid() or root.st_mode&0o777!=0o700 or (root.st_dev,root.st_ino)!=(named.st_dev,named.st_ino):raise RuntimeError('ANCHOR_IDENTITY')
  os.mkdir('sdk-inventory-refresh-v2',0o700,dir_fd=afd);os.fsync(afd)
  attempt=os.open('sdk-inventory-refresh-v2',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=afd)
  attempt_stat=os.fstat(attempt);attempt_named=os.stat('sdk-inventory-refresh-v2',dir_fd=afd,follow_symlinks=False)
  if attempt_stat.st_uid!=os.geteuid() or stat.S_IMODE(attempt_stat.st_mode)!=0o700 or (attempt_stat.st_dev,attempt_stat.st_ino)!=(attempt_named.st_dev,attempt_named.st_ino):raise RuntimeError('ATTEMPT_IDENTITY')
  put(attempt,'intent.json',{'stage':'READ_ONLY_PENDING_NO_RETRY','sourceSHA256':SOURCE_SHA,'oldInventorySHA256':OLD_SHA})
  cmd=[sys.executable,'-I','-S','-B','-c',BOOT,base64.b64encode(source).decode('ascii'),SOURCE_SHA]
  env={k:os.environ[k] for k in ('HOME','LANG','TZ') if k in os.environ};env['PATH']='/usr/bin:/bin'
  prefix='/proc/self/fd/'+str(attempt);oldmask=os.umask(0o077)
  try:
   closure=module.run_group(cmd,cwd='/proc/self/fd/'+str(afd),env=env,stdout_path=prefix+'/stdout.json',stderr_path=prefix+'/stderr.txt',timeout_seconds=40,grace_seconds=2,output_limit=4*1024*1024,file_limit=8*1024*1024)
  finally:os.umask(oldmask)
  if type(closure.get('exit_code')) is not int or closure['exit_code']!=0 or closure.get('timed_out') is not False or closure.get('output_truncated') is not False or closure.get('drain_incomplete') is not False or closure.get('drain_errors')!=[] or closure.get('group_remaining') is not False or closure.get('group_state')!='absent_after_reap_observed':raise RuntimeError('CONTROLLER_UNKNOWN')
  stdout=read(attempt,'stdout.json',4*1024*1024,require_private=True);stderr=read(attempt,'stderr.txt',4*1024*1024,require_private=True)
  if closure.get('output_bytes')!={'stdout':len(stdout),'stderr':len(stderr)}:raise RuntimeError('OUTPUT_COUNTS')
  doc=json.loads(stdout);prior=json.loads(old)
  if type(doc.get('schema')) is not int or doc['schema']!=1 or doc.get('root')!='/tmp/zogre-hf-sdk/site-packages' or not isinstance(doc.get('entries'),list):raise RuntimeError('INVENTORY_SCHEMA')
  same=doc['entries']==prior.get('entries')
  aend=os.fstat(attempt);anamed=os.stat('sdk-inventory-refresh-v2',dir_fd=afd,follow_symlinks=False);rend=os.fstat(afd);rnamed=os.stat(ANCHOR,follow_symlinks=False)
  if (aend.st_dev,aend.st_ino)!=(attempt_stat.st_dev,attempt_stat.st_ino) or (anamed.st_dev,anamed.st_ino)!=(aend.st_dev,aend.st_ino) or (rend.st_dev,rend.st_ino)!=(root.st_dev,root.st_ino) or (rnamed.st_dev,rnamed.st_ino)!=(rend.st_dev,rend.st_ino):raise RuntimeError('CUSTODY_CHANGED')
  put(attempt,'result.json',{'stage':'READ_ONLY_ROWS_MATCH_PENDING_OUTER' if same else 'READ_ONLY_ROWS_DIFFER_PENDING_OUTER','stdoutSHA256':hashlib.sha256(stdout).hexdigest(),'stderrSHA256':hashlib.sha256(stderr).hexdigest(),'entries':len(doc['entries']),'closure':closure,'scope':'source tree point-in-time only; copied snapshot, future import and detached closure unproved'})
 except BaseException:
  if attempt is not None:
   try:put(attempt,'unknown.json',{'stage':'UNKNOWN_RECONCILE_NO_RETRY','scope':'read-only SDK source inventory'})
   except BaseException:pass
  raise
 finally:
  if attempt is not None:
   try:os.close(attempt)
   finally:os.close(afd)
  else:os.close(afd)
if __name__=='__main__':main()
