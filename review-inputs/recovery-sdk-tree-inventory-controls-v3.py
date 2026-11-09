"""Offline fake-tree controls; no actual SDK read/import or provider call."""
import hashlib,os,pathlib,subprocess,sys,tempfile,types,unittest
from unittest import mock
p=pathlib.Path('/tmp/zogre-hf-case/recovery-sdk-tree-inventory-v2.py');raw=p.read_bytes()
if hashlib.sha256(raw).hexdigest()!='4fbf6ec2abdff1e49912e00482c859b0d55ff71a1a66ef3c0c98218d49af4afe':raise RuntimeError('PIN')
m=types.ModuleType('inventory_snapshot');exec(compile(raw,str(p),'exec'),m.__dict__)
class Tests(unittest.TestCase):
 def test_normal_error_and_mutation(self):
  with tempfile.TemporaryDirectory(prefix='sdk-inventory-fake-') as t:
   root=pathlib.Path(t);(root/'a').write_bytes(b'a');fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    self.assertEqual(m.scan(fd)[0]['sha256'],hashlib.sha256(b'a').hexdigest())
    with mock.patch.object(m.os,'scandir',side_effect=PermissionError()):
     with self.assertRaises(PermissionError):m.scan(fd)
    read=m.os.read;changed=[False]
    def mutate(f,n):
     b=read(f,n)
     if b and not changed[0]:(root/'a').rename(root/'old');(root/'a').write_bytes(b'a');changed[0]=True
     return b
    with mock.patch.object(m.os,'read',side_effect=mutate):
     with self.assertRaises(RuntimeError):m.scan(fd)
   finally:os.close(fd)
 def test_fifo_symlink_hardlink(self):
  with tempfile.TemporaryDirectory(prefix='sdk-inventory-fake-') as t:
   root=pathlib.Path(t);os.mkfifo(root/'pipe')
   code="import os,runpy,sys;d=runpy.run_path(sys.argv[1],run_name='testonly');fd=os.open(sys.argv[2],os.O_RDONLY|os.O_DIRECTORY);\ntry:d['scan'](fd)\nexcept (RuntimeError,OSError):sys.exit(0)\nsys.exit(1)"
   self.assertEqual(subprocess.run([sys.executable,'-c',code,str(p),str(root)],capture_output=True,timeout=3).returncode,0)
   (root/'pipe').unlink();(root/'link').symlink_to('/etc/passwd');fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    with self.assertRaises(RuntimeError):m.scan(fd)
    (root/'link').unlink();(root/'a').write_bytes(b'x');os.link(root/'a',root/'b')
    with self.assertRaises(RuntimeError):m.scan(fd)
   finally:os.close(fd)
 def test_entry_bound(self):
  with tempfile.TemporaryDirectory(prefix='sdk-inventory-fake-') as t:
   root=pathlib.Path(t)
   for n in ('a','b','c'):(root/n).write_bytes(b'x')
   fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    with mock.patch.object(m,'MAX_ENTRIES',2):
     with self.assertRaises(RuntimeError):m.scan(fd)
   finally:os.close(fd)
if __name__=='__main__':unittest.main()
