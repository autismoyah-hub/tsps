"""summary.py: numbers from data/spots.jsonl. usage: summary.py [tokens|jobs|all]  -> prints; `all` also writes data/census.txt"""
import sys, json, statistics as st, collections, pathlib, re
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from fxlib import *

rows = [json.loads(l) for l in (HERE / 'data' / 'spots.jsonl').open(encoding='utf-8')]
ok = [r for r in rows if 'faces' in r['m'] and r['m']['faces'] > 0]


def pct(v, ps=(10, 25, 50, 75, 90)):
    v = sorted(v)
    if not v: return 'n=0'
    return ' / '.join(str(round(v[min(len(v) - 1, int(len(v) * p / 100))], 1)) for p in ps)


# job by name. Order matters: the first rule that matches wins.
JOBS = [
    ('projectile', r'travel|projectile|_proj(?![a-z])|projanim|arrow(?!_?(launch|pullback))|_bolt_?travel|_flying|_missile|knife(?!.*launch)|dart(?!.*launch)|javelin|thrownaxe|_shot$|_ball$|_orb$|spell_?travel|_trail'),
    ('impact', r'impact|_hit(?![a-z])|splash|explosion|explode|_burst|_land(?![a-z])|landing|_boom'),
    ('launch/cast', r'launch|casting|_cast(?![a-z])|pullback|_charge|charging|_draw$|muzzle|_fire$|_firing|_start$|windup'),
    ('ground hazard', r'pool|puddle|_fire(?![a-z])|flame|lava|acid|poison_?(cloud|gas)|_gas|smoke|_spike|vent|mushroom|spore|floor|ground|_tile|trap|_hazard|_fissure|_crack|web'),
    ('telegraph', r'shadow|marker|warning|telegraph|target|indicator|_aoe|_ring$|falling|_rock|boulder|_drop'),
    ('aura/shield', r'aura|shield|barrier|_buff|protect|ward|bubble|glow|vengeance|_heal|_curse|overhead|halo|_immune|freeze|frozen|ice_?barrage|stun|_bind|entangle|snare|teleport|_tele'),
]


def job(name):
    for j, pat in JOBS:
        if re.search(pat, name): return j
    return 'other'


def tokens():
    c = collections.Counter()
    for r in rows:
        for t in re.split(r'[_0-9]+', r['name']):
            if t: c[t] += 1
    print(' '.join(f'{t}:{n}' for t, n in c.most_common(400)))


