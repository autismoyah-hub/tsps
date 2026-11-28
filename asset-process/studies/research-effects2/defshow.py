"""defshow.py ID..: the decoded object definition (all fields) and its sequence's fields."""
import sys, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import objlib
from fxlib import *
for a in sys.argv[1:]:
    d = objlib.obj(int(a)); print(a, names(6).get(int(a)), {k: v for k, v in d.items()})
    if d['anim'] is not None:
        s = al.sequence(d['anim']); print('   seq', d['anim'], {k: v for k, v in s.items() if k not in ('frames', 'lengths', 'sounds')}, 'lengths', s['lengths'][:20], 'n', len(s['frames']))
    for mid in d['models']:
        m = oc.model(mid); print('   model', mid, len(m['vertices']), 'v', len(m['faces']), 'f', 'format', m['format'], 'textured', sum(1 for t in m['face_textures'] if t != -1), 'prios', sorted(set(m['priorities'])), 'types', sorted(set(m['render_types'])))
