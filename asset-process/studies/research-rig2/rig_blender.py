"""The Blender side of the skeletal-creature route (runs inside Blender 5.2; import it from a script):

    import sys; sys.path.insert(0, r'<ROOT>\\studies\\research-rig'); import rig_blender as RB

    RB.import_creature(MANIFEST.json, seqs=None)      a real creature (rig_fetch.py output) as mesh objects, an ARMATURE
                                                      and one action per keyframe sequence
    RB.export_scene(OUTDIR, pure=False, mode='baked', step=None, actions=None, base_id=None)
                                                      model file(s), frame base and keyframe files through bonecodec

WHAT THE SCENE HOLDS
  mesh objects  one per model file ("part0"..): vertices at model / 128 (project convention), one triangle per face in
                the file's order, a material per colour named hsl_H_S_L (+ _flat), per-face integer attributes for
                what a material name does not carry (osrs_alpha, osrs_flabel, osrs_prio, osrs_depth, osrs_tex, osrs_tc,
                osrs_render), a vertex group per bone holding weight / 255, an Armature modifier.
  armature      "rig": one bone per bone of the frame base, named bNNN (NNN = the base's bone index, also in the custom
                property osrs_index). The bone's HEAD is the joint; its tail points at the child that continues the
                chain (a Blender bone's orientation is free: rigmath). The game's own rest matrix cannot be held by a
                Blender bone (its axes are arbitrary and may carry scale), so it rides on the bone as osrs_rest
                (16 numbers) with the bone's Blender rest at import (osrs_A); a bone MOVED in edit mode has its rest
                matrix carried along by the same rigid change on export; a NEW bone gets its Blender rest matrix.
  actions       "seq_ID": location / rotation_quaternion / scale keys on pose bones at the cycles where the real file's
                pose changes (osrs_times); one Blender frame = one client cycle (20 ms); scene at 50 fps.
EXPORT of an action = BAKING: the scene is set to each key time, every pose bone's evaluated matrix is read (so
constraints, IK and any interpolation count), turned into the client's nine channels per bone and written as real
files are keyed (mode 'baked': a key each `step` cycles that steps to the next) or as a true curve the client
interpolates (mode 'spline').
  pure=False  bones whose f-curves are bit-for-bit what the importer wrote, and whose baked values agree with the source
              file, keep the SOURCE FILE's curves verbatim (so an untouched action re-exports byte-identical);
  pure=True   every bone is re-derived from Blender's evaluated pose.
Alpha curves (kind 4) and kind-3 curves have no Blender form: they are copied from the action's source file (osrs_file)
or, for a new action, held at the value of a stance file's frame (osrs_stance_file / osrs_stance_frame).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, hashlib, re, colorsys
import bpy
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
for p in (str(HERE), _AP.fix(r'<ROOT>\studies\research-bosses')):
    if p not in sys.path: sys.path.insert(0, p)
import bonecodec as B, rigmath as RM
from bpy_extras import anim_utils

FACE_ATTRS = dict(alpha='osrs_alpha', face_skins='osrs_flabel', priorities='osrs_prio', depth='osrs_depth', face_textures='osrs_tex', texture_coords='osrs_tc')
MAT_RE = re.compile(r'hsl_(\d+)_(\d+)_(\d+)(_flat)?(?:_p(\d+))?(?:_d(\d+))?(?:_a(\d+))?(\.\d+)?')


def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene; sc.render.fps = 50; sc.frame_start = 0


def material(colour, flat=False):
    h, s, l = (colour >> 10) & 63, (colour >> 7) & 7, colour & 127
    name = f'hsl_{h}_{s}_{l}' + ('_flat' if flat else '')
    m = bpy.data.materials.get(name)
    if m is None:
        m = bpy.data.materials.new(name)
        r, g, b = colorsys.hls_to_rgb(h / 64 + 1 / 128, l / 127, s / 8 + 1 / 16); m.diffuse_color = (r, g, b, 1)
    return m


def bone_name(i): return f'b{i:03d}'


# ---------------------------------------------------------------------------------------------------- import
def add_model(name, m, part=0):
    """A decoded model (bonecodec) as a Blender mesh object. Vertex and face order are kept."""
    V = np.array(m['vertices'], float) @ RM.MODEL_TO_BLENDER.T
    me = bpy.data.meshes.new(name); me.from_pydata(V.tolist(), [], [tuple(f) for f in m['faces']])
    rt = m.get('render_types'); slots = {}; idx = []
    for i, c in enumerate(m['colors']):
        mt = material(c, bool(rt and rt[i] == 1))
        if mt.name not in slots: slots[mt.name] = len(me.materials); me.materials.append(mt)
        idx.append(slots[mt.name])
    me.polygons.foreach_set('material_index', idx)
    me.polygons.foreach_set('use_smooth', [not (rt and rt[i] == 1) for i in range(len(idx))])
    for key, attr in FACE_ATTRS.items():
        if m.get(key) is not None:
            a = me.attributes.new(attr, 'INT', 'FACE'); a.data.foreach_set('value', [int(x) for x in m[key]])
    if rt is not None and any(x not in (0, 1) for x in rt):
        a = me.attributes.new('osrs_render', 'INT', 'FACE'); a.data.foreach_set('value', [int(x) for x in rt])
    if m.get('vertex_skins') is not None:
        a = me.attributes.new('osrs_vlabel', 'INT', 'POINT'); a.data.foreach_set('value', [int(x) for x in m['vertex_skins']])
    me.update()
    ob = bpy.data.objects.new(name, me); bpy.context.scene.collection.objects.link(ob)
    ob['osrs_part'] = part; ob['osrs_priority'] = int(m.get('priority', 0)); ob['osrs_has_render'] = rt is not None
    extra = dict(texture_types=m.get('texture_types') or [], texture_triangles=[list(t) for t in m.get('texture_triangles') or []],
                 complex_raw=bytes(m.get('complex_raw') or b'').hex(), metadata=m.get('metadata'))
    ob['osrs_extra'] = json.dumps(extra)
    return ob


def set_weights(ob, weights, names):
    """weights [[(bone, byte)]] -> vertex groups (one per bone name, in bone order), each vertex's pairs in file order."""
    ob.vertex_groups.clear()
    groups = [ob.vertex_groups.new(name=n) for n in names]
    for i, ws in enumerate(weights):
        for b, w in ws: groups[b].add([i], w / 255.0, 'REPLACE')


