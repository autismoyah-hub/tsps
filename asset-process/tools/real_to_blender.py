"""Open a dumped real asset in Blender as an ordinary mesh. Run:
    blender -b -P real_to_blender.py -- DUMP.json OUT.blend
Faces keep their real colour (materials named hsl_H_S_L, with _flat and _pN where the real face is flat-shaded
or has its own draw priority, and _dN where it carries a depth offset), so the file can be studied, measured, and
exported again with blender_export.py. New dumps carry texture bases and raw transparency."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json
sys.path.insert(0, _AP.fix(r'<ROOT>\tools'))
from blender_kit import reset, add_mesh, save
argv = sys.argv[sys.argv.index('--') + 1:]
if argv[0].lower().endswith('.bin'):
    from pathlib import Path
    import osrscache as cache
    m = cache.decode_model(Path(argv[0]).read_bytes(),preserve_layout=True)
    d = dict(m,kind='model',id=Path(argv[0]).stem,vertices=[(x,-y,z) for x,y,z in m['vertices']],
             colours=[cache.hsl_split(c) for c in m['colors']],flat=[r==1 for r in m['render_types']],
             depth_offsets=m['depth_bias'],face_skins=m['face_skins'] if m['has_face_skin'] else None)
else:
    d = json.load(open(argv[0]))
reset()
# the dump's winding is the cache's (outward when y points down); with y up it must be reversed to stay outward
faces = [(a, c, b) for a, b, c in d['faces']]
# Per-face depth offsets (real_dump.py writes them as `depth_offsets`, in view-depth units = 2 x the stored byte)
# are carried into the material name as _dN, N = the stored byte, which is what blender_export.py writes back.
# Without this a real face lying exactly in another's plane (a shield's emblem, a painted pattern) loses its
# offset in the round trip and is hidden by the surface it lies in.
offsets = d.get('depth_offsets') or [0] * len(d['faces'])
if any(v % 2 for v in offsets):
    raise SystemExit('depth offset not a multiple of 2: not a value the model format can store')
cols = [(h, s, l, bool(f), p, v // 2) for (h, s, l), f, p, v in zip(d['colours'], d['flat'], d['priorities'], offsets)]
if 'texture_coords' in d:
    faces = [tuple(reversed(f)) for f in d['faces']]
    cols = [c+(None if t<0 else t,a) for c,t,a in zip(cols,d['face_textures'],d['alpha'])]
ob = add_mesh(f"real_{d['kind']}_{d['id']}", [tuple(v) for v in d['vertices']], faces, cols)
if 'texture_coords' in d:
    me = ob.data
    me['cache_basis'] = json.dumps(d['texture_triangles'])
    if d.get('uniform_priority') is not None:
        me['cache_priority'] = d['uniform_priority']
    me['cache_alpha_stream'] = bool(d.get('has_alpha'))
    if any(d.get('texture_types',[])):
        me['cache_types'] = json.dumps(d['texture_types'])
        me['cache_complex'] = json.dumps(d['complex_sections'])
    if d.get('bone_weights') is not None:
        me['cache_weights'] = json.dumps(d['bone_weights'])
    if 'packing' in d:
        me['cache_packing'] = json.dumps(d['packing'])
    for name, domain, values in [('cache_vertex','POINT',range(len(me.vertices))),
                                 ('cache_coord','FACE',d['texture_coords']),
                                 ('cache_render','FACE',d['render_types']),
                                 ('cache_vskin','POINT',d['vertex_skins']),
                                 ('cache_fskin','FACE',d['face_skins'])]:
        if values is not None:
            me.attributes.new(name,'INT',domain).data.foreach_set('value',list(values))
if '--in-memory' not in argv:
    save(argv[1])
print('saved', argv[1], len(d['vertices']), 'vertices', len(faces), 'faces')
