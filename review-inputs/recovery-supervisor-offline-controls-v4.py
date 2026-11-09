"""Whole composed OFFLINE supervisor controls. No provider/GAME calls."""
import base64,copy,hashlib,json,os,pathlib,select,subprocess,sys,tempfile,threading,time,types,unittest
P=pathlib.Path('/tmp/zogre-hf-case/recovery-supervisor-offline-v4.py');RAW=P.read_bytes();PIN='ac5a54e0c9c4fb7afd5dfa5957454af9fdd6048cede056b1caa20e09f8ffcc39'
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
    p=run(t,mode,fault);result=json.loads((pathlib.Path(t)/'supervisor-once'/'supervisor-result.json').read_text())
    if (mode,fault)==('ok','none'):self.assertEqual((p.returncode,result['stage']),(0,'FAKE_CHECKED_REPORTED_GROUP_ONLY'))
    else:self.assertEqual((p.returncode,result['stage']),(1,'FAKE_UNKNOWN_NO_RETRY'))
    self.assertEqual(result['detachedDescendants'],'UNKNOWN');self.assertNotEqual(run(t).returncode,0)
 def test_contradictory_or_missing_projection(self):
  with tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
   self.assertEqual(run(t).returncode,0)
   guard=pathlib.Path(t)/'supervisor-once'/'zogre-composed-fake-inner'/'guard'
   intent=json.loads((guard/'outer-intent.json').read_text());result=json.loads((guard/'outer-result.json').read_text())
   mod=types.ModuleType('fake_supervisor_snapshot');exec(compile(RAW,str(P),'exec'),mod.__dict__)
   self.assertEqual(set(mod.validate_projection(intent,result)),{'stdout','stderr'})
   for key,bad in (('exit_code',False),('timed_out',True),('drain_incomplete',True),('output_truncated',True),('group_remaining',True),('drain_errors',['bad']),('group_state','unknown'),('detached_descendant_closure','complete')):
    altered=copy.deepcopy(result);altered['controllerObservation'][key]=bad
    with self.subTest(key=key),self.assertRaises(RuntimeError):mod.validate_projection(intent,altered)
   for key in ('intentPersisted','originalLabelsEquality','fakeSelectionSHA256','controllerObservation'):
    altered=copy.deepcopy(result);altered.pop(key)
    with self.subTest(missing=key),self.assertRaises(RuntimeError):mod.validate_projection(intent,altered)
   for key in ('originalSelectionSHA256','originalLabelsEquality','historicalSubmission'):
    altered=copy.deepcopy(intent);altered.pop(key)
    with self.subTest(intent_missing=key),self.assertRaises(RuntimeError):mod.validate_projection(altered,result)
 def test_supervisor_receipt_fsync_failure(self):
  with tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
   p=run(t,sfault='receipt');self.assertNotEqual(p.returncode,0)
   # File may exist with complete-looking bytes, but sync failed: never accept it.
   self.assertTrue((pathlib.Path(t)/'supervisor-once'/'zogre-composed-fake-inner'/'guard'/'outer-intent.json').exists());self.assertNotEqual(run(t).returncode,0)
 def test_competing_supervisors(self):
  with tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
   gate=threading.Barrier(2);results=[]
   def f():gate.wait(timeout=3);results.append(run(t).returncode)
   threads=[threading.Thread(target=f,daemon=True) for _ in range(2)]
   for x in threads:x.start()
   for x in threads:x.join(timeout=15)
   self.assertFalse(any(x.is_alive() for x in threads));self.assertEqual(len(results),2);self.assertEqual(sum(x==0 for x in results),1)
   guard=pathlib.Path(t)/'supervisor-once';self.assertTrue((guard/'zogre-composed-fake-inner').is_dir());a=json.loads((guard/'supervisor-intent.json').read_text());b=json.loads((guard/'supervisor-result.json').read_text());self.assertEqual(a['invocationId'],b['invocationId']);self.assertEqual(b['stage'],'FAKE_CHECKED_REPORTED_GROUP_ONLY')
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
    self.assertFalse((pathlib.Path(t)/'supervisor-once'/'supervisor-result.json').exists())
    self.assertTrue(select.select([outerfd],[],[],8)[0],'fake outer did not naturally exit')
    self.assertNotEqual(run(t).returncode,0)
   finally:
    if supervisor.poll() is None:supervisor.kill();supervisor.wait(timeout=5)
    if outerfd is not None:os.close(outerfd)
if __name__=='__main__':unittest.main()
