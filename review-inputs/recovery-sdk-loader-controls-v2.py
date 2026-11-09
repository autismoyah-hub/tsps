"""Offline synthetic module-origin controls. Never imports actual SDK."""
import base64,hashlib,os,pathlib,subprocess,sys,tempfile,types,unittest
from unittest import mock
p=pathlib.Path('/tmp/zogre-hf-case/recovery-sdk-loader-draft-v2.py');raw=p.read_bytes()
if hashlib.sha256(raw).hexdigest()!='0e5ae7d0e2d30ef3038391083440d5b45f3e9f6ac7e72744ce1b66363783f70e':raise RuntimeError('PIN')
m=types.ModuleType('verified_loader');exec(compile(raw,str(p),'exec'),m.__dict__)
class Tests(unittest.TestCase):
 def test_fake_origin_files_match_and_mutations_refuse(self):
  with tempfile.TemporaryDirectory(prefix='zogre-fake-modules-') as t:
   root=pathlib.Path(t);names={'huggingface_hub':'__init__.py','huggingface_hub.hf_api':'hf_api.py','huggingface_hub._jobs_api':'_jobs_api.py'};modules={};rows=[]
   for name,file in names.items():
    path=root/file;data=('fake '+name).encode();path.write_bytes(data);modules[name]=types.SimpleNamespace(__file__=str(path));rows.append({'path':file,'kind':'file','bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
   opts={'snapshot':str(root),'sdkTopLevel':('huggingface_hub',)};self.assertEqual(set(m.validate_origins(modules,opts,{'entries':rows})),set(names))
   path=root/'hf_api.py';path.write_bytes(b'bad')
   with self.assertRaises(RuntimeError):m.validate_origins(modules,opts,{'entries':rows})
   modules['huggingface_hub.hf_api']=types.SimpleNamespace(__file__='/etc/passwd')
   with self.assertRaises(RuntimeError):m.validate_origins(modules,opts,{'entries':rows})
 def test_actual_entry_refuses_normal_and_optimized_without_sdk(self):
  b64=base64.b64encode(raw).decode();pin=hashlib.sha256(raw).hexdigest()
  code="import base64,hashlib,sys; b=base64.b64decode(sys.argv[1]); h=sys.argv[2]; hashlib.sha256(b).hexdigest()==h or sys.exit(91);\ntry:exec(compile(b,'snapshot','exec'),{'__name__':'__main__'})\nexcept RuntimeError as e:\n if str(e)!='SDK_IMPORT_AND_PROVIDER_DISABLED_PENDING_REVIEW' or any(k=='huggingface_hub' or k.startswith('huggingface_hub.') for k in sys.modules):sys.exit(3)\nelse:sys.exit(4)"

  for optimized in (False,True):
   argv=[sys.executable]+(['-O'] if optimized else [])+['-I','-S','-B','-c',code,b64,pin]
   with self.subTest(optimized=optimized):self.assertEqual(subprocess.run(argv,capture_output=True,timeout=5).returncode,0)
 def test_actual_candidate_with_fake_preflight_refuses_and_restores(self):
  with tempfile.TemporaryDirectory(prefix='zogre-fake-anchor-') as t:
   root=pathlib.Path(t);snap=root/'sdk-snapshot-v1';snap.mkdir();pre=root/'fake-preflight.py'
   pre.write_text('import os\ndef preflight():\n a=os.stat('+repr(str(root))+');s=os.stat('+repr(str(snap))+');return {"anchorIdentity":{"device":a.st_dev,"inode":a.st_ino},"snapshotIdentity":{"device":s.st_dev,"inode":s.st_ino}}\n')
   digest=hashlib.sha256(pre.read_bytes()).hexdigest();original=list(sys.path)
   import importlib.machinery
   hooks=[importlib.machinery.BuiltinImporter,importlib.machinery.FrozenImporter,importlib.machinery.PathFinder]
   with mock.patch.object(m,'ANCHOR',root),mock.patch.object(m,'PREFLIGHT',pre),mock.patch.object(m,'PREFLIGHT_SHA',digest),mock.patch.object(sys,'meta_path',hooks):
    with self.assertRaisesRegex(RuntimeError,'SDK_IMPORT_AND_PROVIDER_DISABLED_PENDING_REVIEW'):m.candidate_load()
   self.assertEqual(sys.path,original);self.assertFalse(any(x=='huggingface_hub' or x.startswith('huggingface_hub.') for x in sys.modules))
if __name__=='__main__':unittest.main()