def add_scale_helper_edit(arm, eb, parent_eb, head, parent_rot, name):
    """In edit mode: the helper bone NAME_s between a bone and its parent. Its rest axes are the PARENT's game axes,
    so its own Blender scale IS the game's scale channels of the bone (the client scales along the parent's axes,
    after the bone's rotation: diag(scale) x R; a Blender bone scales along its own axes before its rotation)."""
    from mathutils import Matrix
    M = np.eye(4); M[:3, :3] = RM.C3 @ parent_rot; M[:3, 3] = head
    h = arm.edit_bones.new(name + '_s'); h.head = head.tolist(); h.tail = (head + M[:3, 1] * (6 / RM.UNIT)).tolist()
    h.matrix = Matrix(M.tolist()); h.length = 6 / RM.UNIT
    if parent_eb is not None: h.parent = parent_eb; h.use_connect = False
    eb.parent = h; eb.use_connect = False
    return h


def build_armature(base, name='rig', helpers=()):
    """A decoded frame base's bones as a Blender armature object. Returns (object, Skeleton).
    helpers: bone indices that get a scale helper (bNNN_s; see add_scale_helper_edit)."""
    sk = RM.Skeleton(base); n = sk.n; helpers = set(helpers)
    arm = bpy.data.armatures.new(name); ob = bpy.data.objects.new(name, arm); bpy.context.scene.collection.objects.link(ob)
    bpy.context.view_layer.objects.active = ob; bpy.ops.object.mode_set(mode='EDIT')
    head = (RM.CS @ sk.rest)[:, :3, 3]                                       # joints in Blender space
    size = [1] * n
    for i in reversed(sk.order):
        if sk.parent[i] >= 0: size[sk.parent[i]] += size[i]
    eb = [None] * n; MIN = 3.0 / RM.UNIT
    import wlib; mark = set(wlib.markers(sk))               # leaves resting on the origin: markers, not deforming bones
    for i in sk.order:
        b = arm.edit_bones.new(bone_name(i)); b.head = head[i].tolist()
        kids = [c for c in sk.children[i] if np.linalg.norm(head[c] - head[i]) > MIN and c not in mark]
        if kids:
            c = max(kids, key=lambda c: (size[c], -c)); tail = head[c]
        else:
            p = sk.parent[i]; d = head[i] - head[p] if p >= 0 else np.array([0, 0, 1.0]); ln = np.linalg.norm(d)
            d = d / ln if ln > MIN else np.array([0, 0, 1.0]); tail = head[i] + d * max(MIN * 2, min(ln * 0.5, 40 / RM.UNIT))
        b.tail = tail.tolist(); b.roll = 0.0; eb[i] = b
        if sk.parent[i] >= 0: b.parent = eb[sk.parent[i]]; b.use_connect = False
        if i in helpers:
            p = sk.parent[i]; Rp = sk.rest[p][:3, :3] if p >= 0 else np.eye(3); Rp = Rp / np.linalg.norm(Rp, axis=0)
            add_scale_helper_edit(arm, b, eb[p] if p >= 0 else None, head[i], Rp, bone_name(i))
    bpy.ops.object.mode_set(mode='OBJECT')
    assert len(arm.bones) == n + len(helpers), f'Blender kept {len(arm.bones)} of {n + len(helpers)} bones'
    for i in helpers:
        h = arm.bones[bone_name(i) + '_s']; h.inherit_scale = 'FULL'; h['osrs_helper'] = i
    for i in range(n):
        bn = arm.bones[bone_name(i)]; bn.inherit_scale = 'FULL'; bn.use_inherit_rotation = True; bn.use_local_location = True
        bn.use_deform = i not in mark
        bn['osrs_index'] = i; bn['osrs_rest'] = [float(x) for x in base['bones'][i]['matrices'][0]]; bn['osrs_extra'] = [float(x) for x in base['bones'][i]['extra'][0]]
        bn['osrs_A'] = [float(x) for row in bn.matrix_local for x in row]
    ob['osrs_groups'] = json.dumps(dict(types=base['types'], labels=base['labels']))
    for pb in ob.pose.bones: pb.rotation_mode = 'QUATERNION'
    return ob, sk


