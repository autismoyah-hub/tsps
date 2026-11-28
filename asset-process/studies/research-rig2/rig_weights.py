"""Blender: assign bone weights to the scene's mesh from its armature, by rule, and clean them to the game's form.
    blender -b SCENE.blend --python-exit-code 1 -P rig_weights.py -- OUT.blend --method nearest|heat|envelope [options]

Any weights already on the mesh are THROWN AWAY first.
Methods (which bone, before cleanup):
  nearest    each vertex goes to the bone whose segment (joint to each child joint; a leaf: a short stub) is nearest;
             --blend B gives a share to every bone whose distance is within B x (nearest distance) + --blend-units U
             --shift S (default 0.25): the last S of each joint-to-joint stretch belongs to the bone BEYOND the joint
             --leaf-reach K: a bone with no deforming bone below it takes no vertex farther from it than K x its own
             length (joint to parent joint): small end bones (spikes, plates, finger tips) do not steal the body's skin
  chain      as nearest, but the two bones of the nearest joint-to-joint stretch share the vertex by its place ALONG the
             stretch: the hand-over is centred at 1 - shift and --band wide (default 0.3 of the stretch)
  heat       Blender's automatic weights (bone heat: Object > Parent > With Automatic Weights), bones as drawn
  envelope   Blender's envelope weights; vertices no envelope reaches fall back to `nearest`
Bones with Deform switched off (bone.use_deform; rig_import switches it off for marker bones: leaves resting exactly on
the origin) take no weight and lend no segment. --no-deform A,B,.. switches more off.
--parts FILE.json   correction by rule, as the project's armour labels are assigned: a list of parts, each a SELECTION of
             vertices and the bones they may use; the method then chooses among those bones only.
             {"parts": [{"name": "left wing", "select": {"colours": ["hsl_30_3_40", ..]} and/or {"box": [[x0, y0, z0],
             [x1, y1, z1]]} (x, height, z in game units, y UP) and/or {"shell": N},
             "bones": {"subtree": BONE} or {"list": [BONE..]} (indices or names), "rigid": true (optional: one bone
             for the whole selection's every vertex = max 1 influence)} ...]}  Later parts win.
             Top level also: "no_deform": [BONE..] and "cut_segments": [BONE..] (the stretch from that bone's parent to it
             is a connector through empty air: it lends the parent no segment). "reach": [{"bones": [..], "units": N}]: those
             bones take no vertex farther than N units from their own segment (small plates, spikes, stubs).
Cleanup (the game's form; measured on real creatures, RIG.md):
  --max N          influences kept per vertex (default 2)      --floor F   shares under F are dropped (default 0.12)
  --snap S         a vertex whose largest share is at least S is given wholly to that bone (default 0.8)
  --steps K        shares rounded to multiples of 1/K before the bytes are made (default 4: 255 | 191+64 | 128+127)
  --smooth R       rounds of neighbour averaging BEFORE cleanup (default 0)
  --rigid-shells N a connected piece of at most N vertices rides ONE bone, the one holding most of its weight (default 0)
  --rings STYLE    soft | medium | hard: THE RING RULE (wlib.paint_rings, RIG2.md section 1). The method's result is
                   reduced to one bone per vertex and the joints are painted: the ring of skin at each joint shares
                   the two bones (soft: half and half at the joint, a quarter out to 0.8 limb radii, two mesh rings;
                   medium: a quarter on the touching ring within 0.45 radii; hard: none), and WEBS (--webs distance
                   | on | off): a vertex whose neighbourhood is 45% or more (--web-min) another, unrelated bone's region
                   shares with it by distance. The parts file may add "joints": {"rigid": [BONE..], "hinge": [..],
                   "ball": [..], "chain": [..]} (the joint at that bone's head) and "styles": {"soft": [BONE..], ..}.
                   Use with --blend 0 --snap 0 --floor 0.06.
Bytes always sum to 255. Writes OUT.blend and prints the distribution.
"""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import bpy
import numpy as np
import rig_blender as RB, wlib
RM = RB.RM
a = sys.argv[sys.argv.index('--') + 1:]
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
method = opt('--method', 'nearest'); rig = RB.find_rig(); obs = [o for o in RB.mesh_objects()]
base, sk, A, names, _ = RB.export_base(rig); index = {n: i for i, n in enumerate(names)}
def bone_id(x): return index[x] if isinstance(x, str) and x in index else int(x)
for x in (opt('--no-deform', '') or '').split(','):
    if x: rig.data.bones[names[bone_id(x)]].use_deform = False
