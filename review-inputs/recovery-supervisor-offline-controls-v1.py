"""Whole composed OFFLINE supervisor controls. No provider/GAME calls."""
import base64,hashlib,json,os,pathlib,select,subprocess,sys,tempfile,threading,time,unittest
P=pathlib.Path('/tmp/zogre-hf-case/recovery-supervisor-offline-v2.py');RAW=P.read_bytes();PIN='890c9deb77802c8df5bbd9c43ee3fc55f9d0973e061f91f41e3a767375a34152'
if hashlib.sha256(RAW).hexdigest()!=PIN:raise RuntimeError('PIN')
BOOT="import base64,hashlib,sys; b=base64.b64decode(sys.argv[1],validate=True); h=sys.argv[2]; hashlib.sha256(b).hexdigest()==h or sys.exit(91); sys.argv=[sys.argv[0]]+sys.argv[3:];exec(compile(b,'verified-supervisor-fake','exec'),{'__name__':'__main__'})"
def argv(root,mode='ok',fault='none',sfault='none'):
 return [sys.executable,'-c',BOOT,base64.b64encode(RAW).decode(),PIN,'--root',root,'--mode',mode,'--outer-fault',fault,'--supervisor-fault',sfault]
def run(root,mode='ok',fault='none',sfault='none'):
 return subprocess.run(argv(root,mode,fault,sfault),capture_output=True,text=True,timeout=12)
class Tests(unittest.TestCase):
 def test_success_failure_timeout_and_injected_storage(self):
  for mode,fault in (('ok','none'),('nonzero','none'),('timeout','none'),('ok','guard'),('ok','intent'),('ok','output'),('ok','receipt')):
   with self.subTest(mode=mode,fault=fault),tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
    p=run(t,mode,fault);result=json.loads((pathlib.Path(t)/'supervisor-result.json').read_text())
    if (mode,fault)==('ok','none'):self.assertEqual((p.returncode,result['stage']),(0,'FAKE_VALIDATED_GROUP_ONLY'))
    else:self.assertEqual((p.returncode,result['stage']),(1,'FAKE_UNKNOWN_NO_RETRY'))
    self.assertEqual(result['detachedDescendants'],'UNKNOWN');self.assertNotEqual(run(t).returncode,0)
 def test_supervisor_receipt_fsync_failure(self):
  with tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
   p=run(t,sfault='receipt');self.assertNotEqual(p.returncode,0)
   # File may exist with complete-looking bytes, but sync failed: never accept it.
   self.assertTrue((pathlib.Path(t)/'zogre-composed-fake-inner'/'guard'/'outer-intent.json').exists());self.assertNotEqual(run(t).returncode,0)
 def test_competing_supervisors(self):
  with tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
   gate=threading.Barrier(2);results=[]
   def f():gate.wait(timeout=3);results.append(run(t).returncode)
   threads=[threading.Thread(target=f,daemon=True) for _ in range(2)]
   for x in threads:x.start()
   for x in threads:x.join(timeout=15)
   self.assertFalse(any(x.is_alive() for x in threads));self.assertEqual(len(results),2);self.assertLessEqual(sum(x==0 for x in results),1)
   self.assertTrue((pathlib.Path(t)/'zogre-composed-fake-inner').is_dir())
 def test_supervisor_killed_after_child_spawn(self):
  if not hasattr(os,'pidfd_open'):self.skipTest('pidfd unavailable')
  with tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
   supervisor=subprocess.Popen(argv(t,'death'),stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);outerfd=None
   try:
    deadline=time.monotonic()+5;outer=None
    while time.monotonic()<deadline:
     path=pathlib.Path(f'/proc/{supervisor.pid}/task/{supervisor.pid}/children')
     if not path.exists():break
     content=path.read_text().strip()
     if content:outer=int(content.split()[0]);break
     time.sleep(.02)
    self.assertIsNotNone(outer);outerfd=os.pidfd_open(outer)
    supervisor.kill();supervisor.wait(timeout=5)
    self.assertFalse((pathlib.Path(t)/'supervisor-result.json').exists())
    self.assertTrue(select.select([outerfd],[],[],8)[0],'fake outer did not naturally exit')
    self.assertNotEqual(run(t).returncode,0)
   finally:
    if supervisor.poll() is None:supervisor.kill();supervisor.wait(timeout=5)
    if outerfd is not None:os.close(outerfd)
if __name__=='__main__':unittest.main()
