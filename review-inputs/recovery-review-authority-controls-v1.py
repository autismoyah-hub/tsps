"""Offline synthetic wiring controls for the source-only named-origin caller."""
import hashlib,json,pathlib,tempfile,types,unittest
from datetime import datetime,timedelta,timezone
from unittest import mock
P=pathlib.Path('/tmp/zogre-hf-case/recovery-review-authority-draft-v1.py');raw=P.read_bytes()
if hashlib.sha256(raw).hexdigest()!='1875d07533948005255691a2ebfd99e1afb7acbaee87df4afe99bf619141c1ed':raise RuntimeError('PIN')
m=types.ModuleType('authority_snapshot');exec(compile(raw,str(P),'exec'),m.__dict__)
class Tests(unittest.TestCase):
 def setUp(self):
  now=datetime.now(timezone.utc);self.now=now;self.receipt=b'synthetic receipt';self.rsha=hashlib.sha256(self.receipt).hexdigest()
  self.selection={'schema':1,'action':'COLLECT_EXISTING_ONCE','owner':m.OWNER,'reviewer':m.REVIEWER,'collector':m.OWNER,'jobId':m.JOB,'namespace':m.NAMESPACE,'source':m.SOURCE,'originalSelectionSHA256':m.ORIGINAL,'requestSHA256':m.REQUEST,'entrySHA256':'1'*64,'launcherSHA256':'2'*64,'bootstrapSHA256':'3'*64,'captureSHA256':'4'*64,'controllerSHA256':'565e79e713f6881cca014107e3a7d9bf502fc77dc8e3ead152baffed5a53d487','wrapperSHA256':'fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755','sdkApiSHA256':'3871f6516f2bf97b77cf4179daea929b30b4c97d7e87e9158fa742f46cbf4860','sdkJobsSHA256':'4f6e280e0f1938d3230625384b25acdb360aac2925fe23040a8525c0bc1306e2','bindingSHA256':'5'*64,'recordsSHA256':'6'*64,'readbackSHA256':'7'*64,'observerSHA256':'8'*64,'payloadSHA256':'220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4','planSHA256':'f8013c6ec9a9bc191db918dc8d93c2903cbb1991590887420b97a77cae2b3f9f','reviewReceiptSHA256':self.rsha,'selectedUTC':(now-timedelta(seconds=10)).isoformat()}
  self.marker={'role':'assistant','kind':'text','ts':(now-timedelta(seconds=5)).isoformat(),'text':'synthetic'}
  self.history=json.dumps({'turns':[{'role':'user','kind':'text','ts':(now-timedelta(seconds=20)).isoformat(),'text':'earlier'},self.marker]}).encode();self.hsha=hashlib.sha256(self.history).hexdigest()
 def test_expected_subject_and_window(self):
  self.selection['captureSHA256']=m.CAPTURE_SHA;raw=json.dumps(self.selection,sort_keys=True).encode();h=hashlib.sha256(raw).hexdigest();_,subject=m.expected_subject(raw,h)
  self.assertEqual(subject['jobId'],m.JOB);self.assertEqual(subject['recoverySelectionSHA256'],h)
  self.assertEqual(m.review_window(self.history,m.timestamp(self.selection['selectedUTC']),self.marker,self.now)['turns'],2)
  bad=json.dumps({'turns':[self.marker]}).encode()
  with self.assertRaises(RuntimeError):m.review_window(bad,m.timestamp(self.selection['selectedUTC']),self.marker,self.now)
  self.assertRaises(RuntimeError,m.expected_subject,raw,'0'*64)
 def test_actual_caller_wiring_with_inert_fake_helper(self):
  with tempfile.TemporaryDirectory(prefix='zogre-authority-fake-') as t:
   root=pathlib.Path(t);sentinel=root/'called';helper=root/'helper.py'
   code=('import json,pathlib\nCLI="/synthetic/cuttle"\nCOMMAND=(CLI,"--json","sessions","read","'+m.REVIEWER+'","--turns","1000")\n'
         'def select_turn(raw,subject,receipt): return json.loads(raw)["turns"][-1]\n'
         'def acquire(subject,receipt,attempt):\n pathlib.Path('+repr(str(sentinel))+').write_text("called")\n return '+repr(self.history)+','+repr(self.marker)+',{"closure":"FAKE"}\n')
   helper.write_text(code);fake_sha=hashlib.sha256(helper.read_bytes()).hexdigest();self.selection['captureSHA256']=fake_sha
   selection=json.dumps(self.selection,sort_keys=True).encode();h=hashlib.sha256(selection).hexdigest()
   with mock.patch.object(m,'CAPTURE',helper),mock.patch.object(m,'CAPTURE_SHA',fake_sha):
    with self.assertRaises(RuntimeError):m.validate_named_origin(selection,'0'*64,self.receipt,self.history,self.hsha,self.marker,'1'*32)
    self.assertFalse(sentinel.exists())
    result=m.validate_named_origin(selection,h,self.receipt,self.history,self.hsha,self.marker,'1'*32)
   self.assertTrue(sentinel.exists());self.assertEqual(result['stage'],'NAMED_TURN_MATCH_PENDING_OUTER_AND_INSTRUCTION_REVIEW')
if __name__=='__main__':unittest.main()
