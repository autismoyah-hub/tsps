"""Blender: a real skeletal creature into a scene.   blender -b --python-exit-code 1 -P rig_import.py -- MANIFEST.json OUT.blend [--seqs A,B,C]
MANIFEST.json is rig_fetch.py's. Writes OUT.blend and OUT.import.json (what was built; per action the worst vertex
error of holding the pose as Blender location / quaternion / scale). See rig_blender.py for what the scene holds."""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import bpy, rig_blender as RB
a = sys.argv[sys.argv.index('--') + 1:]
seqs = {int(x) for x in a[a.index('--seqs') + 1].split(',')} if '--seqs' in a else None
rig, parts, rep = RB.import_creature(a[0], seqs, helpers='none' if '--no-helpers' in a else 'auto')
print('scale helpers on bones', rep['scale_helpers'])
bpy.ops.wm.save_as_mainfile(filepath=str(pathlib.Path(a[1]).resolve()))
pathlib.Path(a[1]).with_suffix('.import.json').write_text(json.dumps(rep, indent=1))
print(f"IMPORTED {rep['name']}: {rep['bones']} bones, parts {rep['parts']}, {len(rep['actions'])} actions; worst vertex error of the Blender form {max([x['worst_vertex_error_units'] for x in rep['actions']] or [0]):.5f} units -> {a[1]}")