def game_bones(ob):
    """The armature's bones that ARE bones of the frame base (helpers excluded)."""
    return [b for b in ob.data.bones if b.get('osrs_helper') is None]
def game_parent(b):
    p = b.parent
    while p is not None and p.get('osrs_helper') is not None: p = p.parent
    return p
def rest_A(ob, n=None):
    """Blender rest matrices (armature space) in bone-index order, float64, and the name of each index."""
    bones = sorted(game_bones(ob), key=lambda b: b['osrs_index']); assert [b['osrs_index'] for b in bones] == list(range(len(bones))), 'osrs_index must number the bones 0..n-1'
    return np.array([[list(r) for r in b.matrix_local] for b in bones], float), [b.name for b in bones]


class Form:
    """How a pose of the frame base is held by THIS armature: the game bones in index order, then the scale helpers.
    basis(channels) gives every Blender bone's basis matrix; a bone with a helper gives the helper its translation
    and scale and keeps only its rotation."""
    def __init__(self, ob, sk):
        self.sk = sk; n = sk.n; A, names = rest_A(ob); hs = sorted((b for b in ob.data.bones if b.get('osrs_helper') is not None), key=lambda b: b['osrs_helper'])
        self.helper = {int(b['osrs_helper']): n + k for k, b in enumerate(hs)}
        self.names = names + [b.name for b in hs]; self.A = np.concatenate([A, np.array([[list(r) for r in b.matrix_local] for b in hs], float).reshape(-1, 4, 4)])
        self.parent = [self.helper.get(i, sk.parent[i]) for i in range(n)] + [sk.parent[int(b['osrs_helper'])] for b in hs]
        self.order = []
        for i in sk.order:
            if i in self.helper: self.order.append(self.helper[i])
            self.order.append(i)
        rc = sk.rest_channels; local = self.locals(rc[None])[0]; self.rest = np.empty_like(local)
        for i in self.order: self.rest[i] = local[i] if self.parent[i] < 0 else self.rest[self.parent[i]] @ local[i]
    def locals(self, ch):
        """channels (T, n, 9) -> local matrices of the extended bone list (T, n + helpers, 4, 4)."""
        n = self.sk.n; out = np.empty(ch.shape[:1] + (len(self.names), 4, 4)); out[:, :n] = RM.compose(ch[..., 0:3], ch[..., 3:6], ch[..., 6:9])
        for i, h in self.helper.items():
            out[:, i] = RM.compose(ch[:, i, 0:3], np.zeros_like(ch[:, i, 3:6]), np.ones_like(ch[:, i, 6:9]))
            out[:, h] = RM.compose(np.zeros_like(ch[:, i, 0:3]), ch[:, i, 3:6], ch[:, i, 6:9])
        return out
    def basis(self, ch): return RM.basis_from_local(self.locals(ch), self.A, self.rest, self.parent)
    def pose(self, loc, quat, sc):
        """What Blender shows for float32 channel values: armature-space pose matrices of the GAME bones (T, n, 4, 4)."""
        f = lambda x: x.astype(np.float32).astype(float)
        return RM.pose_from_basis(RM.matrix_from_trs(f(loc), f(quat), f(sc)), self.A, self.parent, self.order)[:, :self.sk.n]


def fcurve_bag(ob, act):
    """The channel bag (f-curve list) of an action for the armature object, created if needed; assigns the action."""
    if ob.animation_data is None: ob.animation_data_create()
    slot = act.slots[0] if len(act.slots) else act.slots.new(id_type='OBJECT', name=ob.name)
    ob.animation_data.action = act; ob.animation_data.action_slot = slot
    return anim_utils.action_ensure_channelbag_for_slot(act, slot)