def block(title, rs, out):
    m = [r['m'] for r in rs]
    cl = [r for r in rs if r['s'].get('kind') == 'classic']
    out.append(f'--- {title}: n={len(rs)}')
    out.append(f'  faces 10/25/50/75/90: {pct([x["faces"] for x in m])}   max {max(x["faces"] for x in m)}')
    out.append(f'  verts: {pct([x["verts"] for x in m])}')
    out.append(f'  largest dimension (model units, before scale): {pct([max(x["box"]) for x in m])}')
    out.append(f'  colours: {pct([x["colours"] for x in m])}; covering 90% of surface: {pct([x["colours90"] for x in m])}')
    n = len(rs)
    tr = [x for x in m if x['alpha_faces']]
    out.append(f'  any translucent face: {len(tr)} ({100 * len(tr) / n:.0f}%); fully translucent (share>=0.99): {sum(1 for x in m if x["alpha_share"] >= 0.99)} ({100 * sum(1 for x in m if x["alpha_share"] >= 0.99) / n:.0f}%)')
    if tr: out.append(f'    among those: translucent share of surface {pct([x["alpha_share"] for x in tr])}; mean alpha of translucent models {pct([x["mean_alpha"] / max(x["alpha_share"], 0.01) for x in tr])} (0 opaque .. 255 invisible)')
    out.append(f'  any flat-shaded face: {sum(1 for x in m if x["flat"])} ({100 * sum(1 for x in m if x["flat"]) / n:.0f}%); all flat: {sum(1 for x in m if x["flat"] == x["faces"])}; textured: {sum(1 for x in m if x["textured"])} ({100 * sum(1 for x in m if x["textured"]) / n:.0f}%)')
    out.append(f'  bone weights on the model: {sum(1 for x in m if x["weights"])}; vertex labels: {pct([x["vlabels"] for x in m])}; face labels: {pct([x["flabels"] for x in m])}')
    kinds = collections.Counter(r['s'].get('kind') for r in rs)
    out.append(f'  sequence: {dict(kinds)}')
    if cl:
        out.append(f'  classic: frames {pct([r["s"]["frames"] for r in cl])}; cycles (20 ms) {pct([r["s"]["cycles"] for r in cl])}; = seconds {pct([r["s"]["cycles"] * 0.02 for r in cl])}')
        out.append(f'    frame hold (cycles): {pct([l for r in cl for l in r["s"]["lengths"]])}; groups in base {pct([r["s"].get("groups", 0) for r in cl])}')
        u = collections.Counter()
        combo = collections.Counter()
        for r in cl:
            us = r['s'].get('uses', {})
            for k in us: u[k] += 1
            combo['+'.join(sorted(us)) or 'nothing'] += 1
        out.append(f'    transform kinds used: {dict(u)} of {len(cl)}')
        out.append(f'    combinations: {dict(combo.most_common(12))}')
        g = [r['s']['size_max'] / max(1, r['s']['size_min']) for r in cl if 'size_max' in r['s']]
        out.append(f'    size change within the sequence (max/min of largest dimension): {pct(g)}; grows >=1.5x: {sum(1 for x in g if x >= 1.5)} of {len(g)}')
        lp = collections.Counter('loops' if r['s']['loop'] not in (-1, 65535) else 'once' for r in cl)
        out.append(f'    {dict(lp)}')
    sc = collections.Counter((r['scale'][0], r['scale'][1]) for r in rs)
    out.append(f'  definition: scale not 128: {sum(1 for r in rs if r["scale"] != [128, 128])}; commonest {sc.most_common(5)}; rotation set: {sum(1 for r in rs if r["rotation"])}; ambient set: {sum(1 for r in rs if r["ambient"])} {collections.Counter(r["ambient"] for r in rs).most_common(5)}; contrast set: {sum(1 for r in rs if r["contrast"])} {collections.Counter(r["contrast"] for r in rs).most_common(5)}; recoloured: {sum(1 for r in rs if r["recolor"])}')


def summary():
    out = []
    out.append(f'spot animations: {len(rows)}; with a decodable model that has faces: {len(ok)}; distinct models {len(set(r["model"] for r in rows))}; distinct sequences {len(set(r["sequence"] for r in rows if r["sequence"] >= 0))}')
    block('ALL', ok, out)
    block('ids 0..999 (the old game)', [r for r in ok if r['id'] < 1000], out)
    block('ids 1000..1999', [r for r in ok if 1000 <= r['id'] < 2000], out)
    block('ids 2000..2999', [r for r in ok if 2000 <= r['id'] < 3000], out)
    block('ids 3000+ (newest)', [r for r in ok if r['id'] >= 3000], out)
    by = collections.defaultdict(list)
    for r in ok: by[job(r['name'])].append(r)
    for j in [j for j, _ in JOBS] + ['other']:
        block(f'JOB by name: {j}', by[j], out)
    txt = '\n'.join(out)
    print(txt)
    (HERE / 'data' / 'census.txt').write_text(txt, encoding='utf-8')


if __name__ == '__main__':
    a = sys.argv[1] if len(sys.argv) > 1 else 'all'
    if a == 'tokens': tokens()
    elif a == 'grep':
        for r in rows:
            if re.search(sys.argv[2], r['name']):
                m, s = r['m'], r['s']
                print(r['id'], r['name'], '| model', r['model'], 'f', m.get('faces'), 'box', m.get('box'), 'alpha', m.get('alpha_share'), m.get('alpha_values', [])[:4], 'flat', m.get('flat'), 'col', m.get('colours'),
                      '| seq', r['sequence'], r['seq_name'], s.get('kind'), 'fr', s.get('frames'), 'cyc', s.get('cycles'), 'loop', s.get('loop'), s.get('uses'), '| scale', r['scale'], 'amb', r['ambient'], 'con', r['contrast'])
    else: summary()