deform = np.array([rig.data.bones[n].use_deform for n in names])
parts = json.loads(pathlib.Path(opt('--parts')).read_text())['parts'] if opt('--parts') else []
for x in json.loads(pathlib.Path(opt('--parts')).read_text()).get('no_deform', []) if opt('--parts') else []:
    rig.data.bones[names[bone_id(x)]].use_deform = False; deform[bone_id(x)] = False
cut = {bone_id(x) for x in (json.loads(pathlib.Path(opt('--parts')).read_text()).get('cut_segments', []) if opt('--parts') else [])}
REACH = json.loads(pathlib.Path(opt('--parts')).read_text()).get('reach', []) if opt('--parts') else []
PJ = json.loads(pathlib.Path(opt('--parts')).read_text()) if opt('--parts') else {}
subtree = lambda b: [b] + [y for c in sk.children[b] for y in subtree(c)]
print(f'{len(names)} bones, {int(deform.sum())} deforming; method {method}; {len(parts)} parts')

def nearest(V, allowed, blend, blend_units):
    """allowed (nv, nb) bool -> dense W."""
    seg = wlib.segments_deform(sk, deform, cut=cut, shift=float(opt('--shift', 0.25)))
    D = wlib.vertex_bone_distance(V, seg, sk.n); D[~allowed] = np.inf
    if REACH:
        Dk = D.copy()
        for r in REACH:
            for x in r['bones']: Dk[Dk[:, bone_id(x)] > float(r['units']), bone_id(x)] = np.inf
        ok = np.isfinite(Dk).any(1); D[ok] = Dk[ok]
    lr = float(opt('--leaf-reach', 0))
    if lr > 0:                                   # a bone with nothing below it reaches no farther than lr x its own length
        H = sk.head(); Dk = D.copy()
        for i in range(sk.n):
            if not [c for c in sk.children[i] if deform[c]] and sk.parent[i] >= 0: Dk[Dk[:, i] > lr * np.linalg.norm(H[i] - H[sk.parent[i]]), i] = np.inf
        ok = np.isfinite(Dk).any(1); D[ok] = Dk[ok]
    dmin = D.min(1, keepdims=True)
    if blend <= 0 and blend_units <= 0:
        W = np.zeros_like(D); W[np.arange(len(V)), D.argmin(1)] = 1; return W
    band = blend * dmin + blend_units
    W = np.clip(1 - (D - dmin) / np.maximum(band, 1e-9), 0, None); W[~np.isfinite(D)] = 0
    return W / W.sum(1, keepdims=True)

def chain(V, allowed, shift, band):
    """Blend ALONG the bone, not by distance from it. Each vertex finds its nearest joint-to-joint stretch (parent p,
    child c) and its place t along it (0 at p's joint, 1 at c's). The hand-over to c is centred at t = 1 - shift and
    `band` wide (shares of the stretch): c's share = 0.5 + (t - (1 - shift)) / band, clipped to 0..1."""
    H = sk.head(); nv = len(V); best = np.full(nv, np.inf); W = np.zeros((nv, sk.n))
    P = np.full(nv, -1); Cc = np.full(nv, -1); Tt = np.zeros(nv)
    for i in range(sk.n):
        if not deform[i]: continue
        kids = [c for c in sk.children[i] if np.linalg.norm(H[c] - H[i]) > 1e-6 and (deform[c] or sk.children[c]) and c not in cut]
        segs = [(c, H[i], H[c]) for c in kids]
        if not kids:
            p = sk.parent[i]; d = H[i] - H[p] if p >= 0 else np.zeros(3); segs = [(-1, H[i], H[i] + d * 0.35)]
        for c, p0, p1 in segs:
            d, t = wlib.point_segment(V, p0, p1); d = np.where(allowed[:, i], d, np.inf)
            for r in REACH:
                if i in {bone_id(x) for x in r['bones']}: d = np.where(d > float(r['units']), np.inf, d)
            better = (d < best - 1e-9) | ((np.abs(d - best) <= 1e-9) & (t > Tt))        # a ring exactly on a joint: the stretch that ENDS there, so it is shared
            best[better] = d[better]; P[better] = i; Cc[better] = c if (c >= 0 and deform[c]) else -1; Tt[better] = t[better]
    lost = P < 0
    if lost.any(): W[lost] = nearest(V[lost], allowed[lost], 0, 0)
    for v in np.nonzero(~lost)[0]:
        p, c = P[v], Cc[v]
        if c < 0 or not allowed[v, c]: W[v, p] = 1; continue
        wc = float(np.clip(0.5 + (Tt[v] - (1 - shift)) / max(band, 1e-9), 0, 1)); W[v, c] = wc; W[v, p] = 1 - wc
    return W

