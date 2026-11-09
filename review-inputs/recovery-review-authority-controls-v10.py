"""Offline synthetic wiring controls for the source-only named-origin caller."""
import hashlib,json,pathlib,tempfile,types,unittest
from datetime import datetime,timedelta,timezone
from unittest import mock
P=pathlib.Path('/tmp/zogre-hf-case/recovery-review-authority-draft-v9.py');raw=P.read_bytes()
if hashlib.sha256(raw).hexdigest()!='8a5c61505d1a0568bec734c465b3dbcb38f29df80c1a84c15024225ae5a462ff':raise RuntimeError('PIN')
m=types.ModuleType('authority_snapshot');m.__dict__['__verified_authority_sha256__']=hashlib.sha256(raw).hexdigest();exec(compile(raw,str(P),'exec'),m.__dict__)
class Tests(unittest.TestCase):
 def setUp(self):
  now=datetime.now(timezone.utc);self.now=now;self.receipt=b'synthetic receipt';self.rsha=hashlib.sha256(self.receipt).hexdigest()
  self.selection={'schema':1,'action':'COLLECT_EXISTING_ONCE','owner':m.OWNER,'reviewer':m.REVIEWER,'collector':m.OWNER,'jobId':m.JOB,'namespace':m.NAMESPACE,'source':m.SOURCE,'originalSelectionSHA256':m.ORIGINAL,'requestSHA256':m.REQUEST,'entrySHA256':'1'*64,'launcherSHA256':'2'*64,'bootstrapSHA256':'3'*64,'authoritySHA256':hashlib.sha256(raw).hexdigest(),'captureSHA256':'4'*64,'controllerSHA256':'565e79e713f6881cca014107e3a7d9bf502fc77dc8e3ead152baffed5a53d487','wrapperSHA256':'fe33c0f548236836256fae770ffb4dedf42c8ac66b71694c7940b5dafcb2f755','sdkApiSHA256':'3871f6516f2bf97b77cf4179daea929b30b4c97d7e87e9158fa742f46cbf4860','sdkJobsSHA256':'4f6e280e0f1938d3230625384b25acdb360aac2925fe23040a8525c0bc1306e2','bindingSHA256':'5'*64,'recordsSHA256':'6'*64,'readbackSHA256':'7'*64,'observerSHA256':'8'*64,'payloadSHA256':'220ef3554ec1bf8951c22dc8e50b849feeeaaa5b0b0b06f664ea7b393cb1dde4','planSHA256':'f8013c6ec9a9bc191db918dc8d93c2903cbb1991590887420b97a77cae2b3f9f','inventorySHA256':'9'*64,'reviewReceiptSHA256':self.rsha,'selectedUTC':(now-timedelta(seconds=10)).isoformat()}
  self.marker={'role':'assistant','kind':'text','ts':(now-timedelta(seconds=5)).isoformat(),'text':'synthetic'}
  self.history=json.dumps({'turns':[{'role':'user','kind':'text','ts':(now-timedelta(seconds=20)).isoformat(),'text':'earlier'},self.marker]}).encode();self.hsha=hashlib.sha256(self.history).hexdigest()
 def set_receipt(self):
  names=('jobId','source','originalSelectionSHA256','requestSHA256','entrySHA256','launcherSHA256','bootstrapSHA256','authoritySHA256','captureSHA256','controllerSHA256','wrapperSHA256','sdkApiSHA256','sdkJobsSHA256','bindingSHA256','recordsSHA256','readbackSHA256','observerSHA256','payloadSHA256','planSHA256','inventorySHA256')
  receipt={name:self.selection[name] for name in names};receipt.update(schema=1,kind='ZOGRE_PRESELECTION_SOURCE_RECEIPT',reviewer=m.REVIEWER,executor=m.OWNER,receiptNonce='a'*32)
  self.receipt=json.dumps(receipt,sort_keys=True,separators=(',',':')).encode();self.rsha=hashlib.sha256(self.receipt).hexdigest();self.selection['reviewReceiptSHA256']=self.rsha
 def test_expected_subject_and_window(self):
  self.selection['captureSHA256']=m.CAPTURE_SHA;self.set_receipt();raw=json.dumps(self.selection,sort_keys=True).encode();h=hashlib.sha256(raw).hexdigest();_,subject=m.expected_subject(raw,h)
  self.assertEqual(subject['jobId'],m.JOB);self.assertEqual(subject['recoverySelectionSHA256'],h)
  self.assertEqual(m.validate_preselection_receipt(self.receipt,self.selection)['kind'],'ZOGRE_PRESELECTION_SOURCE_RECEIPT')
  self.assertNotIn('recoverySelectionSHA256',json.loads(self.receipt));self.assertNotIn('selectedUTC',json.loads(self.receipt))
  with self.assertRaises(RuntimeError):m.validate_preselection_receipt(self.receipt+b' ',self.selection)
  with self.assertRaises(ValueError):m.strict_json(b'{"schema":1,"schema":1}')
  self.assertEqual(m.review_window(self.history,m.timestamp(self.selection['selectedUTC']),self.marker,self.now)['turns'],2)
  bad=json.dumps({'turns':[self.marker]}).encode()
  with self.assertRaises(RuntimeError):m.review_window(bad,m.timestamp(self.selection['selectedUTC']),self.marker,self.now)
  self.assertRaises(RuntimeError,m.expected_subject,raw,'0'*64)
 def test_same_timestamp_revoke_duplicates_and_schema_boolean(self):
  self.selection['captureSHA256']=m.CAPTURE_SHA;self.set_receipt();selection=json.dumps(self.selection,sort_keys=True).encode();selected=m.timestamp(self.selection['selectedUTC'])
  earlier={'role':'user','kind':'text','ts':(self.now-timedelta(seconds=20)).isoformat(),'text':'earlier'}
  later={'role':'assistant','kind':'text','ts':self.marker['ts'],'text':'UGLUG_RECOVERY_REVOKE'}
  for turns in ([earlier,self.marker,later],[earlier,self.marker,self.marker],[earlier,self.marker,{'role':'assistant','kind':'text','ts':'bad','text':'x'}]):
   with self.assertRaises((RuntimeError,ValueError)):m.review_window(json.dumps({'turns':turns}).encode(),selected,self.marker,self.now)
  bad=dict(self.selection,schema=True);data=json.dumps(bad,sort_keys=True).encode()
  with self.assertRaises(RuntimeError):m.expected_subject(data,hashlib.sha256(data).hexdigest())
 def test_preselection_receipt_nonce_validation_and_binding(self):
  self.selection['captureSHA256']=m.CAPTURE_SHA;self.set_receipt();base=json.loads(self.receipt)
  for bad in (None,5,'','A'*32,'g'*32,'a'*31,'a'*33):
   altered=dict(base,receiptNonce=bad);raw=json.dumps(altered,sort_keys=True,separators=(',',':')).encode();selection=dict(self.selection,reviewReceiptSHA256=hashlib.sha256(raw).hexdigest())
   with self.subTest(nonce=bad),self.assertRaises(RuntimeError):m.validate_preselection_receipt(raw,selection)
  missing=dict(base);missing.pop('receiptNonce');raw=json.dumps(missing,sort_keys=True,separators=(',',':')).encode()
  with self.assertRaises(RuntimeError):m.validate_preselection_receipt(raw,dict(self.selection,reviewReceiptSHA256=hashlib.sha256(raw).hexdigest()))
  changed=dict(base,receiptNonce='b'*32);raw=json.dumps(changed,sort_keys=True,separators=(',',':')).encode();newhash=hashlib.sha256(raw).hexdigest()
  self.assertNotEqual(newhash,self.selection['reviewReceiptSHA256'])
  with self.assertRaises(RuntimeError):m.validate_preselection_receipt(raw,self.selection)
  m.validate_preselection_receipt(raw,dict(self.selection,reviewReceiptSHA256=newhash))
 def test_actual_caller_wiring_with_inert_fake_helper(self):
  with tempfile.TemporaryDirectory(prefix='zogre-authority-fake-') as t:
   root=pathlib.Path(t);sentinel=root/'called';helper=root/'helper.py'
   code=('import json,pathlib\nCLI="/synthetic/cuttle"\nCOMMAND=(CLI,"--json","sessions","read","'+m.REVIEWER+'","--turns","1000")\n'
         'def select_turn(raw,subject,receipt): return json.loads(raw)["turns"][-1]\n'
         'def acquire(subject,receipt,attempt):\n pathlib.Path('+repr(str(sentinel))+').write_text("called")\n return '+repr(self.history)+','+repr(self.marker)+',{"closure":"FAKE"}\n')
   helper.write_text(code);fake_sha=hashlib.sha256(helper.read_bytes()).hexdigest();self.selection['captureSHA256']=fake_sha
   fixed={'controllerSHA256':'/tmp/zogre-control/rsprox-linux/002/files/process_control.py','wrapperSHA256':'/tmp/zogre-hf-case/auth-wrapper-v4.py','sdkApiSHA256':'/tmp/zogre-hf-sdk/site-packages/huggingface_hub/hf_api.py','sdkJobsSHA256':'/tmp/zogre-hf-sdk/site-packages/huggingface_hub/_jobs_api.py','payloadSHA256':'/tmp/zogre-hf-case/native-payload.cjs','planSHA256':'/tmp/zogre-hf-case/pilot-selected-draft.json'}
   artifacts={k:pathlib.Path(v).read_bytes() for k,v in fixed.items()};artifacts['captureSHA256']=helper.read_bytes();artifacts['authoritySHA256']=raw
   for key in ('entrySHA256','launcherSHA256','bootstrapSHA256','bindingSHA256','recordsSHA256','readbackSHA256','observerSHA256','inventorySHA256'):
    artifacts[key]=('synthetic '+key).encode();self.selection[key]=hashlib.sha256(artifacts[key]).hexdigest()
   self.set_receipt();selection=json.dumps(self.selection,sort_keys=True).encode();h=hashlib.sha256(selection).hexdigest()
   with mock.patch.object(m,'CAPTURE',helper),mock.patch.object(m,'CAPTURE_SHA',fake_sha):
    with self.assertRaises(RuntimeError):m.validate_named_origin(selection,'0'*64,self.receipt,self.history,self.hsha,self.marker,artifacts,'1'*32)
    self.assertFalse(sentinel.exists())
    with mock.patch.dict(m.__dict__,{'__verified_authority_sha256__':'0'*64}):
     with self.assertRaises(RuntimeError):m.validate_named_origin(selection,h,self.receipt,self.history,self.hsha,self.marker,artifacts,'1'*32)
    self.assertFalse(sentinel.exists())
    for key in artifacts:
     changed=dict(artifacts);changed[key]=changed[key]+b'!'
     with self.subTest(changed=key),self.assertRaises(RuntimeError):m.validate_named_origin(selection,h,self.receipt,self.history,self.hsha,self.marker,changed,'1'*32)
     self.assertFalse(sentinel.exists())
    missing=dict(artifacts);missing.pop('entrySHA256')
    with self.assertRaises(RuntimeError):m.validate_named_origin(selection,h,self.receipt,self.history,self.hsha,self.marker,missing,'1'*32)
    extra=dict(artifacts,extra=b'')
    with self.assertRaises(RuntimeError):m.validate_named_origin(selection,h,self.receipt,self.history,self.hsha,self.marker,extra,'1'*32)
    self.assertFalse(sentinel.exists())
    result=m.validate_named_origin(selection,h,self.receipt,self.history,self.hsha,self.marker,artifacts,'1'*32)
   self.assertTrue(sentinel.exists());self.assertEqual(result['stage'],'NAMED_TURN_MATCH_PENDING_OUTER_AND_INSTRUCTION_REVIEW')
if __name__=='__main__':unittest.main()
