"""Blender: the scene's skeletal creature out as game files.
    blender -b SCENE.blend --python-exit-code 1 -P rig_export.py -- OUTDIR [--pure] [--mode baked|spline] [--step N] [--actions NAME,NAME] [--base-id N]
Writes OUTDIR/part-N.bin (one per mesh object), base_ID.dat, anim_ACTION.dat per action, export.json.
--pure     re-derive every bone's curves from Blender's evaluated pose (default: untouched bones keep the source curves)
--mode     baked (default): a key every key time of the source, or every --step cycles, each stepping to the next;
           spline: a key every --step cycles with tangents, the client interpolates every cycle
See rig_blender.py."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rig_blender as RB
a = sys.argv[sys.argv.index('--') + 1:]
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
rep = RB.export_scene(a[0], pure='--pure' in a, mode=opt('--mode', 'baked'), step=int(opt('--step')) if opt('--step') else None,
                      actions=set(opt('--actions').split(',')) if opt('--actions') else None, base_id=int(opt('--base-id')) if opt('--base-id') else None, log=lambda *x: None)
print(f"EXPORTED base {rep['base']} ({rep['bones']} bones, {rep['bones_rest_verbatim']} rest matrices verbatim), {len(rep['models'])} models, {len(rep['actions'])} actions -> {a[0]}")
for m in rep['models']: print('  model', m)
for x in rep['actions']: print('  action', {k: v for k, v in x.items()})
