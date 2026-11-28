"""Focused independent byte expectations; no Blender/cache/native execution."""
import copy
import hashlib
import json
from pathlib import Path
import struct
import tempfile
import unittest

import synthetic_expectations
import synthetic_native_probe as probe
import stage_synthetic_cache as stager


class ProbeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.ids = dict(model=65001,skeleton=65002,frames=65003,sequence=65004,location=62001)

    def export(self, observation=None):
        out=self.root/'new'
        result=probe.encode(synthetic_expectations.expected() if observation is None else observation,self.ids,out)
        return out,result

    def test_literal_action_bytes_and_policy(self):
        out,result=self.export()
        self.assertEqual((out/'skeleton-65002.bin').read_bytes(),bytes([3,0,2,1,0,1,1,1,1]))
        # Signed-smart literals: pivot16,-64,8 =>80,0,72; angle64=>c040.
        self.assertEqual((out/'frame-1.bin').read_bytes(),bytes.fromhex('fdea030707075000484040c040404040'))
        sequence=(b'\x01\x00\x03\x00\x32\x00\x4b\x00\x64\x00\x00\x00\x01\x00\x02'
                  +struct.pack('>HHH',65003,65003,65003)+b'\x05\x05\x08\x01\x09\x00\x0a\x00\x0b\x02\0')
        self.assertEqual((out/'sequence.bin').read_bytes(),sequence)
        self.assertEqual((out/'location.bin').read_bytes(),bytes.fromhex('07010000fde918fdec5900'))
        self.assertFalse(result['cache_admitted'])
        allocation=json.loads((out/'allocation.json').read_text())
        self.assertEqual(allocation['namespace'],self.ids)
        for row in allocation['inputs'].values():
            self.assertEqual(hashlib.sha256((out/row['file']).read_bytes()).hexdigest(),row['sha256'])

    def test_stager_retains_exact_explicit_policy(self):
        out,_=self.export()
        doc,payloads,_=stager.inputs_from(out,out/'allocation.json')
        self.assertEqual(doc['namespace'],self.ids)
        self.assertEqual(payloads['sequence'],(out/'sequence.bin').read_bytes())

    def test_stager_refuses_tampered_explicit_policy_even_with_new_hash(self):
        out,_=self.export()
        original=(out/'sequence.bin').read_bytes()
        tail=original[21:]
        wrong_duration=bytearray(original);wrong_duration[4]=49
        wrong_loops=bytearray(original);wrong_loops[24]=2
        variants=[bytes(wrong_duration),bytes(wrong_loops),original[:-1],original+b'\0',
                  original[:21]+bytes([5,5])+tail,original[:21]+bytes([99])+tail,
                  original[:21]+tail[2:4]+tail[:2]+tail[4:]]
        for changed in variants:
            with self.subTest(changed=changed.hex()):
                (out/'sequence.bin').write_bytes(changed)
                allocation=json.loads((out/'allocation.json').read_text())
                allocation['inputs']['sequence']['sha256']=hashlib.sha256(changed).hexdigest()
                (out/'allocation.json').write_text(json.dumps(allocation))
                with self.assertRaisesRegex(ValueError,'explicit action policy'):
                    stager.inputs_from(out,out/'allocation.json')

    def test_hash_tamper_is_rejected_before_policy_validation(self):
        out,_=self.export()
        (out/'model.bin').write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError,'input SHA'):
            stager.inputs_from(out,out/'allocation.json')

    def test_observation_tampering_has_no_output(self):
        mutations=[lambda v:v['model']['vertex_skins'].__setitem__(4,0),
                   lambda v:v['action']['frames'][1]['transforms'][0][1].__setitem__(0,0),
                   lambda v:v['action']['frames'][2].__setitem__('length',99),
                   lambda v:v['action']['frames'].pop(),
                   lambda v:v['model'].pop('vertex_skins'),
                   lambda v:v['model']['vertices'][0].__setitem__(0,True),
                   lambda v:v['preview'][1][4].__setitem__(0,-47)]
        for change in mutations:
            with self.subTest(change=change):
                observation=copy.deepcopy(synthetic_expectations.expected());change(observation)
                with self.assertRaises(ValueError): self.export(observation)
                self.assertFalse((self.root/'new').exists())

    def test_namespace_is_explicit_and_strict(self):
        for invalid in [dict(self.ids,model=True),dict(self.ids,model=65535),dict(self.ids,model=0),
                        dict(self.ids,unexpected=1),{k:v for k,v in self.ids.items() if k!='frames'}]:
            with self.subTest(invalid=invalid),self.assertRaises(ValueError):probe.namespace(invalid)

    def test_existing_output_is_untouched(self):
        out,_=self.export(); before={p.name:p.read_bytes() for p in out.iterdir()}
        with self.assertRaises(ValueError):self.export()
        self.assertEqual(before,{p.name:p.read_bytes() for p in out.iterdir()})

    def test_model_header_counts_and_labels(self):
        out,_=self.export(); data=(out/'model.bin').read_bytes()
        self.assertEqual(data[-2:],b'\xff\xfd')
        # SkinnedExtended footer: counts, texture count, flags; independent offsets.
        self.assertEqual(struct.unpack('>HH',data[-26:-22]),(8,12))
        self.assertLess(sum(p.stat().st_size for p in out.iterdir()),65536)


if __name__=='__main__': unittest.main()
