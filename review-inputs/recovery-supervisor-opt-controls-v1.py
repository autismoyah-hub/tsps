"""Offline actual fake-supervisor invocation under normal and optimized Python."""
import base64,hashlib,json,pathlib,subprocess,sys,tempfile,unittest
P=pathlib.Path('/tmp/zogre-hf-case/recovery-supervisor-offline-v4.py');RAW=P.read_bytes();PIN='ac5a54e0c9c4fb7afd5dfa5957454af9fdd6048cede056b1caa20e09f8ffcc39'
if hashlib.sha256(RAW).hexdigest()!=PIN:raise RuntimeError('PIN_MISMATCH')
BOOT="import base64,hashlib,sys; b=base64.b64decode(sys.argv[1],validate=True); h=sys.argv[2]; hashlib.sha256(b).hexdigest()==h or sys.exit(91); sys.argv=[sys.argv[0]]+sys.argv[3:];exec(compile(b,'verified-supervisor-fake','exec'),{'__name__':'__main__'})"
def run(root,opt):
 args=[sys.executable]+(['-O'] if opt else [])+['-c',BOOT,base64.b64encode(RAW).decode(),PIN,'--root',root,'--mode','ok']
 return subprocess.run(args,capture_output=True,text=True,timeout=12)
class Controls(unittest.TestCase):
 def test_positive_and_pre_adapter_refusal(self):
  for optimized in (False,True):
   with self.subTest(optimized=optimized),tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
    child=run(t,optimized);receipt=json.loads((pathlib.Path(t)/'supervisor-once'/'supervisor-result.json').read_text());self.assertEqual(child.returncode,0);self.assertEqual(receipt['stage'],'FAKE_CHECKED_REPORTED_GROUP_ONLY')
   with self.subTest(optimized=optimized,case='prior artifact'),tempfile.TemporaryDirectory(prefix='zogre-supervised-fake-') as t:
    (pathlib.Path(t)/'prior').touch();child=run(t,optimized);self.assertNotEqual(child.returncode,0);self.assertFalse((pathlib.Path(t)/'supervisor-once').exists());self.assertFalse((pathlib.Path(t)/'zogre-composed-fake-inner').exists())
if __name__=='__main__':unittest.main()
