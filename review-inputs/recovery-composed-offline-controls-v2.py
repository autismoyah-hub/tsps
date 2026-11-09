"""Real-filesystem, fake-child controls; no GAME or provider access."""
import base64,hashlib,json,os,pathlib,select,subprocess,sys,tempfile,threading,time,unittest
P=pathlib.Path('/tmp/zogre-hf-case/recovery-composed-offline-v2.py');RAW=P.read_bytes();PIN='a5ae72eb4b8c7b5ffa934e56de95a080bcd31718cba6ff3efe30b8af670af7e2'
if hashlib.sha256(RAW).hexdigest()!=PIN:raise RuntimeError('PIN_MISMATCH')
BOOT="import base64,hashlib,sys; b=base64.b64decode(sys.argv[1],validate=True); h=sys.argv[2]; hashlib.sha256(b).hexdigest()==h or sys.exit(91); sys.argv=[sys.argv[0]]+sys.argv[3:];exec(compile(b,'verified-composed-fake','exec'),{'__name__':'__main__'})"
def argv(root,mode='ok',fault='none'):
 return [sys.executable,'-c',BOOT,base64.b64encode(RAW).decode(),PIN,'--root',root,'--mode',mode,'--fault',fault]
def run(root,mode='ok',fault='none'):
 return subprocess.run(argv(root,mode,fault),capture_output=True,text=True,timeout=12)
class Tests(unittest.TestCase):
 def test_complete_nonzero_timeout_and_duplicates(self):
  for mode in ('ok','nonzero','timeout'):
   with self.subTest(mode=mode),tempfile.TemporaryDirectory(prefix='zogre-composed-fake-') as t:
    p=run(t,mode);guard=pathlib.Path(t)/'guard';intent=json.loads((guard/'outer-intent.json').read_text());receipt=json.loads((guard/'outer-result.json').read_text())
    self.assertEqual(intent['stage'],'FAKE_PENDING_NO_RETRY');self.assertEqual(receipt['originalSubmission'],'UNKNOWN');self.assertEqual(receipt['detachedDescendants'],'UNKNOWN')
    if mode=='ok':
     self.assertEqual((p.returncode,receipt['stage']),(0,'FAKE_GROUP_ONLY_UNREVIEWED'));self.assertEqual(receipt['fakeNativeObservations']['interpretation'],'UNKNOWN_PENDING_RAW_REVIEW');self.assertEqual(receipt['fakeNativeObservations']['stageStartCounts'],{'build':1})
     for name,entry in receipt['outputSnapshots'].items():
      data=(guard/name).read_bytes();self.assertEqual((entry['bytes'],entry['sha256']),(len(data),hashlib.sha256(data).hexdigest()))
    else:self.assertEqual((p.returncode,receipt['stage'],receipt['outputSnapshots']),(1,'FAKE_UNKNOWN_NO_RETRY',{}))
    self.assertNotEqual(run(t).returncode,0)
 def test_guard_intent_output_receipt_failures(self):
  for fault in ('guard','intent','output','receipt'):
   with self.subTest(fault=fault),tempfile.TemporaryDirectory(prefix='zogre-composed-fake-') as t:
    p=run(t,fault=fault);root=pathlib.Path(t);self.assertNotEqual(p.returncode,0)
    if fault=='guard':self.assertFalse((root/'guard').exists())
    else:
     guard=root/'guard';self.assertTrue(guard.is_dir());self.assertNotEqual(run(t).returncode,0)
     if fault!='receipt':self.assertEqual(json.loads((guard/'outer-result.json').read_text())['stage'],'FAKE_UNKNOWN_NO_RETRY')
     else:self.assertTrue((guard/'outer-intent.json').exists()) # result may be partial or unsynced; never interpret it
 def test_concurrent_callers(self):
  with tempfile.TemporaryDirectory(prefix='zogre-composed-fake-') as t:
   gate=threading.Barrier(2);out=[]
   def f():gate.wait(timeout=3);out.append(run(t).returncode)
   threads=[threading.Thread(target=f,daemon=True) for _ in range(2)]
   for x in threads:x.start()
   for x in threads:x.join(timeout=12)
   self.assertFalse(any(x.is_alive() for x in threads));self.assertEqual(sum(x==0 for x in out),1);self.assertEqual(len(out),2)
 def test_controller_death_after_spawn(self):
  if not hasattr(os,'pidfd_open'):self.skipTest('pidfd unavailable')
  with tempfile.TemporaryDirectory(prefix='zogre-composed-fake-') as t:
   proc=subprocess.Popen(argv(t,'death'),stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True);pidfd=None
   try:
    deadline=time.monotonic()+5;child=None
    while time.monotonic()<deadline:
     path=pathlib.Path(f'/proc/{proc.pid}/task/{proc.pid}/children')
     if not path.exists():break
     text=path.read_text().strip()
     if text:child=int(text.split()[0]);break
     time.sleep(.02)
    self.assertIsNotNone(child,'fake child was not observed');pidfd=os.pidfd_open(child)
    proc.kill();proc.communicate(timeout=5)
    guard=pathlib.Path(t)/'guard';self.assertTrue((guard/'outer-intent.json').exists());self.assertFalse((guard/'outer-result.json').exists());self.assertNotEqual(run(t).returncode,0)
    self.assertTrue(select.select([pidfd],[],[],5)[0],'fake child did not naturally exit within bound')
   finally:
    if proc.poll() is None:proc.kill();proc.communicate(timeout=5)
    if pidfd is not None:os.close(pidfd)
if __name__=='__main__':unittest.main()
