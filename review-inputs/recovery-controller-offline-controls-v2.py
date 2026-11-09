"""Whole-path synthetic controls for OFFLINE-ONLY recovery design."""
import copy, hashlib, json, os, pathlib, subprocess, sys, tempfile, threading, types, unittest
from datetime import datetime, timezone
from unittest import mock
ROOT=pathlib.Path('/tmp/zogre-hf-case');p=ROOT/'recovery-controller-offline-v1.py';raw=p.read_bytes()
if hashlib.sha256(raw).hexdigest()!='449c80b72dc2e48d522f2b727891b1a858088f6fdb4b1f310590c696a0bc7efd':raise RuntimeError('PIN')
c=types.ModuleType('control');exec(compile(raw,str(p),'exec'),c.__dict__);modules=c.snapshots()
R={k:(ROOT/n).read_bytes() for k,n in {'return':'provider-return.json','unknown':'submission-unknown.json','intent':'case-intent-v16.json','selection':'selection.json'}.items()}
request=json.loads(R['intent'])['request'];b=modules['binding'];a=modules['authority']
selected=json.dumps({'owner':a.OWNER,'reviewer':a.REVIEWER,'collector':a.OWNER,'jobId':a.JOB,'action':'COLLECT_EXISTING_ONCE','selectedUTC':'2026-11-09T01:00:00Z'},sort_keys=True).encode();digest=hashlib.sha256(selected).hexdigest();now=datetime(2026,11,9,1,0,6,tzinfo=timezone.utc)
marker=f'UGLUG_RECOVERY_GO owner={a.OWNER} reviewer={a.REVIEWER} job={a.JOB} selection={digest}'
evidence={'authenticated':True,'completeWindow':True,'checkedUTC':'2026-11-09T01:00:05Z','knownHold':False,'knownRevocation':False,'reviewWithdrawn':False,'actions':[{'session':a.REVIEWER,'role':'assistant','kind':'tool','recipient':a.OWNER,'text':marker,'toolId':'id','timestamp':'2026-11-09T01:00:01Z'}],'results':[{'session':a.REVIEWER,'toolId':'id','timestamp':'2026-11-09T01:00:02Z','retainedCliResponse':{'verifiedProvenance':True,'exitCode':0,'stdout':f'sent input to {a.OWNER}\n'}}],'deliveries':[{'session':a.OWNER,'senderSession':a.REVIEWER,'role':'user','text':marker,'timestamp':'2026-11-09T01:00:03Z'}]}
observed={'id':b.JOB,'owner':b.NAMESPACE,'image':request['image'],'command':request['command'],'flavor':request['flavor'],'retry':0,'labels':dict(request['labels'],name=request['name']),'stage':'ERROR'}
class Fake:
 kind='OFFLINE_FAKE_ONLY'
 def __init__(self,mode='ok'):self.mode=mode;self.unknown=[];self.fetches=0
 def authority(self):return copy.deepcopy(evidence)
 def claim(self):return b.CLAIM
 def inspect(self):return copy.deepcopy(observed)
 def before_fetch(self):
  if self.mode=='crash':os._exit(42)
 def fetch(self):
  self.fetches+=1;yield '{"event":"stage-start","name":"build"}'
  if self.mode=='partial':raise TimeoutError()
 def closure(self):return 'FAKE_UNVERIFIED'
 def record_unknown(self,record):self.unknown.append(record)
