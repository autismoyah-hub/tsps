"""UNREVIEWED DRAFT. Launch verified auth wrapper snapshot and case entry exactly once."""
import base64,hashlib,importlib.util,json,os,pathlib,signal,subprocess,sys,time
ROOT=pathlib.Path(__file__).resolve().parent
HELPER=pathlib.Path('/tmp/zogre-control/hf-author-control/004/files/authenticated-child-snapshot-command.py')
HELPER_SHA='00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'
WRAPPER_SHA='fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755'
sha=lambda b:hashlib.sha256(b).hexdigest()
def main():
 if sys.version_info[:3]!=(3,12,3) or sys.argv[1:]!=['--submit-once']:raise RuntimeError('EXACT_INTERPRETER_AND_EXPLICIT_ACTION_REQUIRED')
 helper=HELPER.read_bytes()
 if sha(helper)!=HELPER_SHA:raise RuntimeError('HELPER_HASH_MISMATCH')
 selection=json.loads((ROOT/'selection.json').read_bytes());entry=ROOT/'case-entry-v5.py';wrapper=ROOT/'auth-wrapper-v4.py'
 if globals().get('digest')!=selection.get('launcherSHA256'):raise RuntimeError('LAUNCHER_SNAPSHOT_MISMATCH')
 snapshot=wrapper.read_bytes()
 if sha(snapshot)!=WRAPPER_SHA or sha(entry.read_bytes())!=selection['entrySHA256']:raise RuntimeError('SNAPSHOT_MISMATCH')
 spec=importlib.util.spec_from_loader('accepted_snapshot_helper',loader=None);module=importlib.util.module_from_spec(spec);exec(compile(helper,str(HELPER),'exec'),module.__dict__)
 cmd=[sys.executable,'-c',module.CHILD_BOOTSTRAP,base64.b64encode(snapshot).decode('ascii'),WRAPPER_SHA,str(wrapper),'/tmp/zogre-hf-sdk/site-packages','--entry',str(entry),'--entry-sha256',selection['entrySHA256'],'--','--submit-once']
 if any('\0' in x or len(x.encode())+1>=131072 for x in cmd):raise RuntimeError('ARGV_BOUND')
 # A timeout or post-launch logging error is UNKNOWN; never resubmit automatically.
 env=os.environ.copy();env['TSPS_LAUNCHER_SNAPSHOT_SHA256']=globals()['digest']
 child=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True,env=env)
 timed_out=False
 try: out,err=child.communicate(timeout=240)
 except subprocess.TimeoutExpired:
  timed_out=True
  try: os.killpg(child.pid,signal.SIGKILL)
  except ProcessLookupError: pass
  out,err=child.communicate()
 leftover=False
 try: os.killpg(child.pid,0);leftover=True
 except ProcessLookupError: pass
 if leftover:
  try: os.killpg(child.pid,signal.SIGKILL)
  except ProcessLookupError: pass
 closure={'returncode':child.returncode,'timeout':timed_out,'groupLeftoverAtCheck':leftover,'scope':'owned process group only; detached descendants not proven absent','stdoutSHA256':sha(out),'stderrSHA256':sha(err)}
 try:
  for name,data in [('launch-stdout.txt',out),('launch-stderr.txt',err),('launch-closure.json',(json.dumps(closure,sort_keys=True)+'\n').encode())]:
   with (ROOT/name).open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
  fd=os.open(str(ROOT),os.O_DIRECTORY)
  try:os.fsync(fd)
  finally:os.close(fd)
 except BaseException:
  # A prior output, disk error or fsync failure cannot authorize another submission.
  print(json.dumps({'stage':'UNKNOWN_LOGGING_FAILURE_RECONCILE_SAME_INTENT','scope':closure['scope']}));raise
 print(json.dumps(dict(closure,stage='UNKNOWN_RECONCILE' if timed_out or leftover else 'CHILD_RETURNED_RECONCILE_RECEIPT')))
if __name__=='__main__':main()
