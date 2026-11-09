"""Synthetic pinned frame preflight/decoder/seal compatibility, no provider."""
import base64,hashlib,json,pathlib,tempfile,types,unittest
r=pathlib.Path('/tmp/zogre-hf-case')
def load(name,digest):
 p=r/name;raw=p.read_bytes()
 if hashlib.sha256(raw).hexdigest()!=digest:raise RuntimeError('PIN')
 m=types.ModuleType(name);exec(compile(raw,str(p),'exec'),m.__dict__);return m
pre=load('recovery-frame-preflight-draft-v1.py','43dccdb042923bc3fa4af4b2ecfbdbef72ad5cdec14c19710ba521a94918ffa4')
frames=load('frame_transport.py','f53689d4f2389af93ad8f6a43fe0de94caf8dcaabde3eacbd13b1c957b129a76')
seal=load('recovery-frame-seal-draft-v1.py','d2db3092f8870f566a6d52af0ba3917b1a3bca3d475d052056e38f029e556049')
class Controls(unittest.TestCase):
 def input(self):
  data=b'hello';empty=hashlib.sha256(b'').hexdigest();h=hashlib.sha256(data).hexdigest()
  frame={'event':'child-output-frame','version':1,'stage':'build','stream':'stdout','index':0,'base64':base64.b64encode(data).decode(),'bytes':len(data),'sha256':h}
  result={'event':'stage-result','name':'build','outputTransport':{'stdout':{'version':1,'frames':1,'bytes':len(data),'sha256':h},'stderr':{'version':1,'frames':0,'bytes':0,'sha256':empty}}}
  return [json.dumps(frame),json.dumps(result)]
 def test_pinned_decoder_and_seal(self):
  with tempfile.TemporaryDirectory(prefix='zogre-frame-chain-') as t:
   lines=self.input();self.assertEqual(pre.validate(lines)['streamPairs'],2)
   d=pathlib.Path(t)/'decoded';report=frames.reconstruct_child_outputs(lines,d)
   self.assertTrue(report['complete']);self.assertEqual(len(report['streams']),2)
   self.assertEqual(seal.seal(d,report)['stage'],'FRAME_BYTES_SEALED_UNREVIEWED')
 def test_missing_frame_or_bad_metrics(self):
  with tempfile.TemporaryDirectory(prefix='zogre-frame-chain-') as t:
   lines=self.input();bad=frames.reconstruct_child_outputs(lines[1:],pathlib.Path(t)/'decoded')
   self.assertFalse(bad['complete'])
   with self.assertRaises(RuntimeError):seal.seal(pathlib.Path(t)/'decoded',bad)
  bad=json.loads(self.input()[1]);bad['outputTransport']['stdout']['bytes']=pre.MAX_DECLARED_BYTES+1
  with self.assertRaises(RuntimeError):pre.validate([json.dumps(bad)])
 def test_boundaries(self):
  for bad in ([],['{"event":"child-output-frame"'],[None],['x'*(pre.MAX_LINE_BYTES+1)]):
   with self.assertRaises(RuntimeError):pre.validate(bad)
if __name__=='__main__':unittest.main()