PATHS = (('location', 3), ('rotation_quaternion', 4), ('scale', 3))
def bone_fingerprints(bag):
    """{bone name: hash of every key (time, value) of its f-curves, as Blender stores them}."""
    acc = {}
    for fc in sorted(bag.fcurves, key=lambda f: (f.data_path, f.array_index)):
        mt = re.match(r'pose\.bones\["(.+?)"\]\.(\w+)', fc.data_path)
        if not mt: continue
        co = np.empty(2 * len(fc.keyframe_points), np.float32); fc.keyframe_points.foreach_get('co', co)
        owner = mt[1][:-2] if mt[1].endswith('_s') else mt[1]                 # a scale helper's curves belong to its bone
        h = acc.setdefault(owner, hashlib.sha1()); h.update(f'{mt[1]}.{mt[2]}[{fc.array_index}]'.encode()); h.update(co.tobytes())
    return {k: h.hexdigest()[:16] for k, h in acc.items()}


def write_action(ob, name, times, loc, quat, scale, names, interpolation='LINEAR'):
    """Keys on pose bones. loc (T, n, 3), quat (T, n, 4) w x y z, scale (T, n, 3) are Blender BASIS values at `times`.
    A channel that never changes gets one key at the first time."""
    act = bpy.data.actions.new(name); act.use_fake_user = True; bag = fcurve_bag(ob, act)
    quat = RM.continuous_quats(quat); data = dict(location=loc, rotation_quaternion=quat, scale=scale)
    interp = bpy.types.Keyframe.bl_rna.properties['interpolation'].enum_items[interpolation].value
    tt = np.array(times, np.float32)
    for i, bn in enumerate(names):
        for path, k in PATHS:
            v = data[path][:, i, :].astype(np.float32)
            for c in range(k):
                fc = bag.fcurves.new(f'pose.bones["{bn}"].{path}', index=c, group_name=bn)
                col = v[:, c]; nk = 1 if (col == col[0]).all() else len(tt)
                fc.keyframe_points.add(nk); co = np.empty(2 * nk, np.float32); co[0::2] = tt[:nk]; co[1::2] = col[:nk]
                fc.keyframe_points.foreach_set('co', co); fc.keyframe_points.foreach_set('interpolation', [interp] * nk); fc.update()
    return act, bag


def nonuniform_bones(ch, tol=1e-4):
    """Bones whose scale channels are not all equal in some pose of a channel table (T, n, 9)."""
    scl = ch[..., 6:9]; return set(int(i) for i in np.nonzero((np.abs(scl - scl[..., :1]) > tol).any(-1).any(0))[0])


def import_action(ob, form, table, name, length):
    """A per-cycle channel table (length + 1, n, 9) as a Blender action. Returns (action, report)."""
    sk = form.sk; n = int(length); times = RM.step_times(table)
    ch = table[times]; K = sk.skin_matrices(ch)
    loc, quat, sc = RM.trs_from_matrix(form.basis(ch))
    Kb = RM.from_blender_pose(form.pose(loc, quat, sc), np.linalg.inv(form.A[:sk.n]))
    act, bag = write_action(ob, name, times, loc, quat, sc, form.names)
    act['osrs_length'] = n; act['osrs_times'] = [int(t) for t in times]; act['osrs_fp'] = json.dumps(bone_fingerprints(bag))
    act.use_frame_range = True; act.frame_start = 0; act.frame_end = n
    return act, dict(name=name, length=n, poses=len(times), K=K, K_back=Kb, bones_nonuniform_scale=len(nonuniform_bones(ch)))


