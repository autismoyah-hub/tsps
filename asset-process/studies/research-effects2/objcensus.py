"""objcensus.py: every object definition that names a sequence (opcode 24), with its model and sequence numbers.
usage: objcensus.py            -> data/objanim.jsonl (one row per animated object)
       objcensus.py summary    -> data/objanim-summary.txt (counts by job from the name; centiles)
       objcensus.py grep REGEX -> matching rows, one line each
Row: id, name, models, shapes, size, walkable (opcode 17: no collision), randomstart (opcode 89 absent), ambient, contrast,
anim, anim_name, s: kind classic/skeletal/empty, frames, cycles, loop (loop_frames; -1 = plays once then RESTS), restart,
m: faces, verts, colours, alpha share / values, flat, box, labels (the combined models of its placement shape),
uses: transform kinds of a classic sequence; posed size range and alpha range over it (as census.py).
"""
import sys, json, re, collections, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from fxlib import *
import objlib, census

OUT = HERE / 'data'; OUT.mkdir(exist_ok=True)

JOBS = [('pool / liquid / acid', r'pool|acid|puddle|blood|slime|ooze|lava(?!_?fall)|venom|poison|goo|spill|sludge'),
        ('fire / flame / brazier', r'fire|flame|brazier|torch|burn|candle|lantern|lamp|ember'),
        ('smoke / gas / cloud / vent', r'smoke|gas|cloud|vent|steam|geyser|fog|mist|fume|vapour|spore'),
        ('crystal / orb / light', r'crystal|orb|light|glow|beam|star|shard|gem|energy|spark'),
        ('portal / rift / barrier', r'portal|rift|barrier|forcefield|gate_?way|teleport|vortex|shield|wall_?of|ward'),
        ('pillar / rock / falling', r'pillar|column|rock|boulder|debris|rubble|collapse|stalag|icicle|spike'),
        ('tile / floor mark / symbol', r'tile|floor|symbol|rune|glyph|sigil|mark|circle|pattern|trap'),
        ('water / fountain / fall', r'water|fountain|fall|wave|ripple|splash|drip|stream|river'),
        ('creature part / tentacle / egg', r'tentacle|egg|cocoon|web|head|hand|claw|eye|tendril|vine|root|flower|plant|tree|mushroom'),
        ('machine / lever / door', r'door|gate|lever|machine|wheel|gear|pump|engine|chest|altar|switch|crank|bell|flag|banner')]


def job(name):
    for j, pat in JOBS:
        if re.search(pat, name): return j
    return 'other'


def combined(d, shape):
    """The models of the placement shape as one decoded model (vertices offset-free concatenation), recoloured."""
    ms = [oc.model(m) for m in objlib.models_for(d, shape)]
    if not ms: return None
    if len(ms) == 1: m = ms[0]
    else:
        m = dict(vertices=[], faces=[], colors=[], alpha=[], render_types=[], face_textures=[], priorities=[], vertex_skins=[], face_skins=[], format=ms[0]['format'],
                 has_bone_weights=any(x['has_bone_weights'] for x in ms))
        for x in ms:
            o = len(m['vertices'])
            m['vertices'] += x['vertices']; m['faces'] += [(a + o, b + o, c + o) for a, b, c in x['faces']]
            for k in ('colors', 'alpha', 'render_types', 'face_textures', 'priorities', 'face_skins'): m[k] += list(x[k])
            m['vertex_skins'] += list(x['vertex_skins'] or [0] * len(x['vertices']))
    if d['recolor']:
        rc = dict(d['recolor']); m = dict(m, colors=[rc.get(c, c) for c in m['colors']])
    return m


def main():
    nm = names(6); sn = names(7); rows = []; bad = 0
    for oid in sorted(objlib.objects()):
        try: d = objlib.decode(objlib.objects()[oid])
        except Exception: bad += 1; continue
        if d['anim'] is None: continue
        shape = objlib.shape_for(d)
        row = dict(id=oid, name=nm.get(oid, ''), display=d['name'], models=d['models'], shapes=d['shapes'], shape=shape, size=d['size'], walkable=d['interact'] == 0, randomstart=d['randomstart'],
                   ambient=d['ambient'], contrast=d['contrast'], anim=d['anim'], anim_name=sn.get(d['anim'], ''), scale=d['scale'], recolor=len(d['recolor']), transforms=bool(d['transforms']),
                   ops=[t for t in d['ops'].values() if t and t.lower() != 'hidden'])
        m = None
        try:
            m = combined(d, shape); row['m'] = census.model_stats(m) if m else dict(faces=0, verts=0)
        except Exception as e: row['m'] = dict(error=str(e)[:60])
        try:
            s = al.sequence(d['anim'])
            if s['cached_animation'] is not None:
                row['s'] = dict(kind='skeletal', frames=objlib.seq_cycles(s), cycles=objlib.seq_cycles(s), loop=s['loop_frames'], restart=s['restart'])
            else:
                row['s'] = census.seq_stats(d['anim'], m); row['s']['restart'] = s['restart']
        except Exception as e: row['s'] = dict(kind='missing', error=str(e)[:60])
        rows.append(row)
    with (OUT / 'objanim.jsonl').open('w', encoding='utf-8') as f:
        for r in rows: f.write(json.dumps(r) + '\n')
    print(len(objlib.objects()), 'object definitions;', bad, 'not decoded;', len(rows), 'with a sequence')


def load(): return [json.loads(l) for l in (OUT / 'objanim.jsonl').open(encoding='utf-8')]