class Tests(unittest.TestCase):
 def test_complete_and_duplicate(self):
  with tempfile.TemporaryDirectory(prefix='zogre-fake-recovery-') as t:
   fake=Fake();r=c.run_fake_only(fake,t,R,selected,now,modules);self.assertEqual(r['stage'],'FAKE_CONTROL_ONLY');self.assertEqual(fake.fetches,1)
   with self.assertRaises(RuntimeError):c.run_fake_only(fake,t,R,selected,now,modules)
   self.assertEqual(fake.fetches,1)
 def test_partial_preserved_and_no_retry(self):
  with tempfile.TemporaryDirectory(prefix='zogre-fake-recovery-') as t:
   fake=Fake('partial')
   with self.assertRaises(RuntimeError):c.run_fake_only(fake,t,R,selected,now,modules)
   work=pathlib.Path(t)/modules['once'].GUARD;result=json.loads((work/'records-result.json').read_text())
   self.assertEqual((result['outcome'],result['reason'],result['records']),('UNKNOWN','TIMEOUT',1));self.assertEqual(fake.unknown[0]['stage'],'UNKNOWN_NO_RETRY')
   with self.assertRaises(RuntimeError):c.run_fake_only(fake,t,R,selected,now,modules)
   self.assertEqual(fake.fetches,1)
 def test_fsync_failure_and_bad_authority(self):
  with tempfile.TemporaryDirectory(prefix='zogre-fake-recovery-') as t:
   fake=Fake()
   with mock.patch.object(c.os,'fsync',side_effect=OSError()):
    with self.assertRaises(OSError):c.run_fake_only(fake,t,R,selected,now,modules)
   self.assertTrue((pathlib.Path(t)/modules['once'].GUARD).exists());self.assertEqual(fake.fetches,0)
   with self.assertRaises(RuntimeError):c.run_fake_only(fake,t,R,selected,now,modules)
  with tempfile.TemporaryDirectory(prefix='zogre-fake-recovery-') as t:
   fake=Fake();fake.authority=lambda:dict(evidence,completeWindow=False)
   with self.assertRaises(RuntimeError):c.run_fake_only(fake,t,R,selected,now,modules)
   self.assertFalse((pathlib.Path(t)/modules['once'].GUARD).exists());self.assertEqual(fake.fetches,0)
 def test_receipt_failure_after_fetch(self):
  with tempfile.TemporaryDirectory(prefix='zogre-fake-recovery-') as t:
   fake=Fake();real=c.durable
   def fail_receipt(path,document):
    if path.name=='records-result.json':raise OSError('synthetic disk failure')
    return real(path,document)
   with mock.patch.object(c,'durable',side_effect=fail_receipt):
    with self.assertRaises(OSError):c.run_fake_only(fake,t,R,selected,now,modules)
   self.assertEqual(fake.fetches,1);self.assertEqual(fake.unknown[0]['stage'],'UNKNOWN_NO_RETRY')
   with self.assertRaises(RuntimeError):c.run_fake_only(fake,t,R,selected,now,modules)
   self.assertEqual(fake.fetches,1)
 def test_competing_callers(self):
  with tempfile.TemporaryDirectory(prefix='zogre-fake-recovery-') as t:
   gate=threading.Barrier(2);outcomes=[];fakes=[Fake(),Fake()]
   def run(fake):
    try:gate.wait(timeout=2);c.run_fake_only(fake,t,R,selected,now,modules);outcomes.append('finished')
    except Exception:outcomes.append('refused')
   threads=[threading.Thread(target=run,args=(fake,),daemon=True) for fake in fakes]
   for thread in threads:thread.start()
   for thread in threads:thread.join(timeout=5)
   self.assertFalse(any(thread.is_alive() for thread in threads));self.assertCountEqual(outcomes,['finished','refused']);self.assertEqual(sum(x.fetches for x in fakes),1)
 def test_crash_retains_guard(self):
  with tempfile.TemporaryDirectory(prefix='zogre-fake-recovery-') as t:
   child=subprocess.run([sys.executable,__file__,'--crash',t],capture_output=True,timeout=10)
   self.assertEqual(child.returncode,42);work=pathlib.Path(t)/modules['once'].GUARD;self.assertTrue((work/'collection-intent.json').exists())
   fake=Fake()
   with self.assertRaises(RuntimeError):c.run_fake_only(fake,t,R,selected,now,modules)
   self.assertEqual(fake.fetches,0)
if __name__=='__main__':
 if len(sys.argv)==3 and sys.argv[1]=='--crash':c.run_fake_only(Fake('crash'),sys.argv[2],R,selected,now,modules)
 else:unittest.main()