def blender_auto(ob, kind):
    """Blender's own weighting; returns dense W in bone-index order (rows may be all zero)."""
    ob.vertex_groups.clear()
    for m in list(ob.modifiers):
        if m.type == 'ARMATURE': ob.modifiers.remove(m)
    ob.parent = None
    bpy.ops.object.select_all(action='DESELECT'); ob.select_set(True); rig.select_set(True); bpy.context.view_layer.objects.active = rig
    r = bpy.ops.object.parent_set(type=kind)
    W = np.zeros((len(ob.data.vertices), sk.n)); gmap = {g.index: index[g.name] for g in ob.vertex_groups if g.name in index}
    for v in ob.data.vertices:
        for g in v.groups:
            if g.group in gmap: W[v.index, gmap[g.group]] = g.weight
    return W

for ob in obs:
    if ob.get('osrs_file') and not len(ob.vertex_groups): print(f'  {ob.name}: an imported model with no weights at all: left unweighted'); continue
    m, _ = RB.export_model(ob, None); V = np.array(m['vertices'], float); F = np.array(m['faces']); nv = len(V)
    allowed = np.tile(deform, (nv, 1)); rigid_sel = []
    # colours a vertex touches
    vcol = [set() for _ in range(nv)]
    for f, c in zip(m['faces'], m['colors']):
        for v in f: vcol[v].add(f'hsl_{(c >> 10) & 63}_{(c >> 7) & 7}_{c & 127}')
    sh = wlib.weld_shells(V, F)
    for p in parts:
        s = p.get('select', {}); sel = np.ones(nv, bool)
        if 'colours' in s: cs = set(s['colours']); sel &= np.array([bool(vc & cs) and (not s.get('only') or vc <= cs) for vc in vcol])
        if 'hues' in s: hs = set(s['hues']); sel &= np.array([all(int(c.split('_')[1]) in hs for c in vc) if vc else False for vc in vcol])
        if 'box' in s:
            (x0, y0, z0), (x1, y1, z1) = s['box']; yu = -V[:, 1]
            sel &= (V[:, 0] >= x0) & (V[:, 0] <= x1) & (yu >= y0) & (yu <= y1) & (V[:, 2] >= z0) & (V[:, 2] <= z1)
        if 'shell' in s: sel &= sh == int(s['shell'])
        if 'vertices' in s: only = np.zeros(nv, bool); only[s['vertices']] = True; sel &= only
        b = p['bones']; bl = subtree(bone_id(b['subtree'])) if 'subtree' in b else [bone_id(x) for x in b['list']]
        for x in b.get('minus_subtree', []): bl = [y for y in bl if y not in set(subtree(bone_id(x)))]
        row = np.zeros(sk.n, bool); row[bl] = True; row &= deform
        allowed[sel] = row
        if p.get('rigid'): rigid_sel.append(sel)
        print(f"  part {p.get('name')}: {int(sel.sum())} vertices, {int(row.sum())} candidate bones")
    blend = float(opt('--blend', 0)); bu = float(opt('--blend-units', 0))
    if method == 'nearest': W = nearest(V, allowed, blend, bu)
    elif method == 'chain': W = chain(V, allowed, float(opt('--shift', 0.25)), float(opt('--band', 0.3)))
    else:
        W = blender_auto(ob, 'ARMATURE_AUTO' if method == 'heat' else 'ARMATURE_ENVELOPE'); W[~allowed] = 0
        empty = W.sum(1) <= 1e-9; print(f'  {method}: {int(empty.sum())} of {nv} vertices got no weight from Blender (filled by nearest)')
        if empty.any(): W[empty] = nearest(V[empty], allowed[empty], blend, bu)
        W = W / W.sum(1, keepdims=True)
    if int(opt('--smooth', 0)):
        W = wlib.smooth(W, wlib.adjacency(nv, F), int(opt('--smooth'))); W[~allowed] = 0; W = W / W.sum(1, keepdims=True)
    for sel in rigid_sel:
        if sel.any(): b = int(W[sel].sum(0).argmax()); W[sel] = 0; W[sel, b] = 1
    if opt('--rings'):
        # THE RING RULE (wlib.paint_rings): the method above is reduced to ONE bone per vertex, then the joints are painted
        dom = W.argmax(1); owned = np.zeros(sk.n, bool); owned[np.unique(dom)] = True
        declared = {bone_id(b): k for k, bs in PJ.get('joints', {}).items() for b in bs}
        styles = {bone_id(b): k for k, bs in PJ.get('styles', {}).items() for b in bs}
        for sel in rigid_sel:                                   # a rigid part stays rigid: no ring at the joints of its bones
            for b in np.unique(dom[sel]): declared.setdefault(int(b), 'rigid')
        rep = []; wopt = opt('--webs', 'distance')
        W = wlib.paint_rings(sk, V, F, dom, wlib.joint_kinds(sk, owned, declared=declared), style=opt('--rings'), styles=styles, report=rep,
                             webs=(wopt if wopt == 'distance' else wopt == 'on'), web_min=float(opt('--web-min', 0.45)))
        for sel in rigid_sel:
            if sel.any(): W[sel] = 0; W[sel, dom[sel]] = 1
        import collections as _c
        kc = _c.Counter(j['kind'] for j in rep if 'kind' in j)
        print(f"  rings ({opt('--rings')}): {sum(1 for j in rep if j.get('painted'))} joints painted {dict(kc)}, {sum(j.get('painted', 0) for j in rep)} ring vertices, {sum(j.get('webs', 0) for j in rep)} web vertices")
    k = int(opt('--rigid-shells', 0))
    if k:
        raw = wlib.shells(nv, F)
        for s in range(int(raw.max()) + 1):
            idx = np.nonzero(raw == s)[0]
            if len(idx) <= k: b = int(W[idx].sum(0).argmax()); W[idx] = 0; W[idx, b] = 1
    steps = int(opt('--steps', 4))
    if steps:
        Wq = np.zeros_like(W)
        for i, row in enumerate(W):
            idx = np.argsort(-row)[:int(opt('--max', 2))]; w = row[idx] / row[idx].sum(); q = np.round(w * steps).astype(int)
            q[0] += steps - q.sum(); Wq[i, idx] = np.maximum(q, 0) / steps
        W = Wq / Wq.sum(1, keepdims=True)
    out = wlib.to_bytes(W, int(opt('--max', 2)), float(opt('--floor', 0.12)), float(opt('--snap', 0.8)))
    RB.set_weights(ob, out, names)
    if not any(md.type == 'ARMATURE' for md in ob.modifiers):
        md = ob.modifiers.new('Armature', 'ARMATURE'); md.object = rig
    ob.parent = rig; ob.matrix_parent_inverse.identity()
    inf = collections.Counter(len(w) for w in out)
    print(f"  {ob.name}: {nv} vertices; influences {dict(sorted(inf.items()))}; one influence {inf[1] / nv:.3f}; bones used {len({b for w in out for b, _ in w})}; sums not 255: {sum(1 for w in out if sum(x for _, x in w) != 255)}")
bpy.ops.wm.save_as_mainfile(filepath=str(pathlib.Path(a[0]).resolve()))
