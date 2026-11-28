"""Measure the end-game bosses: definition, models, skeleton, sequences, mesh measures, style check, relatives.

usage: boss_measure.py            -> data/bosses.json, data/bosses-table.txt
Reads the NPC census (studies/research-npc/census, made by asset_npc.exe census from this cache), the cache itself for
spot animations and objects (archive 2 groups 13 and 6; decoders ported from the client's spot_animation.rs and
object.rs), and the assembled rest mesh through asset_npc.exe merged (cmesh.load; its cache repointed to data/merged).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, collections
HERE = pathlib.Path(__file__).parent
for p in (_AP.fix(r'<ROOT>\studies\research-creatures'), _AP.fix(r'<ROOT>\studies\research-npc'), _AP.fix(r'<ROOT>\studies\research-animation')):
    sys.path.insert(0, p)
import cmesh, style_npc, npc, animlib
cmesh.DATA = HERE / 'data'
C = pathlib.Path(_AP.fix(r'<ROOT>\studies\research-npc\census'))
def rd(n): return {j['id']: j for j in (json.loads(l) for l in (C / f'{n}.jsonl').read_text(encoding='utf-8').splitlines() if l.strip())}
NPCS, MODELS, SEQS, SKELS = rd('npcs'), rd('models'), rd('sequences'), rd('skeletons')

# (npc id, short label, set)   set: cox / tob / toa = raids; own = the owner's other named; other = other end-game bosses
BOSSES = [
 (7541, 'Tekton', 'cox'), (7566, 'Vasa Nistirio', 'cox'), (7530, 'Vespula', 'cox'), (7527, 'Vanguard', 'cox'), (7563, 'Muttadile', 'cox'),
 (8360, 'Maiden of Sugadinti', 'tob'), (8359, 'Pestilent Bloat', 'tob'), (8355, 'Nylocas Vasilias', 'tob'), (8388, 'Sotetseg', 'tob'), (8340, 'Xarpus', 'tob'),
 (8370, 'Verzik P1 (throne)', 'tob'), (8372, 'Verzik P2', 'tob'), (8374, 'Verzik P3', 'tob'),
 (11789, 'Akkha', 'toa'), (11778, 'Ba-Ba', 'toa'), (11719, 'Kephri', 'toa'), (11730, 'Zebak', 'toa'), (11753, "Elidinis' Warden P2", 'toa'), (11756, "Tumeken's Warden P2", 'toa'),
 (11761, "Elidinis' Warden P3", 'toa'), (11762, "Tumeken's Warden P3", 'toa'), (11751, 'Obelisk', 'toa'),
 (11278, 'Nex', 'own'), (9425, 'The Nightmare', 'own'), (9416, "Phosani's Nightmare", 'own'), (14707, 'Doom of Mokhaiotl', 'own'), (14709, 'Doom (burrowed)', 'own'), (15742, 'Maggot King', 'own'),
 (8061, 'Vorkath', 'other'), (2042, 'Zulrah', 'other'), (8615, 'Alchemical Hydra', 'other'), (319, 'Corporeal Beast', 'other'), (5862, 'Cerberus', 'other'),
 (12077, 'Phantom Muspah', 'other'), (12223, 'Vardorvis', 'other'), (12191, 'Duke Sucellus', 'other'), (12214, 'The Leviathan', 'other'), (12204, 'The Whisperer', 'other'),
 (12821, 'Sol Heredit', 'other'), (13668, 'Araxxor', 'other'), (13685, 'Amoxliatl', 'other'), (14009, 'The Hueycoatl', 'other'), (12596, 'Branda (Royal Titans)', 'other'),
 (14147, 'Eldric (Royal Titans)', 'other'), (14176, 'Yama', 'other'),
]
# attack / death not in the server tables: INFERRED by me from the skeleton's sequence list and frame counts, then looked at
INFERRED = {}
inf = HERE / 'data' / 'inferred-seqs.json'
if inf.exists(): INFERRED = {int(k): v for k, v in json.loads(inf.read_text()).items()}

class R:
    def __init__(s, d): s.d = d; s.p = 0
    def u8(s): s.p += 1; return s.d[s.p - 1]
    def u16(s): s.p += 2; return int.from_bytes(s.d[s.p - 2:s.p], 'big')
    def u24(s): s.p += 3; return int.from_bytes(s.d[s.p - 3:s.p], 'big')
    def u32(s): s.p += 4; return int.from_bytes(s.d[s.p - 4:s.p], 'big')
    def skip(s, n): s.p += n
    def string(s):
        e = s.d.index(0, s.p); t = s.d[s.p:e].decode('cp1252', 'replace'); s.p = e + 1; return t
def decode_spot(data):
    r = R(data); d = dict(model=0, seq=-1, scale=[128, 128], ambient=0, contrast=0)
    while True:
        op = r.u8()
        if op == 0: break
        elif op == 1: d['model'] = r.u16()
        elif op == 3: d['model'] = r.u32()
        elif op == 2: d['seq'] = r.u16()
        elif op == 4: d['scale'][0] = r.u16()
        elif op == 5: d['scale'][1] = r.u16()
        elif op == 6: r.u16()
        elif op == 7: d['ambient'] = r.u8()
        elif op == 8: d['contrast'] = r.u8()
        elif op == 10: pass
        elif op in (40, 41): r.skip(4 * r.u8())
        else: raise ValueError(f'spot opcode {op}')
    return d
def decode_object(data):
    r = R(data); d = dict(name='null', models=[], anim=None, size=[1, 1])
    while True:
        op = r.u8()
        if op == 0: break
        elif op in (1, 5):
            n = r.u8()
            for _ in range(n):
                d['models'].append(r.u16())
                if op == 1: r.u8()
        elif op in (6, 7):
            n = r.u8()
            for _ in range(n):
                d['models'].append(r.u32())
                if op == 6: r.u8()
        elif op == 2: d['name'] = r.string()
        elif op == 3: r.string()
        elif op == 14: d['size'][0] = r.u8()
        elif op == 15: d['size'][1] = r.u8()
        elif op in (19, 28, 29, 39, 69, 75, 81, 91, 95, 96): r.u8()
        elif op == 24:
            v = r.u16(); d['anim'] = None if v == 65535 else v
        elif 30 <= op <= 34: r.string()
        elif op == 100: r.skip(2); r.string()
        elif op in (101, 102): r.skip(1 + 4 + (2 if op == 102 else 0) + 8); r.string()
        elif op in (40, 41): r.skip(4 * r.u8())
        elif op in (61, 65, 66, 67, 68, 70, 71, 72, 82, 200): r.u16()
        elif op in (77, 92):
            r.skip(4 + (2 if op == 92 else 0)); n = r.u8(); r.skip(2 * (n + 1))
        elif op == 78: r.skip(4)
        elif op == 79: r.skip(6); r.skip(2 * r.u8())
        elif op == 93: r.skip(6)
        elif op == 249:
            for _ in range(r.u8()):
                k = r.u8(); r.u24()
                if k == 1: r.string()
                elif k == 2: r.skip(8)
                else: r.u32()
    return d
def load_relatives():
    f = HERE / 'data' / 'relatives.json'
    if f.exists(): return json.loads(f.read_text())
    c = animlib.cache(); spots = {}; objs = {}; bad = [0, 0]
    for i, data in c.files(2, 13).items():
        try: spots[i] = decode_spot(data)
        except Exception: bad[0] += 1
    for i, data in c.files(2, 6).items():
        try:
            o = decode_object(data)
            if o['anim'] is not None or o['name'] != 'null': objs[i] = o
        except Exception: bad[1] += 1
    out = dict(spots=spots, objects=objs, undecoded=bad)
    f.write_text(json.dumps(out)); return out

def skeleton_of(seq):
    q = SEQS.get(seq); return (q['skeleton'], q['kind']) if q else (-1, 'none')

def main():
    rel = load_relatives()
    spots = {int(k): v for k, v in rel['spots'].items()}; objs = {int(k): v for k, v in rel['objects'].items()}
    print('spot animations', len(spots), 'objects kept', len(objs), 'undecoded', rel['undecoded'])
    seq_by_skel = collections.defaultdict(list)
    for s in SEQS.values():
        if s['skeleton'] >= 0: seq_by_skel[s['skeleton']].append(s)
    npc_by_skel = collections.defaultdict(list)
    for n in NPCS.values():
        for slot in (0, 3):
            if n['anims'][slot] >= 0:
                sk, _ = skeleton_of(n['anims'][slot])
                if sk >= 0: npc_by_skel[sk].append(n); break
    spot_by_skel = collections.defaultdict(list)
    for i, s in spots.items():
        sk, _ = skeleton_of(s['seq'])
        if sk >= 0: spot_by_skel[sk].append(i)
    obj_by_skel = collections.defaultdict(list)
    for i, o in objs.items():
        if o['anim'] is not None:
            sk, _ = skeleton_of(o['anim'])
            if sk >= 0: obj_by_skel[sk].append(i)
    out = {}
    for nid, label, group in BOSSES:
        d = NPCS[nid]; ms = [MODELS[m] for m in d['models'] if m in MODELS]
        r = dict(id=nid, label=label, set=group, name=d['name'], size=d['size'], combat=d['combat'], models=d['models'], scale=d['scale'], ambient=d['ambient'],
                 contrast=d['contrast'], stand=d['anims'][0], walk=d['anims'][3], transforms=d['transforms'], recolours=len(d['recolor']), retextures=d['retexture'],
                 zbuffer=d['zbuffer'])
        r['faces_files'] = sum(m['faces'] for m in ms); r['verts_files'] = sum(m['verts'] for m in ms)
        r['weighted'] = any(m['weights'] for m in ms); r['labels_files'] = len({l for m in ms for l in m['labels']})
        r['has_labels'] = any(m['has_labels'] for m in ms)
        r['textured_faces'] = sum(m['textures'] for m in ms); r['alpha_faces'] = sum(m['alpha'] for m in ms); r['flat_faces'] = sum(m['flat'] for m in ms)
        r['depth_offset_faces'] = sum(m['depth_offsets'] for m in ms)
        same = [n for n in NPCS.values() if n['name'] == d['name']]
        r['forms_ids'] = len(same); r['forms_model_lists'] = len({tuple(n['models']) for n in same}); r['forms_stands'] = len({n['anims'][0] for n in same})
        r['forms_transforming'] = sum(1 for n in same if n['transforms']); r['forms_sizes'] = sorted({n['size'] for n in same})
        sk, kind = skeleton_of(d['anims'][0]) if d['anims'][0] >= 0 else skeleton_of(d['anims'][3]) if d['anims'][3] >= 0 else (-1, 'none')
        r['skeleton'] = sk; r['stand_kind'] = kind
        if sk >= 0:
            S = seq_by_skel[sk]; r['seqs'] = len(S); r['seqs_skeletal'] = sum(1 for s in S if s['kind'] == 'cached')
            r['seq_ids'] = sorted(s['id'] for s in S); r['seq_ids_skeletal'] = sorted(s['id'] for s in S if s['kind'] == 'cached')
            N = npc_by_skel[sk]; r['skel_npcs'] = len(N); r['skel_names'] = sorted({n['name'] for n in N}); r['skel_model_lists'] = len({tuple(n['models']) for n in N})
            b = SKELS.get(sk, {}); r['skel_groups'] = b.get('groups'); r['skel_bones'] = b.get('bones'); r['skel_labels'] = len(b.get('labels', []))
            r['skel_spots'] = sorted(spot_by_skel[sk]); r['skel_spot_models'] = sorted({spots[i]['model'] for i in spot_by_skel[sk]})
            r['skel_objects'] = [(i, objs[i]['name']) for i in sorted(obj_by_skel[sk])]
        try:
            a, de, where, listed = npc.server_anims(nid)
        except Exception as e:
            a = de = None; where = f'error {e}'; listed = None
        r['attack'], r['death'], r['anims_from'] = a, de, where; r['server_listed'] = listed
        if nid in INFERRED:
            for k, v in INFERRED[nid].items():
                if r.get(k) is None: r[k] = v; r['anims_from'] += f'+INFERRED {k}'
        if d['models']:
            try:
                m = cmesh.load(f'npc:{nid}'); meas = cmesh.measure(m, tuple(d['scale']))
                r['measure'] = meas
                outs, lines = style_check(meas); r['style_game_out'], r['style_recent_out'], r['style_lines'] = outs['GAME'], outs['RECENT'], lines
                A = m['A']; r['alpha_face_share_any'] = float(((A > 0) & (A < 255)).mean()); r['undrawn_faces'] = int((A == 255).sum())
            except SystemExit as e:
                r['measure_error'] = str(e)[:200]
        out[str(nid)] = r
        me = r.get('measure', {})
        print(f"{nid:6d} {label[:24]:24s} size {d['size']} faces {r['faces_files']:5d} verts {r['verts_files']:5d} {'WEIGHTS' if r['weighted'] else 'labels %3d' % r['labels_files']} base {sk:5d} ({kind}) seqs {r.get('seqs', 0):3d}/{r.get('seqs_skeletal', 0):3d} skel-npcs {r.get('skel_npcs', 0):3d} forms {r['forms_ids']}/{r['forms_model_lists']} col {me.get('colours', '-')} attack {a} death {de} [{where}]", flush=True)
    # The Great Olm is SCENERY: three animated objects (found by id beside the raid's other objects; names are disguises).
    # Measured and drawn here through the NPC harness as defs of the same model lists (object lighting differs slightly).
    for key, obj, attack in (('olm-head', 29881, 7345), ('olm-hand-rock', 29884, 7356), ('olm-hand-crystal', 29887, 7352)):
        j = json.loads((HERE / 'data' / f'{key}.json').read_text()); entry = 'def:' + str(HERE / 'data' / f'{key}.json')
        m = cmesh.load(entry); meas = cmesh.measure(m, (128, 128)); sk, kind = skeleton_of(j['stand']); S = seq_by_skel[sk]
        outs, lines = style_check(meas); o = objs[obj]
        r = dict(id=key, label=j['name'], set='cox', name=j['name'], size=max(o['size']), object_size=o['size'], combat=0, models=[int(x[6:]) for x in j['models']], scale=[128, 128],
                 ambient=0, contrast=0, stand=j['stand'], walk=-1, transforms=False, entry=entry, faces_files=int(len(m['F'])), verts_files=int(len(m['V'])), weighted=False,
                 labels_files=int(len(set(m['L'].tolist()))), has_labels=True, textured_faces=None, skeleton=sk, stand_kind=kind, seqs=len(S),
                 seqs_skeletal=sum(1 for s in S if s['kind'] == 'cached'), seq_ids=sorted(s['id'] for s in S), skel_npcs=0, skel_model_lists=0,
                 skel_objects=[(i, objs[i]['name']) for i in sorted(obj_by_skel[sk])], skel_spots=sorted(spot_by_skel[sk]), attack=attack, death=None,
                 anims_from='INFERRED attack', measure=meas, style_game_out=outs['GAME'], style_recent_out=outs['RECENT'], style_lines=lines, is_object=True,
                 forms_ids=len(obj_by_skel[sk]), forms_model_lists=len({tuple(objs[i]['models']) for i in obj_by_skel[sk]}), forms_transforming=0,
                 alpha_face_share_any=float(((m['A'] > 0) & (m['A'] < 255)).mean()), undrawn_faces=int((m['A'] == 255).sum()),
                 skel_groups=SKELS.get(sk, {}).get('groups'), skel_bones=SKELS.get(sk, {}).get('bones'))
        out[key] = r
        print(f"{key:18s} OBJECT {obj} size {o['size']} faces {r['faces_files']} verts {r['verts_files']} labels {r['labels_files']} base {sk} ({kind}) seqs {r['seqs']}/{r['seqs_skeletal']} objects on base {r['skel_objects']} col {meas['colours']}")
    (HERE / 'data' / 'bosses.json').write_text(json.dumps(out, indent=1))

def style_check(meas):
    outs = dict(GAME=0, RECENT=0); lines = []
    for key, desc, lo, hi, kind in style_npc.RULES:
        x = meas.get(key)
        if x is None: continue
        ok = (lo is None or x >= lo) and (hi is None or x <= hi)
        if not ok: outs[kind] += 1; lines.append(f'{kind} {key}={round(x, 3)} (limit {lo}..{hi})')
    return outs, lines
if __name__ == '__main__':
    main()