def import_creature(manifest, seqs=None, helpers='auto', log=print):
    """helpers: 'auto' = a scale helper for every bone that some imported sequence scales non-uniformly; 'none'."""
    man = json.loads(pathlib.Path(manifest).read_text()); root = pathlib.Path(manifest).resolve().parent; src = root / man['src']
    reset(); sc = bpy.context.scene
    base = B.decode_base((src / man['base_file']).read_bytes()); sk0 = RM.Skeleton(base)
    todo = [s for s in man['sequences'] if not seqs or s['seq'] in seqs]; tables = {}; need = set()
    for s in todo:
        if s['file'] not in tables: tables[s['file']] = {}
        if s['frames'] not in tables[s['file']]:
            tables[s['file']][s['frames']] = RM.channel_table(B.decode_anim((src / s['file']).read_bytes()), sk0, range(s['frames'] + 1))
        need |= nonuniform_bones(tables[s['file']][s['frames']])
    rig, sk = build_armature(base, helpers=need if helpers == 'auto' else ())
    rig['osrs_base_id'] = man['base']; rig['osrs_src'] = str(src); rig['osrs_npc'] = man['npc']
    form = Form(rig, sk); names = form.names[:sk.n]
    parts = []
    for k, f in enumerate(man['models']):
        m = B.decode_model((src / f).read_bytes()); ob = add_model(f'part{k}', m, k); ob['osrs_file'] = f
        if m['bone_weights'] is not None:
            set_weights(ob, m['bone_weights'], names)
            md = ob.modifiers.new('Armature', 'ARMATURE'); md.object = rig; ob.parent = rig
        parts.append((ob, m))
    report = dict(npc=man['npc'], name=man['name'], bones=sk.n, scale_helpers=sorted(form.helper), parts=[dict(vertices=len(m['vertices']), faces=len(m['faces'])) for _, m in parts], actions=[])
    for s in todo:
        act, r = import_action(rig, form, tables[s['file']][s['frames']], f"seq_{s['seq']}", s['frames'])
        act['osrs_seq'] = s['seq']; act['osrs_anim'] = s['anim']; act['osrs_file'] = s['file']
        # what Blender's location / quaternion / scale channels cannot hold (float32; and, without helpers, the shear
        # of the client's parent-axis scale): the worst vertex displacement in game units over every key pose
        worst = 0.0
        for ob, m in parts:
            if m['bone_weights'] is None: continue
            W = RM.weight_matrix(m['bone_weights'], sk.n)
            worst = max(worst, float(np.abs(RM.skin_fast(m['vertices'], W, r['K']) - RM.skin_fast(m['vertices'], W, r['K_back'])).max()))
        report['actions'].append(dict(name=r['name'], length=r['length'], poses=r['poses'], bones_nonuniform_scale=r['bones_nonuniform_scale'], worst_vertex_error_units=worst))
        log(f"action seq_{s['seq']}: {r['length']} cycles, {r['poses']} key poses, {r['bones_nonuniform_scale']} bones with non-uniform scale, worst vertex error of the Blender form {worst:.5f} units")
    if rig.animation_data: sc.frame_end = max(a['length'] for a in report['actions']) if report['actions'] else 100
    return rig, [ob for ob, _ in parts], report


# ---------------------------------------------------------------------------------------------------- export
def mesh_objects():
    obs = [o for o in bpy.context.scene.objects if o.type == 'MESH' and not o.hide_render]
    return sorted(obs, key=lambda o: (o.get('osrs_part', 999), o.name))
def find_rig():
    rigs = [o for o in bpy.context.scene.objects if o.type == 'ARMATURE']
    assert len(rigs) == 1, f'{len(rigs)} armatures in the scene: one is expected'
    return rigs[0]


def number_bones(rig):
    """Give every bone an osrs_index. Existing numbers are kept when they are a full 0..n-1 set; otherwise bones are
    numbered CHILD FIRST, root last, as every real base is (SKELETAL.md section 3)."""
    bones = game_bones(rig); have = [b.get('osrs_index') for b in bones]
    if all(h is not None for h in have) and sorted(have) == list(range(len(bones))): return
    order = []
    def visit(b):
        for c in sorted((c for c in bones if game_parent(c) == b), key=lambda c: c.name): visit(c)
        order.append(b)
    for r in sorted((b for b in bones if game_parent(b) is None), key=lambda b: b.name): visit(r)
    for i, b in enumerate(order): b['osrs_index'] = i