def cent(v, ps=(10, 50, 90)):
    v = sorted(v)
    return ' / '.join(str(v[min(len(v) - 1, int(len(v) * p / 100))]) for p in ps) if v else '-'


def summary():
    rows = load(); out = []
    ok = [r for r in rows if r['m'].get('faces') and r['s'].get('kind') in ('classic', 'skeletal')]
    out.append(f'animated objects {len(rows)}; with a drawable model and a sequence {len(ok)}; distinct sequences {len(set(r["anim"] for r in rows))}; distinct model sets {len(set(tuple(r["models"]) for r in rows))}')
    k = collections.Counter(r['s'].get('kind') for r in rows); out.append(f'sequence kinds: {dict(k)}')
    out.append(f'random start (no opcode 89): {sum(r["randomstart"] for r in rows)}; fixed start (opcode 89): {sum(not r["randomstart"] for r in rows)}')
    cl = [r for r in ok if r['s']['kind'] == 'classic']
    out.append(f'loops (loop field set): {sum(1 for r in ok if r["s"].get("loop", -1) != -1)}; plays once then rests (no loop field): {sum(1 for r in ok if r["s"].get("loop", -1) == -1)}')
    out.append(f'walkable (opcode 17): {sum(r["walkable"] for r in rows)}; with a menu option: {sum(1 for r in rows if r["ops"])}; size 1x1: {sum(1 for r in rows if r["size"] == [1, 1])}; multi-model: {sum(1 for r in rows if len(r["models"]) > 1)}')
    out.append(f'ambient 10/50/90: {cent([r["ambient"] for r in rows])}; contrast (stored) 10/50/90: {cent([r["contrast"] for r in rows])}; ambient set: {sum(1 for r in rows if r["ambient"])}, contrast set: {sum(1 for r in rows if r["contrast"])}')
    out.append(f'faces 10/50/90: {cent([r["m"]["faces"] for r in ok])}; largest dimension: {cent([max(r["m"]["box"]) for r in ok])}; colours90: {cent([r["m"]["colours90"] for r in ok])}')
    out.append(f'any translucent face: {100 * sum(1 for r in ok if r["m"]["alpha_faces"]) / len(ok):.0f}%; wholly: {100 * sum(1 for r in ok if r["m"]["alpha_share"] > 0.99) / len(ok):.0f}%; any flat face {100 * sum(1 for r in ok if r["m"]["flat"]) / len(ok):.0f}%; textured {100 * sum(1 for r in ok if r["m"]["textured"]) / len(ok):.0f}%')
    out.append(f'cycles of one pass 10/50/90: {cent([r["s"]["cycles"] for r in ok])}; frames: {cent([r["s"]["frames"] for r in ok])}')
    uses = collections.Counter(u for r in cl for u in r['s'].get('uses', {}))
    out.append(f'classic sequences {len(cl)}: transform kinds used: {dict(uses)}; grows 1.5x or more: {sum(1 for r in cl if r["s"].get("size_max", 0) >= 1.5 * max(1, r["s"].get("size_min", 1)))}')
    out.append('')
    out.append('job (by name) | n | faces 10/50/90 | largest dim 50 | translucent | wholly | loops | random start | walkable | cycles 10/50/90 | classic/skeletal | alpha animated | scale animated')
    by = collections.defaultdict(list)
    for r in ok: by[job(r['name'])].append(r)
    for j, rs in sorted(by.items(), key=lambda kv: -len(kv[1])):
        c = [r for r in rs if r['s']['kind'] == 'classic']
        out.append(f'{j} | {len(rs)} | {cent([r["m"]["faces"] for r in rs])} | {cent([max(r["m"]["box"]) for r in rs], (50,))} | {100 * sum(1 for r in rs if r["m"]["alpha_faces"]) / len(rs):.0f}% | {100 * sum(1 for r in rs if r["m"]["alpha_share"] > 0.99) / len(rs):.0f}% | '
                   f'{100 * sum(1 for r in rs if r["s"].get("loop", -1) != -1) / len(rs):.0f}% | {100 * sum(r["randomstart"] for r in rs) / len(rs):.0f}% | {100 * sum(r["walkable"] for r in rs) / len(rs):.0f}% | {cent([r["s"]["cycles"] for r in rs])} | '
                   f'{len(c)}/{len(rs) - len(c)} | {100 * sum(1 for r in c if "alpha" in r["s"].get("uses", {})) / max(1, len(c)):.0f}% | {100 * sum(1 for r in c if "scale" in r["s"].get("uses", {})) / max(1, len(c)):.0f}%')
    (OUT / 'objanim-summary.txt').write_text('\n'.join(out) + '\n', encoding='utf-8')
    print('\n'.join(out))


def line(r):
    m, s = r['m'], r['s']
    return (f'{r["id"]} {r["name"]} | models {r["models"]} shape {r["shape"]} size {r["size"]} {"walk" if r["walkable"] else "solid"} {"random" if r["randomstart"] else "fixed"} amb {r["ambient"]} con {r["contrast"]} | '
            f'{m.get("faces")}f alpha {m.get("alpha_share")} {m.get("alpha_values", [])[:6]} box {m.get("box")} | seq {r["anim"]} {r["anim_name"]} {s.get("kind")} {s.get("frames")}fr {s.get("cycles")}cy loop {s.get("loop")} uses {s.get("uses")} size {s.get("size_min")}..{s.get("size_max")}')


if __name__ == '__main__':
    a = sys.argv[1:]
    if not a: main()
    elif a[0] == 'summary': summary()
    elif a[0] == 'grep':
        for r in load():
            if re.search(a[1], r['name']): print(line(r))
