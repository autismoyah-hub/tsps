"""Test: do hidden faces vote in an NPC's lighting?  A four-sided limb of two segments (rings at heights 0, 40, 80,
closed at both ends), painted one colour (lightness 64), in four versions:
  A  no face inside
  B  a hidden fan across the middle ring (the "joint"), smooth, facing up
  C  the same fan flat-marked (render type 1)
  D  the same fan smooth with alpha 255 (the client does not draw such a face)
Each is written with research-npc/meshio.py, lit by the CLIENT (creature_light.exe file:PATH, i.e. Model::light with
the NPC numbers 64, 850, (-30,-50,-30)) and by the port (light_npc.light); the lit lightness of the outer side faces'
corners at the middle ring is printed per version.
usage: light_test_votes.py OUTDIR
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, subprocess, collections
HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-npc'))
import light_npc, creature_kit, meshio
out = pathlib.Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
PAINT = meshio.col(6, 3, 64); INNER = meshio.col(40, 5, 64)
def build(version):
    k = creature_kit.Kit()
    r0 = k.ring_y(0, 0, 0, 14, 14, 4, 1); r1 = k.ring_y(0, 40, 0, 14, 14, 4, 2); r2 = k.ring_y(0, 80, 0, 14, 14, 4, 3)
    k.loft([r0, r1, r2], PAINT)
    k.cap(r0, PAINT, (0, 40, 0)); k.cap(r2, PAINT, (0, 40, 0))
    n_outer = len(k.F)
    if version != 'A':
        k.cap(r1, INNER, (0, 0, 0), flat=(version == 'C'))      # a fan inside the limb, facing up, never visible
    m = k.model()
    if version == 'D':
        m['alpha'] = [0] * n_outer + [255] * (len(k.F) - n_outer)
    p = out / f'limb-{version}.bin'; meshio.save(m, p)
    return p, [k.V[i] for i in r1]
rows = {}
for version in 'ABCD':
    p, ring = build(version)
    t = out / f'limb-{version}.txt'
    r = subprocess.run([light_npc.EXE, light_npc.CACHE, f'file:{p}', str(t)], capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    V, L, F, amb, con, _, _ = light_npc.read(t)
    port = light_npc.light(V, F, amb, con)
    same = sum(1 for f, m in zip(F, port) if f[7] == m)
    ringpos = {(x, -h, z) for x, h, z in ring}
    lit = collections.defaultdict(set)
    for f in F:
        if f[3] != PAINT or f[7][2] < 0: continue
        for corner, v in enumerate(f[:3]):
            if V[v] in ringpos: lit[V[v]].add(f[7][corner] & 127)
    drawn = sum(1 for f in F if f[7][2] != -2)
    rows[version] = {k: sorted(v) for k, v in sorted(lit.items())}
    print(f'{version}: {len(V)} vertices {len(F)} faces ({drawn} drawn); port = client on {same} of {len(F)} faces; '
          f'middle-ring corners of the OUTER faces, lit lightness by vertex (x, y, z): ' + ', '.join(f'{k}: {v[0] if len(v) == 1 else v}' for k, v in rows[version].items()))
print('B differs from A:', rows['B'] != rows['A'], '| C (flat fan) equals A:', rows['C'] == rows['A'], '| D (alpha 255 fan, not drawn) equals B:', rows['D'] == rows['B'])