def export_base(rig):
    """The armature as a frame base dict, and the Skeleton made from it."""
    number_bones(rig); A, names = rest_A(rig); n = len(names); bones = [rig.data.bones[x] for x in names]
    assert len([b for b in bones if game_parent(b) is None]) == 1, 'a frame base has exactly one root bone'
    assert n <= 256, 'a vertex names its bones in one byte: at most 256 bones'
    G = np.empty((n, 4, 4)); kept = [False] * n; moved = {}
    for i, b in enumerate(bones):
        if b.get('osrs_rest') is not None and b.get('osrs_A') is not None:
            A0 = np.array(list(b['osrs_A']), float).reshape(4, 4); kept[i] = bool(np.array_equal(A0.astype(np.float32), A[i].astype(np.float32)))
            if not kept[i]: moved[i] = RM.CSI @ A[i] @ np.linalg.inv(A0) @ RM.CS            # the rigid change made in edit mode
        else:
            # a NEW bone. research-rig wrote its Blender rest matrix (rotation and all). A Blender bone lying along the
            # armature's Y axis (a spine, a tail: front-back) then has a game rest rotation of exactly 90 degrees about
            # x, which is the POLE of the client's own angle formula: channels decompose with a residual of 0.005 to
            # 0.007 (found on the Zodiac ray's hub bone, research-rig2). The game rest orientation of a bone is free
            # (rigmath fact 2), so a new bone is written as real bases write most of theirs (4,602 of 8,822): a pure
            # translation to its joint, unless the scene asks otherwise (rig['osrs_new_rest'] = 'blender').
            G[i] = RM.CSI @ A[i] @ RM.CS
            if rig.get('osrs_new_rest', 'translation') != 'blender':
                T = np.eye(4); T[:3, 3] = G[i][:3, 3]; G[i] = T
    parent = [game_parent(b)['osrs_index'] if game_parent(b) else -1 for b in bones]
    # global rest of the bones that still stand where they were imported: from the stored local matrices
    stored = {i: RM.col(list(b['osrs_rest'])) for i, b in enumerate(bones) if b.get('osrs_rest') is not None}
    memo = {}
    def stored_global(i):
        if i not in memo:
            p = parent[i]; memo[i] = stored[i] if p < 0 or p not in stored else stored_global(p) @ stored[i]
        return memo[i]
    for i, b in enumerate(bones):
        if b.get('osrs_rest') is None: continue
        G[i] = stored_global(i) if kept[i] else moved[i] @ stored_global(i)
    out = []; verbatim = 0
    for i, b in enumerate(bones):
        p = parent[i]
        if kept[i] and (p < 0 or kept[p]): mat = [float(x) for x in b['osrs_rest']]; verbatim += 1
        else:
            Lc = G[i] if p < 0 else np.linalg.inv(G[p]) @ G[i]; mat = [RM.f32(x) for x in RM.flat(Lc)]
        out.append(dict(parent=p if p >= 0 else -1, matrices=[mat], extra=[[float(x) for x in b['osrs_extra']] if b.get('osrs_extra') is not None else [0.0, 0.0, 0.0]]))
    g = json.loads(rig['osrs_groups']) if rig.get('osrs_groups') else dict(types=[], labels=[])
    base = dict(types=g['types'], labels=g['labels'], bones=out, poses=1, _tail=b'\0\0')
    return base, RM.Skeleton(base), A, names, verbatim


def export_model(ob, rig_names):
    """A mesh object as a model dict for bonecodec.encode_model. rig_names {bone name: index} or None (no weights)."""
    me = ob.data; tmp = None
    if any(len(p.vertices) != 3 for p in me.polygons):
        import bmesh
        tmp = me.copy(); bm = bmesh.new(); bm.from_mesh(tmp); bmesh.ops.triangulate(bm, faces=bm.faces[:]); bm.to_mesh(tmp); bm.free(); me = tmp
    nv, nf = len(me.vertices), len(me.polygons)
    co = np.empty(nv * 3, np.float32); me.vertices.foreach_get('co', co); co = co.reshape(nv, 3).astype(float)
    wm = np.array(ob.matrix_world, float); co = co @ wm[:3, :3].T + wm[:3, 3]
    V = np.rint(co @ np.linalg.inv(RM.MODEL_TO_BLENDER).T).astype(int)
    fv = np.empty(nf * 3, np.int32); me.polygons.foreach_get('vertices', fv); F = [tuple(int(x) for x in r) for r in fv.reshape(nf, 3)]
    mi = np.empty(nf, np.int32); me.polygons.foreach_get('material_index', mi)
    info = []
    for s in ob.material_slots:
        mt = MAT_RE.fullmatch(s.material.name) if s.material else None
        if not mt: raise SystemExit(f'{ob.name}: material {s.material.name if s.material else None!r} is not named hsl_H_S_L')
        info.append(mt)
    m = dict(vertices=[tuple(int(x) for x in v) for v in V], faces=F, colors=[(int(info[k][1]) << 10) | (int(info[k][2]) << 7) | int(info[k][3]) for k in mi])
    def attr(name, n):
        a = me.attributes.get(name)
        if a is None: return None
        v = np.empty(n, np.int32); a.data.foreach_get('value', v); return [int(x) for x in v]
    flat = [bool(info[k][4]) for k in mi]; raw = attr('osrs_render', nf)
    m['render_types'] = raw if raw is not None else [int(x) for x in flat] if (any(flat) or ob.get('osrs_has_render')) else None
    for key, name in FACE_ATTRS.items(): m[key] = attr(name, nf)
    if m['priorities'] is None and any(info[k][5] for k in mi): m['priorities'] = [int(info[k][5]) if info[k][5] else int(ob.get('osrs_priority', 4)) for k in mi]
    if m['depth'] is None and any(info[k][6] for k in mi): m['depth'] = [int(info[k][6] or 0) for k in mi]
    if m['alpha'] is None and any(info[k][7] for k in mi): m['alpha'] = [int(info[k][7] or 0) for k in mi]
    m['priority'] = int(ob.get('osrs_priority', 0)); m['vertex_skins'] = attr('osrs_vlabel', nv)
    ex = json.loads(ob['osrs_extra']) if ob.get('osrs_extra') else {}
    m['texture_types'] = ex.get('texture_types') or []; m['texture_triangles'] = [tuple(t) for t in ex.get('texture_triangles') or []]
    m['complex_raw'] = bytes.fromhex(ex.get('complex_raw') or ''); md = ex.get('metadata'); m['metadata'] = (tuple(md[0]), md[1]) if md else None
    stats = dict(vertices=nv, faces=nf)
    if rig_names is not None and len(ob.vertex_groups):
        gmap = {g.index: rig_names[g.name] for g in ob.vertex_groups if g.name in rig_names}
        W = []
        for v in me.vertices:
            row = [(gmap[g.group], int(round(g.weight * 255))) for g in v.groups if g.group in gmap]
            W.append([(b, w) for b, w in row if w > 0])
        m['bone_weights'] = W
        stats.update(unweighted=sum(1 for w in W if not w), sums_not_255=sum(1 for w in W if w and sum(x for _, x in w) != 255), influences=sum(len(w) for w in W))
    else: m['bone_weights'] = None
    if tmp is not None: bpy.data.meshes.remove(tmp)
    return m, stats


