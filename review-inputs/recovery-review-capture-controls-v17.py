"""Synthetic non-GAME named-reviewer selector controls; no session calls."""
import hashlib,json,pathlib,types,unittest,tempfile,subprocess,sys
from unittest import mock
p=pathlib.Path('/tmp/zogre-hf-case/recovery-review-capture-v14.py');raw=p.read_bytes()
if hashlib.sha256(raw).hexdigest()!='6bf3309b38275f4aa725888d307ef18b1a3fb77dcc4800fa90abea1b5f703475':raise RuntimeError('PIN_MISMATCH')
m=types.ModuleType('review_snapshot');exec(compile(raw,str(p),'exec'),m.__dict__)
class Controls(unittest.TestCase):
 def setUp(self):
  self.subject={'case':'zogre-existing-job','job':'6af14d46e6320c66fad7d183','action':'COLLECTION_ONLY_REVIEW_DRAFT'};self.sha='a'*64
  self.marker={'type':'TSPS_DIRECT_REVIEW','verdict':'EXACT_INPUT_PROFILE_GO','reviewer':m.REVIEWER,'executor':m.EXECUTOR,'subject':self.subject,'receiptSHA256':self.sha}
  self.turn={'role':'assistant','kind':'text','ts':'2026-11-09T00:00:01Z','text':json.dumps(self.marker)}
 def raw(self,turns):return json.dumps({'turns':turns}).encode()
 def test_exact_one_and_forged_envelopes(self):
  self.assertEqual(m.select_turn(self.raw([self.turn]),self.subject,self.sha),self.turn)
  for role,kind in (('user','text'),('assistant','tool'),('user','tool')):
   with self.subTest(role=role,kind=kind):
    bad=dict(self.turn,role=role,kind=kind)
    with self.assertRaises(ValueError):m.select_turn(self.raw([bad]),self.subject,self.sha)
  duplicate_marker=dict(self.turn,text=json.dumps(self.marker)[:-1]+',"type":"TSPS_DIRECT_REVIEW"}')
  with self.assertRaises(ValueError):m.select_turn(self.raw([duplicate_marker]),self.subject,self.sha)
  with self.assertRaises(ValueError):m.select_turn(self.raw([self.turn,duplicate_marker]),self.subject,self.sha)
  unrelated=dict(self.turn,text='unrelated prose')
  self.assertEqual(m.select_turn(self.raw([self.turn,unrelated]),self.subject,self.sha),self.turn)
  conflicting=dict(self.marker,verdict='HOLD');conflicting_turn=dict(self.turn,text=json.dumps(conflicting))
  with self.assertRaises(ValueError):m.select_turn(self.raw([self.turn,conflicting_turn]),self.subject,self.sha)
  other=dict(conflicting,receiptSHA256='b'*64);other_turn=dict(self.turn,text=json.dumps(other))
  self.assertEqual(m.select_turn(self.raw([self.turn,other_turn]),self.subject,self.sha),self.turn)
  ambiguous=dict(self.turn,text='{\"receiptSHA256\":\"\\u0061...')
  with self.assertRaises(ValueError):m.select_turn(self.raw([self.turn,ambiguous]),self.subject,self.sha)
  with self.assertRaises(ValueError):m.select_turn(b'{"turns":[],"turns":[]}',self.subject,self.sha)
  forged=dict(self.turn,text='[From Cuttle session] '+json.dumps(self.marker))
  with self.assertRaises(ValueError):m.select_turn(self.raw([forged]),self.subject,self.sha)
  for turns in ([],[self.turn,self.turn],[dict(self.turn,ts='')],[dict(self.turn,ts='not-a-date')],[dict(self.turn,ts='2026-11-09T00:00:01')]):
   with self.assertRaises(ValueError):m.select_turn(self.raw(turns),self.subject,self.sha)
 def test_subject_receipt_and_named_command(self):
  for subject,sha in ((dict(self.subject,job='wrong'),self.sha),(self.subject,'b'*64)):
   with self.assertRaises(ValueError):m.select_turn(self.raw([self.turn]),subject,sha)
  calls=[]
  def fake(args,**kwargs):
   calls.append((args,kwargs));data=self.raw([self.turn]);pathlib.Path(kwargs['stdout_path']).write_bytes(data);pathlib.Path(kwargs['stderr_path']).write_bytes(b'')
   return {'exit_code':0,'timed_out':False,'output_truncated':False,'drain_incomplete':False,'drain_errors':[],'group_remaining':False,'group_state':'absent_after_reap_observed','detached_descendant_closure':'unknown','output_bytes':{'stdout':len(data),'stderr':0}}
  with tempfile.TemporaryDirectory(prefix='zogre-capture-fake-') as t:
   original=m._bounded_read
   with mock.patch.object(m,'SCRATCH',pathlib.Path(t)),mock.patch.object(m,'_bounded_read',side_effect=lambda path,attempt,subject,receipt:original(path,attempt,subject,receipt,runner=fake)):
    raw,turn,proof=m.acquire(self.subject,self.sha,'1'*32)
   self.assertEqual(turn,self.turn);self.assertEqual(calls[0][0],[m.CLI,'--json','sessions','read',m.REVIEWER,'--turns','1000']);self.assertEqual(calls[0][1]['timeout_seconds'],40);self.assertEqual(calls[0][1]['env']['PATH'],'/usr/bin:/bin');self.assertNotIn('NODE_OPTIONS',calls[0][1]['env'])
   self.assertEqual(raw,self.raw([self.turn]));self.assertTrue(proof['captureDirectory'].startswith('review-capture-'));self.assertEqual(proof['closure']['exit_code'],0)
   attempt=pathlib.Path(t)/proof['captureDirectory'];intent=json.loads((attempt/'capture-intent.json').read_text());result=json.loads((attempt/'capture-result.json').read_text())
   self.assertEqual((intent['stage'],result['stage']),('PENDING_UNKNOWN','CAPTURE_BYTES_UNREVIEWED'));self.assertEqual(result['stdoutSHA256'],hashlib.sha256(raw).hexdigest())
   def at_limit(args,**kwargs):
    return {'exit_code':0,'timed_out':False,'output_truncated':False,'drain_incomplete':False,'drain_errors':[],'group_remaining':False,'group_state':'absent_after_reap_observed','detached_descendant_closure':'unknown','output_bytes':{'stdout':m.MAX_CAPTURE,'stderr':0}}
   with self.assertRaises(ValueError):original(pathlib.Path(t),'2'*32,'a'*64,'a'*64,runner=at_limit)
   def timeout(args,**kwargs):raise subprocess.TimeoutExpired(args,40)
   with self.assertRaises(subprocess.TimeoutExpired):original(pathlib.Path(t),'3'*32,'a'*64,'a'*64,runner=timeout)
   failed=pathlib.Path(t)/('review-capture-'+'3'*32)
   self.assertTrue((failed/'capture-intent.json').exists());self.assertFalse((failed/'capture-result.json').exists())
   with self.assertRaises(FileExistsError):original(pathlib.Path(t),'3'*32,'a'*64,'a'*64,runner=fake)
   def symlinked(args,**kwargs):
    out=pathlib.Path(kwargs['stdout_path']);out.symlink_to('capture-intent.json');pathlib.Path(kwargs['stderr_path']).write_bytes(b'')
    size=(out.parent/'capture-intent.json').stat().st_size
    return {'exit_code':0,'timed_out':False,'output_truncated':False,'drain_incomplete':False,'drain_errors':[],'group_remaining':False,'group_state':'absent_after_reap_observed','detached_descendant_closure':'unknown','output_bytes':{'stdout':size,'stderr':0}}
   with self.assertRaises(OSError):original(pathlib.Path(t),'8'*32,'a'*64,'a'*64,runner=symlinked)
  with mock.patch.object(m,'COMMAND',(m.CLI,'--json','sessions','read','wrong','--turns','1000')):
   with self.assertRaises(ValueError):m.acquire(self.subject,self.sha,'4'*32)
  for broken in (b'{', b'{}', b'{"turns":null}'):
   with self.assertRaises(ValueError):m.select_turn(broken,self.subject,self.sha)
 def test_malformed_controller_outcomes(self):
  valid={'exit_code':0,'timed_out':False,'output_truncated':False,'drain_incomplete':False,'drain_errors':[],'group_remaining':False,'group_state':'absent_after_reap_observed','detached_descendant_closure':'unknown','output_bytes':{'stdout':1,'stderr':0}}
  for key,value in (('exit_code',False),('exit_code',None),('timed_out',True),('output_truncated',True),('drain_incomplete',True),('drain_errors',['x']),('group_remaining',True),('group_state','unknown'),('detached_descendant_closure','closed')):
   with self.subTest(key=key,value=value),tempfile.TemporaryDirectory(prefix='zogre-capture-fake-') as t:
    def fake(args,**kwargs):
     x=dict(valid);x[key]=value;return x
    with self.assertRaises(ValueError):m._bounded_read(pathlib.Path(t),'5'*32,'a'*64,'a'*64,runner=fake)
 def test_real_inert_subprocess_size_limit(self):
  with tempfile.TemporaryDirectory(prefix='zogre-capture-fake-') as t:
   with mock.patch.object(m,'MAX_CAPTURE',1024),mock.patch.object(m,'COMMAND',(sys.executable,'-c','print("small")')):
    self.assertEqual(m._bounded_read(pathlib.Path(t),'6'*32,'a'*64,'a'*64)[0],b'small\n')
   with mock.patch.object(m,'MAX_CAPTURE',1024),mock.patch.object(m,'COMMAND',(sys.executable,'-c','print("x"*100000)')):
    with self.assertRaises(ValueError):m._bounded_read(pathlib.Path(t),'7'*32,'a'*64,'a'*64)
if __name__=='__main__':unittest.main()
