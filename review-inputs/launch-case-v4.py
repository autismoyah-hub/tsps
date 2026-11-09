"""UNREVIEWED DRAFT. Launch verified auth wrapper snapshot and case entry exactly once."""
import base64,hashlib,importlib.util,json,os,pathlib,subprocess,sys
ROOT=pathlib.Path(__file__).resolve().parent
HELPER=pathlib.Path('/tmp/zogre-control/hf-author-control/004/files/authenticated-child-snapshot-command.py')
HELPER_SHA='00ff45d75be1eae3bc8f04995cfb8bcbe2bc5adc28c1162b843cfe88c87024f9'
WRAPPER_SHA='fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755'
sha=lambda b:hashlib.sha256(b).hexdigest()
def main():
 if sys.version_info[:3]!=(3,12,3) or sys.argv[1:]!=['--submit-once']:raise RuntimeError('EXACT_INTERPRETER_AND_EXPLICIT_ACTION_REQUIRED')
 helper=HELPER.read_bytes()
 if sha(helper)!=HELPER_SHA:raise RuntimeError('HELPER_HASH_MISMATCH')
 selection=json.loads((ROOT/'selection.json').read_bytes());entry=ROOT/'case-entry-v4.py';wrapper=ROOT/'auth-wrapper-v4.py'
 if globals().get('digest')!=selection.get('launcherSHA256'):raise RuntimeError('LAUNCHER_SNAPSHOT_MISMATCH')
 snapshot=wrapper.read_bytes()
 if sha(snapshot)!=WRAPPER_SHA or sha(entry.read_bytes())!=selection['entrySHA256']:raise RuntimeError('SNAPSHOT_MISMATCH')
 spec=importlib.util.spec_from_loader('accepted_snapshot_helper',loader=None);module=importlib.util.module_from_spec(spec);exec(compile(helper,str(HELPER),'exec'),module.__dict__)
 cmd=[sys.executable,'-c',module.CHILD_BOOTSTRAP,base64.b64encode(snapshot).decode('ascii'),WRAPPER_SHA,str(wrapper),'/tmp/zogre-hf-sdk/site-packages','--entry',str(entry),'--entry-sha256',selection['entrySHA256'],'--','--submit-once']
 if any('\0' in x or len(x.encode())+1>=131072 for x in cmd):raise RuntimeError('ARGV_BOUND')
 # A timeout is UNKNOWN, never a reason to retry. Child's local/remote intent remains authoritative.
 env=os.environ.copy();env['TSPS_LAUNCHER_SNAPSHOT_SHA256']=globals()['digest']
 try:
  result=subprocess.run(cmd,capture_output=True,timeout=240,check=False,env=env);out,err,rc=result.stdout,result.stderr,result.returncode
 except subprocess.TimeoutExpired as exc:
  out,err,rc=exc.stdout or b'',exc.stderr or b'',None
 for name,data in [('launch-stdout.txt',out),('launch-stderr.txt',err)]:
  with (ROOT/name).open('xb') as f:f.write(data);f.flush();__import__('os').fsync(f.fileno())
 print(json.dumps({'returncode':rc,'stdoutSHA256':sha(out),'stderrSHA256':sha(err),'timeout':rc is None}))
if __name__=='__main__':main()