def read_poses(rig, names, times):
    """Blender's evaluated armature-space pose matrices at each time: (T, n, 4, 4)."""
    sc = bpy.context.scene; pbs = [rig.pose.bones[x] for x in names]; out = np.empty((len(times), len(names), 4, 4))
    for k, t in enumerate(times):
        sc.frame_set(int(t)); bpy.context.view_layer.update()
        for i, pb in enumerate(pbs): out[k, i] = pb.matrix
    return out


def export_action(rig, act, sk, A, names, base_id, pure=False, mode='baked', step=None, log=print):
    """One action as a keyframe-file dict. Returns (anim dict, report)."""
    bag = fcurve_bag(rig, act); n = int(act.get('osrs_length', act.frame_end if act.use_frame_range else act.frame_range[1]))
    src_times = [int(t) for t in act['osrs_times']] if act.get('osrs_times') is not None else None
    if step is None and src_times is not None and mode == 'baked': times = src_times
    else:
        step = int(step or act.get('osrs_step', 3)); times = sorted(set(range(0, n + 1, step)) | {n})
    source = None; srcdir = pathlib.Path(rig['osrs_src']) if rig.get('osrs_src') else None
    if act.get('osrs_file') and srcdir: source = B.decode_anim((srcdir / act['osrs_file']).read_bytes())
    sample = list(range(n + 1)) if mode == 'spline' else times
    P = read_poses(rig, names, sample); K = RM.from_blender_pose(P, np.linalg.inv(A))
    Lc = sk.locals_from_globals(K @ sk.rest); rot, tr, scl, res, det = RM.decompose(Lc); ch = np.concatenate([rot, tr, scl], -1)        # (T, n, 9)
    rep = dict(action=act.name, length=n, keys=len(times), mode=mode, decompose_residual=float(res.max()), verbatim_bones=0, rewritten_bones=0)
    keep = {}
    if source is not None and not pure and mode == 'baked' and times == src_times:
        fp_now = bone_fingerprints(bag); fp_then = json.loads(act['osrs_fp']) if act.get('osrs_fp') else {}
        want = RM.channel_table(source, sk, times)
        # A bone is untouched when its f-curves are bit-for-bit the importer's. If the armature carries constraints
        # (IK and the like can move a bone whose own curves are untouched), the evaluated pose must ALSO equal the
        # pose the importer's keys give by themselves (the Blender form of the source, recomputed here).
        constrained = any(len(pb.constraints) for pb in rig.pose.bones)
        if constrained:
            form = Form(rig, sk); gap = np.abs(P - form.pose(*RM.trs_from_matrix(form.basis(want)))).max(axis=(0, 2, 3))
        for i, bn in enumerate(names):
            if fp_now.get(bn) == fp_then.get(bn) and fp_then.get(bn) is not None:
                if not constrained or gap[i] < 1e-4: keep[i] = True
                else: rep.setdefault('guard_refused', []).append((i, float(gap[i])))
    by_slot = {}
    if source is not None:
        for c in source['channels']: by_slot.setdefault((c['kind'], c['target'] if c['kind'] == 1 else None), []).append(c)
    channels = []
    if mode == 'spline':
        chs = ch.copy(); chs[..., 0:3] = RM.unwrap(chs[..., 0:3]); kt = times; worst = 0.0
    for i in range(sk.n):
        if i in keep:
            channels += by_slot[(1, i)]; rep['verbatim_bones'] += 1; continue
        rep['rewritten_bones'] += 1
        for c in range(9):
            rest = sk.rest_channels[i, c]
            if mode == 'baked':
                v = ch[:, i, c]
                if np.ptp(v) < (2e-4 if 3 <= c <= 5 else 2e-6): v = np.full_like(v, v[0])      # float noise of a still channel: one value
                keys = RM.baked_curve(times, v, rest, n)
            else:
                v = chs[:, i, c]
                if np.ptp(v) < (2e-4 if 3 <= c <= 5 else 2e-6): v = np.full_like(v, v[0])
                sl = np.gradient(v); sl[0] = v[1] - v[0] if n else 0; sl[-1] = v[-1] - v[-2] if n else 0
                keys = RM.spline_curve(kt, v[kt], sl[kt], rest, n)
                cv = dict(before=0, after=0, weighted=0, keys=keys); worst = max(worst, max(abs(RM.evaluate(cv, float(t)) - v[t]) * (1 if c < 3 or c > 5 else 0.01) for t in range(n + 1)))
            channels.append(dict(kind=1, target=i, channel=c + 1, curve=dict(byte0=RM.BYTE0[c + 1], before=0, after=0, weighted=0, keys=keys)))
    if mode == 'spline': rep['spline_worst_channel_error'] = float(worst)
    # channels Blender does not hold
    if source is not None:
        for c in source['channels']:
            if c['kind'] != 1: channels.append(c)
    elif act.get('osrs_stance_file') and srcdir:
        st = B.decode_anim((srcdir / act['osrs_stance_file']).read_bytes()); t0 = float(act.get('osrs_stance_frame', 0))
        for c in st['channels']:
            if c['kind'] == 4:
                v = RM.f32(RM.evaluate(c['curve'], t0))
                channels.append(dict(kind=4, target=c['target'], channel=16, curve=dict(byte0=c['curve']['byte0'], before=0, after=0, weighted=0, keys=[[0, v, float(n), 0.0, RM.FMAX, RM.FMAX], [n, v, float(n), 0.0, RM.FMAX, RM.FMAX]])))
    if source is not None and not pure and len(keep) == sk.n and mode == 'baked':
        channels = source['channels']                                           # nothing touched: the file's own channel order
    anim = dict(version=1, base=int(base_id), range=[0, n], pose=0, channels=channels)
    rep['channels'] = len(channels); rep['distinct_poses'] = len(times) if mode == 'baked' else n + 1
    return anim, rep


