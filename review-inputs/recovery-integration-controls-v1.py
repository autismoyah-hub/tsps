"""Offline synthetic controls; never import or execute the recovery entry."""
import ast, hashlib, json, os, pathlib, tempfile, types, unittest
from unittest import mock
ROOT=pathlib.Path('/tmp/zogre-hf-case')
ENTRY='a109e3b2633fa2f93561ad7d97a63b934ba3fa920153cf537b0ea232722a70ed'
RECORDS='b37e74ba6bde114d968dead3a0ad2b4d7f2306976e6c402af98368584826b22c'
def readpin(path, expected):
 raw=path.read_bytes()
 if hashlib.sha256(raw).hexdigest()!=expected:raise RuntimeError('PIN_MISMATCH')
 return raw
source=readpin(ROOT/'recovery-entry-draft-v1.py',ENTRY)
tree=ast.parse(source); matches=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='durable']
if len(matches)!=1:raise RuntimeError('EXTRACTION_ERROR')
ns={'json':json,'os':os};exec(compile(ast.fix_missing_locations(ast.Module(body=matches,type_ignores=[])),'extracted-durable','exec'),ns)
record_source=readpin(ROOT/'recovery-records-draft-v5.py',RECORDS);rec=types.ModuleType('record_control');exec(compile(record_source,'verified-record-snapshot','exec'),rec.__dict__)
class Controls(unittest.TestCase):
 def test_durable_and_duplicate(self):
  with tempfile.TemporaryDirectory(prefix='zogre-recovery-controls-') as tmp:
   p=pathlib.Path(tmp)/'intent';ns['durable'](p,{'stage':'PENDING_NO_RETRY'})
   self.assertEqual(json.loads(p.read_text())['stage'],'PENDING_NO_RETRY')
   with self.assertRaises(FileExistsError):ns['durable'](p,{'stage':'OTHER'})
   self.assertEqual(json.loads(p.read_text())['stage'],'PENDING_NO_RETRY')
 def test_fsync_and_disk_error(self):
  with tempfile.TemporaryDirectory(prefix='zogre-recovery-controls-') as tmp:
   p=pathlib.Path(tmp)/'intent'
   with mock.patch.object(os,'fsync',side_effect=OSError('synthetic')):
    with self.assertRaises(OSError):ns['durable'](p,{'stage':'PENDING_NO_RETRY'})
   self.assertTrue(p.exists())
   with mock.patch.object(os,'open',side_effect=OSError('synthetic')):
    with self.assertRaises(OSError):ns['durable'](pathlib.Path(tmp)/'other',{})
 def test_timeout_prefix_retained(self):
  with tempfile.TemporaryDirectory(prefix='zogre-recovery-controls-') as tmp:
   def sequence():
    yield 'one'
    raise TimeoutError()
   p=pathlib.Path(tmp)/'records';result=rec.retain_records(sequence(),p)
   self.assertEqual((result['outcome'],result['reason'],result['records']),('UNKNOWN','TIMEOUT',1))
   self.assertEqual(result['fileSHA256'],hashlib.sha256(p.read_bytes()).hexdigest())
if __name__=='__main__':unittest.main()
