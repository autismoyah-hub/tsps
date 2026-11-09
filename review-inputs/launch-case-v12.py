"""UNREVIEWED DRAFT. Launch verified auth wrapper snapshot and case entry exactly once."""
import base64,hashlib,importlib.util,json,os,pathlib,signal,subprocess,sys,time
ROOT=pathlib.Path(__file__).resolve().parent
HELPER=pathlib.Path('/tmp/zogre-hf-case/authenticated-child-v2.py')
CONTROL=pathlib.Path('/tmp/zogre-control/rsprox-linux/002/files/process_control.py')
CONTROL_SHA='565e79e713f6881cca014107e3a7d9bf502fc77dc8e3ead152baffed5a53d487'
HELPER_SHA='4663c9f62db6556261be389ab9830db755c711e1f7182f69f187df2635d97853'
WRAPPER_SHA='fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755'
sha=lambda b:hashlib.sha256(b).hexdigest()
def main():
 if sys.version_info[:3]!=(3,12,3) or sys.argv[1:]!=['--submit-once']:raise RuntimeError('EXACT_INTERPRETER_AND_EXPLICIT_ACTION_REQUIRED')
 helper=HELPER.read_bytes()
 if sha(helper)!=HELPER_SHA:raise RuntimeError('HELPER_HASH_MISMATCH')
 selection=json.loads((ROOT/'selection.json').read_bytes());entry=ROOT/'case-entry-v12.py';wrapper=ROOT/'auth-wrapper-v4.py'
 if globals().get('digest')!=selection.get('launcherSHA256'):raise RuntimeError('LAUNCHER_SNAPSHOT_MISMATCH')
 snapshot=wrapper.read_bytes()
 if sha(snapshot)!=WRAPPER_SHA or sha(entry.read_bytes())!=selection['entrySHA256']:raise RuntimeError('SNAPSHOT_MISMATCH')
 spec=importlib.util.spec_from_loader('accepted_snapshot_helper',loader=None);module=importlib.util.module_from_spec(spec);exec(compile(helper,str(HELPER),'exec'),module.__dict__)
 cmd=[sys.executable,'-c',module.CHILD_BOOTSTRAP,base64.b64encode(snapshot).decode('ascii'),WRAPPER_SHA,str(wrapper),'/tmp/zogre-hf-sdk/site-packages','--entry',str(entry),'--entry-sha256',selection['entrySHA256'],'--','--submit-once']
 if any('\0' in x or len(x.encode())+1>=131072 for x in cmd):raise RuntimeError('ARGV_BOUND')
 # Accepted bounded controller, run as dedicated foreground exclusive child reaper.
 control=CONTROL.read_bytes()
 if sha(control)!=CONTROL_SHA:raise RuntimeError('CONTROL_HASH_MISMATCH')
 control_mod=importlib.util.module_from_spec(importlib.util.spec_from_loader('bounded_control',loader=None));exec(compile(control,str(CONTROL),'exec'),control_mod.__dict__)
 env=os.environ.copy();env['TSPS_LAUNCHER_SNAPSHOT_SHA256']=globals()['digest']
 try:
  closure=control_mod.run_group(cmd,cwd=str(ROOT),env=env,stdout_path=str(ROOT/'launch-stdout.txt'),stderr_path=str(ROOT/'launch-stderr.txt'),timeout_seconds=240,grace_seconds=2,output_limit=4194304,file_limit=None)
  for file in (ROOT/'launch-stdout.txt',ROOT/'launch-stderr.txt'):
   with file.open('rb') as f:os.fsync(f.fileno())
  with (ROOT/'launch-closure.json').open('x') as f:json.dump(closure,f,sort_keys=True,indent=2);f.flush();os.fsync(f.fileno())
  fd=os.open(str(ROOT),os.O_DIRECTORY)
  try:os.fsync(fd)
  finally:os.close(fd)
 except BaseException:
  print(json.dumps({'stage':'UNKNOWN_CONTROLLER_OR_LOG_FAILURE_RECONCILE_SAME_INTENT'}));raise
 unknown=closure['timed_out'] or closure['output_truncated'] or closure['drain_incomplete'] or closure['drain_errors'] or closure['group_remaining'] or closure['exit_code'] != 0
 print(json.dumps(dict(closure,stage='UNKNOWN_RECONCILE' if unknown else 'CHILD_RETURNED_RECONCILE_RECEIPT')))
if __name__=='__main__':main()
