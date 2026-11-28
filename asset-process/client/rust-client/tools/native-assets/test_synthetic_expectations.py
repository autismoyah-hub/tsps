"""Independent numeric/contract checks only; never imports bpy or runs artifacts."""
import copy
from collections import Counter
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import blender_synthetic_probe as probe
import synthetic_expectations as oracle

class SyntheticContractTests(unittest.TestCase):
    def sample(self):
        return probe.observation(*probe.authored_input())

    def test_independent_literal_pose_agreement(self):
        result=self.sample()
        oracle.validate(result)
        # Native integer rounding is intentionally NOT ideal floating rotation.
        self.assertEqual(result['preview'][1][4],[-48,-49,0])
        self.assertEqual(result['preview'][2][6],[-40,-128,32])

    def test_closed_connected_oriented_mesh(self):
        model=self.sample()['model'];edges=Counter()
        for a,b,c in model['faces']:
            for u,v in [(a,b),(b,c),(c,a)]:edges[(u,v)]+=1
        self.assertEqual(len(model['vertices']),8)
        self.assertEqual(len(model['faces']),12)
        for (u,v),count in edges.items():
            self.assertEqual(count,1)
            self.assertEqual(edges[(v,u)],1)
        reached={0}
        while True:
            expanded=reached|{v for u,v in edges if u in reached}
            if expanded==reached:break
            reached=expanded
        self.assertEqual(reached,set(range(8)))

    def test_two_labels_and_exact_durations(self):
        result=self.sample()
        self.assertEqual(Counter(result['model']['vertex_skins']),{0:4,1:4})
        self.assertEqual([r['length'] for r in result['action']['frames']],[50,75,100])
        self.assertEqual(result['action']['frames'][0]['labels'][0],[])

    def test_tampered_observations_fail_independently(self):
        changes=[lambda d:d['model']['vertices'][1].__setitem__(0,127),
                 lambda d:d['model']['vertices'][4].__setitem__(1,128),
                 lambda d:d['model']['vertex_skins'].__setitem__(4,0),
                 lambda d:d['model']['colors'].__setitem__(0,8084),
                 lambda d:d['action']['frames'][1]['transforms'][0][1].__setitem__(0,0),
                 lambda d:d['action']['frames'][1]['transforms'][1][1].__setitem__(2,-64),
                 lambda d:d['action']['frames'][2]['transforms'].reverse(),
                 lambda d:d['action']['frames'][1].__setitem__('length',74),
                 lambda d:d['preview'][1][4].__setitem__(1,-48),
                 lambda d:d['model']['faces'][0].reverse()]
        for i,change in enumerate(changes):
            with self.subTest(case=i):
                data=self.sample();change(data)
                with self.assertRaises(ValueError):oracle.validate(data)

    def test_missing_extra_empty_type_and_nonfinite_fail(self):
        cases=[{},[],oracle.expected(),oracle.expected(),oracle.expected(),oracle.expected()]
        del cases[2]['action']['frames'][0]['labels']
        cases[3]['unexpected']=1
        cases[4]['model']['vertices'][0][0]=False
        cases[5]['model']['vertices'][0][0]=float('nan')
        for data in cases:
            with self.subTest(data_type=type(data).__name__):
                with self.assertRaises(ValueError):oracle.validate(data)

    def test_oracle_returns_fresh_contract(self):
        data=oracle.expected();data['preview'][0][0][0]=999
        self.assertEqual(oracle.expected()['preview'][0][0][0],0)

    def test_json_reader_rejects_duplicate_nonfinite_and_oversize(self):
        with tempfile.TemporaryDirectory() as temp:
            f=Path(temp)/'input.json'
            for text in ['{"schema":1,"schema":1}','{"schema":NaN}',' '*(1<<20)+'x']:
                f.write_text(text)
                with self.assertRaises(ValueError):oracle.load(f)

    def test_output_is_deterministic_and_declaration_precedes_builder(self):
        with tempfile.TemporaryDirectory() as temp:
            def builder(folder):
                self.assertTrue((folder/'expectations.json').is_file())
                self.assertFalse((folder/'observation.json').exists())
                return self.sample()
            for name in ['a','b']:probe.export_bundle(Path(temp)/name,builder)
            for name in ['expectations.json','observation.json','complete.json']:
                self.assertEqual((Path(temp)/'a'/name).read_bytes(),(Path(temp)/'b'/name).read_bytes())

    def test_existing_destination_never_calls_builder_or_clobbers(self):
        with tempfile.TemporaryDirectory() as temp:
            folder=Path(temp)/'existing';folder.mkdir();sentinel=folder/'keep';sentinel.write_bytes(b'unchanged')
            def forbidden(_):self.fail('builder must not run')
            with self.assertRaises(FileExistsError):probe.export_bundle(folder,forbidden)
            self.assertEqual(sentinel.read_bytes(),b'unchanged')

    def test_failed_builder_retains_partial_without_complete_marker(self):
        with tempfile.TemporaryDirectory() as temp:
            folder=Path(temp)/'partial'
            def failed(_):raise ValueError('synthetic failure')
            with self.assertRaises(ValueError):probe.export_bundle(folder,failed)
            self.assertTrue((folder/'expectations.json').exists())
            self.assertFalse((folder/'complete.json').exists())
            with self.assertRaises(FileExistsError):probe.export_bundle(folder,failed)

    def test_invalid_builder_result_cannot_publish_complete(self):
        with tempfile.TemporaryDirectory() as temp:
            folder=Path(temp)/'bad'
            with self.assertRaises(ValueError):probe.export_bundle(folder,lambda _:dict(schema=1))
            self.assertFalse((folder/'observation.json').exists())
            self.assertFalse((folder/'complete.json').exists())

    def test_cli_is_numeric_only_and_tamper_fails(self):
        with tempfile.TemporaryDirectory() as temp:
            f=Path(temp)/'observation.json';f.write_text(json.dumps(self.sample()))
            cmd=[sys.executable,str(Path(oracle.__file__)),str(f)]
            valid=subprocess.run(cmd,capture_output=True,text=True)
            self.assertEqual(valid.returncode,0,valid.stderr)
            self.assertIn('NUMERIC_ONLY_PASS',valid.stdout)
            data=self.sample();data['action']['frames'].pop();f.write_text(json.dumps(data))
            invalid=subprocess.run(cmd,capture_output=True,text=True)
            self.assertEqual(invalid.returncode,2)
            self.assertIn('numeric check failed',invalid.stderr)

    def test_pure_import_does_not_import_blender(self):
        self.assertNotIn('bpy',probe.__dict__)

if __name__=='__main__':unittest.main()
