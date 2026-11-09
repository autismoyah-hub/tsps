"""Synthetic-only controls: never performs an actual SDK inventory refresh."""
import ast,hashlib,json,os,pathlib,shlex,stat,subprocess,sys,tempfile,types,unittest
from unittest import mock
ROOT=pathlib.Path('/tmp/zogre-hf-case');source=(ROOT/'recovery-inventory-refresh-runner-draft-v2.py').read_bytes();literal=(ROOT/'recovery-inventory-refresh-literal-v2.txt').read_bytes()
if hashlib.sha256(source).hexdigest()!='e53938a9f93ba25f1bb6e9b2933531710f08605a19115aa0c7cb766c9ff6c454' or hashlib.sha256(literal).hexdigest()!='92ee34cf31988fc8879ec2044564d42d925e54a16fbb062b08fe7142d1d4caee':raise RuntimeError('PIN')
tree=ast.parse(source);defs=[x for x in tree.body if isinstance(x,ast.FunctionDef) and x.name in ('put','read')]
if len(defs)!=2:raise RuntimeError('EXTRACTION')
ns={'os':os,'json':json,'stat':stat};exec(compile(ast.fix_missing_locations(ast.Module(body=defs,type_ignores=[])),'fake-inventory-receipts','exec'),ns)
class Tests(unittest.TestCase):
 def test_no_action_before_wrong_argument(self):
  args=shlex.split(literal.decode());self.assertEqual(args[-1],'--readonly-once');args[-1]='--not-selected'
  attempt=pathlib.Path('/home/ubuntu/.zogre-recovery-42689c/sdk-inventory-refresh-v2')
  self.assertFalse(attempt.exists())
  for optimized in (False,True):
   command=[args[0]]+(['-O'] if optimized else [])+args[1:]
   with self.subTest(optimized=optimized):self.assertNotEqual(subprocess.run(command,capture_output=True,timeout=5).returncode,0);self.assertFalse(attempt.exists())
 def test_fake_receipt_collision_and_sync_failure(self):
  with tempfile.TemporaryDirectory(prefix='zogre-inventory-fake-') as t:
   fd=os.open(t,os.O_RDONLY|os.O_DIRECTORY)
   try:
    ns['put'](fd,'x.json',{'stage':'SYNTHETIC'});self.assertEqual(json.loads(ns['read'](fd,'x.json',1024))['stage'],'SYNTHETIC')
    with self.assertRaises(FileExistsError):ns['put'](fd,'x.json',{})
    with mock.patch.object(os,'fsync',side_effect=OSError('synthetic')):
     with self.assertRaises(OSError):ns['put'](fd,'failed.json',{})
    self.assertTrue((pathlib.Path(t)/'failed.json').exists())
   finally:os.close(fd)
 def test_reject_symlink_and_device_before_open(self):
  with tempfile.TemporaryDirectory(prefix='zogre-inventory-fake-') as t:
   root=pathlib.Path(t);(root/'link').symlink_to('/etc/passwd');(root/'device').write_bytes(b'x');dfd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    with self.assertRaises(RuntimeError):ns['read'](dfd,'link',1024)
    realstat=os.stat;realopen=os.open;opened=[]
    def fake_stat(name,*a,**kw):
     v=realstat(name,*a,**kw)
     return types.SimpleNamespace(st_mode=stat.S_IFCHR|0o600,st_nlink=1,st_size=1) if name=='device' else v
    def tracked(name,*a,**kw):opened.append(name);return realopen(name,*a,**kw)
    with mock.patch.object(os,'stat',side_effect=fake_stat),mock.patch.object(os,'open',side_effect=tracked):
     with self.assertRaises(RuntimeError):ns['read'](dfd,'device',1024)
    self.assertNotIn('device',opened)
   finally:os.close(dfd)
if __name__=='__main__':unittest.main()
