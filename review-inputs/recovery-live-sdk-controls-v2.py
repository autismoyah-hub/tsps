"""Offline inert SDK seam controls; provider methods are never called."""
import hashlib,pathlib,types,unittest
from unittest import mock
p=pathlib.Path('/tmp/zogre-hf-case/recovery-live-sdk-draft-v1.py');raw=p.read_bytes()
if hashlib.sha256(raw).hexdigest()!='3e37a79c4b42755075100d76099a77554b641011f1585a4fbe8da006bbe60875':raise RuntimeError('PIN')
m=types.ModuleType('sdk_snapshot');exec(compile(raw,str(p),'exec'),m.__dict__)
class Tests(unittest.TestCase):
 def test_both_real_entry_seams_refuse_before_adapter(self):
  api=mock.Mock();binder=mock.Mock();records=mock.Mock()
  with mock.patch.object(m,'sdk_identity',side_effect=AssertionError('must not import')) as sdk:
   with self.assertRaisesRegex(RuntimeError,'LIVE_AUTHORITY_AND_CUSTODY_NOT_IMPLEMENTED'):m.inspect_existing({},binder)
   with self.assertRaisesRegex(RuntimeError,'LIVE_AUTHORITY_AND_CUSTODY_NOT_IMPLEMENTED'):m.fetch_existing(api,records,0)
   sdk.assert_not_called();api.assert_not_called();binder.validate.assert_not_called();records.retain_records.assert_not_called()
 def test_explicit_fake_positive_and_mismatch(self):
  info=types.SimpleNamespace(id=m.JOB,owner=types.SimpleNamespace(name=m.NAMESPACE),docker_image='image',command=['node'],flavor='cpu',labels={'case':'fake'},retry=0,status=types.SimpleNamespace(stage='ERROR'))
  api=mock.Mock();api.whoami.return_value={'name':m.NAMESPACE};api.inspect_job.return_value=info
  hub=types.SimpleNamespace(HfApi=lambda:api);binder=mock.Mock();binder.validate.return_value='BOUND_TERMINAL'
  with mock.patch.object(m,'_not_ready',return_value=None),mock.patch.object(m,'sdk_identity',return_value=hub),mock.patch.dict(m.__dict__,{'__verified_wrapper_sha256__':m.WRAPPER_SHA}):
   out,observed=m.inspect_existing({'fake':b'x'},binder)
   self.assertIs(out,api);self.assertEqual(observed['stage'],'ERROR');api.inspect_job.assert_called_once_with(job_id=m.JOB,namespace=m.NAMESPACE)
   api.reset_mock();api.whoami.return_value={'name':'wrong'}
   with self.assertRaises(RuntimeError):m.inspect_existing({},binder)
   api.inspect_job.assert_not_called()
   fake_stream=iter(['synthetic']);api.fetch_job_logs.return_value=fake_stream;retainer=mock.Mock();retainer.retain_records.return_value={'outcome':'UNKNOWN'}
   self.assertEqual(m.fetch_existing(api,retainer,99),{'outcome':'UNKNOWN'});api.fetch_job_logs.assert_called_once_with(job_id=m.JOB,namespace=m.NAMESPACE,follow=False)
if __name__=='__main__':unittest.main()