def export_scene(outdir, pure=False, mode='baked', step=None, actions=None, base_id=None, log=print):
    out = pathlib.Path(outdir); out.mkdir(parents=True, exist_ok=True)
    rig = find_rig(); assert np.allclose(np.array(rig.matrix_world), np.eye(4)), 'the armature object must sit at the origin, unrotated, unscaled'
    base, sk, A, names, verbatim = export_base(rig); bid = int(base_id if base_id is not None else rig.get('osrs_base_id', 0))
    data = B.encode_base(base); assert B.encode_base(B.decode_base(data)) == data
    (out / f'base_{bid}.dat').write_bytes(data)
    report = dict(base=bid, bones=sk.n, base_file=f'base_{bid}.dat', bones_rest_verbatim=verbatim, models=[], actions=[])
    index = {n: i for i, n in enumerate(names)}
    saved = (rig.animation_data.action, rig.animation_data.action_slot) if rig.animation_data else None
    rig.data.pose_position = 'REST'
    for k, ob in enumerate(mesh_objects()):
        m, stats = export_model(ob, index); enc = B.encode_model(m, hints=False); back = B.decode_model(enc)
        diff = B.model_equal(m, back); assert not diff, f'model round trip changed {diff}'
        (out / f'part-{k}.bin').write_bytes(enc); stats.update(file=f'part-{k}.bin', bytes=len(enc), object=ob.name); report['models'].append(stats)
        log(f"model {ob.name}: {stats}")
    rig.data.pose_position = 'POSE'
    for act in sorted(bpy.data.actions, key=lambda a: a.name):
        if actions and act.name not in actions: continue
        if not len(act.slots): continue
        anim, rep = export_action(rig, act, sk, A, names, bid, pure, mode, step, log); enc = B.encode_anim(anim)
        assert B.encode_anim(B.decode_anim(enc)) == enc
        fn = f'anim_{act.name}.dat'; (out / fn).write_bytes(enc); rep.update(file=fn, bytes=len(enc), seq=act.get('osrs_seq'), source=act.get('osrs_file')); report['actions'].append(rep)
        log(f"action {act.name}: {rep}")
    if saved and saved[0]: rig.animation_data.action = saved[0]; rig.animation_data.action_slot = saved[1]
    (out / 'export.json').write_text(json.dumps(report, indent=1))
    return report
