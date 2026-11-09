"""Synthetic tiny trees only; never imports SDK or calls provider."""
import hashlib,os,pathlib,tempfile,types,unittest,subprocess,sys
from unittest import mock
p=pathlib.Path('/tmp/zogre-hf-case/recovery-sdk-preflight-draft-v8.py');raw=p.read_bytes()
if hashlib.sha256(raw).hexdigest()!='ad4fd8a9fa8a18c39e4f96b9a2de8ed28baef78ea397d97c64168ea267275dc0':raise RuntimeError('PIN')
m=types.ModuleType('sdk_preflight_snapshot');exec(compile(raw,str(p),'exec'),m.__dict__)
class Tests(unittest.TestCase):
 def test_exact_tampered_missing_extra_and_duplicate(self):
  with tempfile.TemporaryDirectory(prefix='sdk-preflight-fake-') as t:
   root=pathlib.Path(t);(root/'pkg').mkdir(mode=0o700);(root/'pkg'/'module.py').write_bytes(b'FAKE ONLY\n')
   p=root/'pkg'/'module.py';f=p.stat();d=(root/'pkg').stat();rows=[{'path':'pkg','kind':'directory','mode':d.st_mode&0o7777},{'path':'pkg/module.py','kind':'file','mode':f.st_mode&0o7777,'bytes':len(p.read_bytes()),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}]
   fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    self.assertEqual(m.verify_tree(fd,rows)['entries'],2)
    for key in ('mode','bytes'):
     bad=[dict(x) for x in rows];bad[-1][key]=True
     with self.subTest(boolean=key),self.assertRaises(RuntimeError):m.verify_tree(fd,bad)
    with self.assertRaises(RuntimeError):m.verify_tree(fd,rows+[rows[-1]])
    p.write_bytes(b'CHANGED!!\n')
    with self.assertRaises(RuntimeError):m.verify_tree(fd,rows)
    p.unlink()
    with self.assertRaises(RuntimeError):m.verify_tree(fd,rows)
    p.write_bytes(b'FAKE ONLY\n');(root/'extra').touch()
    with self.assertRaises(RuntimeError):m.verify_tree(fd,rows)
   finally:os.close(fd)
 def test_symlink_and_traversal_refuse(self):
  with tempfile.TemporaryDirectory(prefix='sdk-preflight-fake-') as t:
   root=pathlib.Path(t);(root/'linked').symlink_to('/etc/passwd');fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    for rows in ([{'path':'linked','kind':'file','mode':0o600,'bytes':0,'sha256':'0'*64}],[{'path':'../escape','kind':'file'}],[{'path':'linked','kind':'symlink','target':'/etc/passwd'}]):
     with self.assertRaises((RuntimeError,OSError)):m.verify_tree(fd,rows)
   finally:os.close(fd)
 def test_fifo_no_writer_and_hardlink(self):
  with tempfile.TemporaryDirectory(prefix='sdk-preflight-fake-') as t:
   root=pathlib.Path(t);os.mkfifo(root/'pipe');(root/'real').write_bytes(b'x');os.link(root/'real',root/'hard')
   # A separate bounded process ensures a regression cannot hang this runner.
   code="import os,runpy,sys;d=runpy.run_path(sys.argv[1],run_name='testonly');fd=os.open(sys.argv[2],os.O_RDONLY|os.O_DIRECTORY);\ntry:d['verify_tree'](fd,[{'path':'pipe','kind':'file','mode':384,'bytes':0,'sha256':'0'*64}])\nexcept (RuntimeError,OSError):sys.exit(0)\nsys.exit(1)"
   child=subprocess.run([sys.executable,'-c',code,str(p),str(root)],capture_output=True,timeout=3)
   self.assertEqual(child.returncode,0)
   (root/'pipe').unlink();fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    rows=[{'path':'real','kind':'file','mode':(root/'real').stat().st_mode&0o7777,'bytes':1,'sha256':hashlib.sha256(b'x').hexdigest()},{'path':'hard','kind':'file','mode':(root/'hard').stat().st_mode&0o7777,'bytes':1,'sha256':hashlib.sha256(b'x').hexdigest()}]
    with self.assertRaises(RuntimeError):m.verify_tree(fd,rows)
   finally:os.close(fd)
 def test_directory_enumeration_and_rename(self):
  with tempfile.TemporaryDirectory(prefix='sdk-preflight-fake-') as t:
   root=pathlib.Path(t)
   for n in ('a','b','c'):(root/n).write_bytes(b'x')
   rows=[{'path':n,'kind':'file','mode':(root/n).stat().st_mode&0o7777,'bytes':1,'sha256':hashlib.sha256(b'x').hexdigest()} for n in ('a','b','c')]
   fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    with mock.patch.object(m,'MAX_ENTRIES',2):
     with self.assertRaises(RuntimeError):m.verify_tree(fd,rows[:2])
    original=m.os.read;changed=[False]
    def replace(fd,n):
     data=original(fd,n)
     if data and not changed[0]:(root/'a').rename(root/'old-a');(root/'a').write_bytes(b'x');changed[0]=True
     return data
    with mock.patch.object(m.os,'read',side_effect=replace):
     with self.assertRaises(RuntimeError):m.verify_tree(fd,rows)
    self.assertTrue(changed[0])
   finally:os.close(fd)
 def test_mocked_device_refused_before_open(self):
  import stat
  with tempfile.TemporaryDirectory(prefix='sdk-preflight-fake-') as t:
   root=pathlib.Path(t);(root/'device').write_bytes(b'x');fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   real_stat=os.stat;real_open=os.open;opened=[]
   def fake_stat(name,*args,**kwargs):
    result=real_stat(name,*args,**kwargs)
    if name=='device':return types.SimpleNamespace(st_mode=stat.S_IFCHR|0o600,st_nlink=1,st_size=1)
    return result
   def tracked_open(name,*args,**kwargs):opened.append(name);return real_open(name,*args,**kwargs)
   try:
    with mock.patch.object(m.os,'stat',side_effect=fake_stat),mock.patch.object(m.os,'open',side_effect=tracked_open):
     with self.assertRaises(RuntimeError):m.verify_tree(fd,[{'path':'device','kind':'file','mode':0o600,'bytes':1,'sha256':hashlib.sha256(b'x').hexdigest()}])
    self.assertNotIn('device',opened)
   finally:os.close(fd)
 def test_loaded_sdk_dependency_refused(self):
  m.reject_loaded_modules(('json','os','pathlib'))
  for name in ('huggingface_hub','huggingface_hub.hf_api','httpx2','yaml.loader','typing_extensions'):
   with self.subTest(name=name),self.assertRaises(RuntimeError):m.reject_loaded_modules(('json',name))
 def test_manifest_fifo_symlink_and_replace(self):
  with tempfile.TemporaryDirectory(prefix='sdk-preflight-fake-') as t:
   root=pathlib.Path(t);name='sdk-tree-inventory-v1.json';target=root/name;os.mkfifo(target)
   code="import os,runpy,sys;d=runpy.run_path(sys.argv[1],run_name='testonly');fd=os.open(sys.argv[2],os.O_RDONLY|os.O_DIRECTORY);\ntry:d['read_manifest'](fd)\nexcept (RuntimeError,OSError):sys.exit(0)\nsys.exit(1)"
   child=subprocess.run([sys.executable,'-c',code,str(p),str(root)],capture_output=True,timeout=3);self.assertEqual(child.returncode,0)
   target.unlink();target.symlink_to('/etc/passwd');fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY)
   try:
    with self.assertRaises((RuntimeError,OSError)):m.read_manifest(fd)
    target.unlink();target.write_bytes(b'{}');self.assertEqual(m.read_manifest(fd),b'{}')
    original=m.os.read;changed=[False]
    def replace(f,n):
     data=original(f,n)
     if data and not changed[0]:target.rename(root/'old');target.write_bytes(b'{}');changed[0]=True
     return data
    with mock.patch.object(m.os,'read',side_effect=replace):
     with self.assertRaises(RuntimeError):m.read_manifest(fd)
    self.assertTrue(changed[0])
   finally:os.close(fd)
 def test_manifest_mock_device_not_opened(self):
  import stat
  with tempfile.TemporaryDirectory(prefix='sdk-preflight-fake-') as t:
   root=pathlib.Path(t);(root/'sdk-tree-inventory-v1.json').write_bytes(b'{}');fd=os.open(root,os.O_RDONLY|os.O_DIRECTORY);realstat=os.stat;realopen=os.open;opened=[]
   def fake_stat(name,*args,**kwargs):
    result=realstat(name,*args,**kwargs)
    if name=='sdk-tree-inventory-v1.json':return types.SimpleNamespace(st_mode=stat.S_IFCHR|0o600,st_nlink=1,st_size=2)
    return result
   def tracked(name,*args,**kwargs):opened.append(name);return realopen(name,*args,**kwargs)
   try:
    with mock.patch.object(m.os,'stat',side_effect=fake_stat),mock.patch.object(m.os,'open',side_effect=tracked):
     with self.assertRaises(RuntimeError):m.read_manifest(fd)
    self.assertNotIn('sdk-tree-inventory-v1.json',opened)
   finally:os.close(fd)
if __name__=='__main__':unittest.main()
