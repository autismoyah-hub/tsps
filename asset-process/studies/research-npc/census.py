"""Numbers for NPCS.md from the census files written by `asset_npc.exe CACHE census census/`.

usage: census.py [--skeleton ID] [--find NAME] [--npc ID]
With no option prints the whole survey. --skeleton lists the NPCs (distinct model sets) and the sequences on one
frame base; --find lists definitions whose name contains NAME; --npc prints one definition with its models' stats.
Everything is read from the cache through the client's own decoders (npc.rs, animation.rs, model.rs).
"""
import json, sys, pathlib, collections
D = pathlib.Path(__file__).parent / 'census'
def load(n): return [json.loads(l) for l in (D / n).read_text().splitlines() if l.strip()]
npcs = load('npcs.jsonl'); seqs = {s['id']: s for s in load('sequences.jsonl')}; models = {m['id']: m for m in load('models.jsonl')}
kits = load('kits.jsonl'); items = load('items.jsonl'); skels = {s['id']: s for s in load('skeletons.jsonl')}
kit_models = {m for k in kits for m in k['models']}
item_models = {m for i in items for m in i['male'] + i['female'] if m >= 0}
item_of = {}
for i in items:
    for m in i['male'] + i['female']:
        if m >= 0: item_of.setdefault(m, i)
def skel(n, slot=0):
    s = seqs.get(n['anims'][slot]); return s['skeleton'] if s else -1
def kind(n, slot=0):
    s = seqs.get(n['anims'][slot]); return s['kind'] if s else 'none'
def faces(n): return sum(models[m]['faces'] for m in n['models'] if m in models)
def bounds(n):
    ms = [models[m] for m in n['models'] if m in models and models[m]['verts']]
    if not ms: return (0, 0, 0)
    lo = [min(m['min'][i] for m in ms) for i in range(3)]; hi = [max(m['max'][i] for m in ms) for i in range(3)]
    sx, sy = n['scale'][0] / 128, n['scale'][1] / 128
    return (round((hi[0] - lo[0]) * sx), round(-lo[1] * sy), round((hi[2] - lo[2]) * sx))
def labels(n): return sorted({l for m in n['models'] if m in models for l in models[m]['labels']})
def weights(n): return any(models[m]['weights'] for m in n['models'] if m in models)
def describe(n):
    w, h, d = bounds(n)
    return (f"{n['id']} {n['name']!r} size {n['size']} combat {n['combat']} models {len(n['models'])} faces {faces(n)} "
            f"box {w}x{h}x{d} scale {n['scale']} stand {n['anims'][0]} walk {n['anims'][3]} skeleton {skel(n)} ({kind(n)}) "
            f"labels {len(labels(n))} weights {weights(n)} zbuffer {n['zbuffer']} recolours {len(n['recolor'])}")
a = sys.argv
if '--find' in a:
    q = a[a.index('--find') + 1].lower()
    for n in npcs:
        if q in n['name'].lower(): print(describe(n))
    raise SystemExit
if '--npc' in a:
    i = int(a[a.index('--npc') + 1]); n = next(x for x in npcs if x['id'] == i)
    print(describe(n)); print(json.dumps(n))
    for m in n['models']:
        s = models.get(m)
        if s:
            src = 'body kit' if m in kit_models else f"item {item_of[m]['id']} {item_of[m]['name']!r}" if m in item_models else ''
            print(f"  model {m}: {s['verts']} v {s['faces']} f, labels {s['labels']}, prio {s['prio']}, depth offsets {s['depth_offsets']}, alpha {s['alpha']}, textures {s['textures']}, weights {s['weights']}, box {s['min']} {s['max']} {src}")
    raise SystemExit
if '--skeleton' in a:
    k = int(a[a.index('--skeleton') + 1])
    seen = set()
    print('skeleton', k, skels.get(k, {}).get('groups'), 'groups', skels.get(k, {}).get('bones'), 'bones')
    for n in npcs:
        if n['models'] and skel(n) == k and tuple(n['models']) not in seen:
            seen.add(tuple(n['models'])); print(' ', describe(n))
    ss = [s for s in seqs.values() if s['skeleton'] == k]
    print(len(ss), 'sequences on it:', ' '.join(f"{s['id']}({s['frames']})" for s in ss))
    raise SystemExit

P = print
with_models = [n for n in npcs if n['models']]
P(f'NPC definitions {len(npcs)}; with at least one model {len(with_models)}; named (not "null") {sum(n["name"] != "null" for n in with_models)}')
sets = {}
for n in with_models: sets.setdefault((tuple(n['models']), tuple(map(tuple, n['recolor'])), tuple(n['scale'])), n)
distinct = list(sets.values())
msets = {}
for n in with_models: msets.setdefault(tuple(n['models']), n)
P(f'distinct looks (model list + recolours + scale) {len(distinct)}; distinct model lists {len(msets)}')
for title, pop in (('all definitions with models', with_models), ('distinct model lists', list(msets.values()))):
    c = collections.Counter(min(len(n['models']), 9) for n in pop)
    P(f'-- number of models per NPC, {title}:')
    P('   ' + ', '.join(f'{k if k < 9 else "9+"}: {v} ({100 * v / len(pop):.1f}%)' for k, v in sorted(c.items())))
    one = sum(len(n['models']) == 1 for n in pop)
    P(f'   one model {one} ({100 * one / len(pop):.1f}%), several {len(pop) - one} ({100 * (len(pop) - one) / len(pop):.1f}%)')
pop = with_models
P('-- how it is painted: z_buffer cleared (opcode 147, painted by priority like a player):', sum(not n['zbuffer'] for n in pop), 'of', len(pop))
for n in [n for n in pop if not n['zbuffer']][:40]: P('     ', describe(n))
# frame bases
player_skel = seqs[808]['skeleton']
P(f'-- the player frame base is skeleton {player_skel} (sequence 808 stand, 819 walk: {seqs[819]["skeleton"]})')
st = collections.Counter(kind(n) for n in pop)
P('   stand sequence kind over definitions:', dict(st))
P('   walk sequence present:', sum(n['anims'][3] >= 0 for n in pop), ' stand present:', sum(n['anims'][0] >= 0 for n in pop), 'of', len(pop))
P('   stand and walk on the same frame base:', sum(skel(n) == skel(n, 3) for n in pop if n['anims'][0] >= 0 and n['anims'][3] >= 0), 'of', sum(1 for n in pop if n['anims'][0] >= 0 and n['anims'][3] >= 0))
hum = [n for n in pop if skel(n) == player_skel or (n['anims'][0] < 0 and skel(n, 3) == player_skel)]
P(f'   definitions standing on the player frame base: {len(hum)} ({100 * len(hum) / len(pop):.1f}%); distinct model lists {len({tuple(n["models"]) for n in hum})}')
uses_kit = [n for n in pop if any(m in kit_models for m in n['models'])]
uses_item = [n for n in pop if any(m in item_models for m in n['models'])]
either = [n for n in pop if any(m in kit_models or m in item_models for m in n['models'])]
P(f'   definitions containing a player body-kit model: {len(uses_kit)}; containing a worn-item model: {len(uses_item)}; either: {len(either)} ({100 * len(either) / len(pop):.1f}%)')
hk = [n for n in hum if any(m in kit_models or m in item_models for m in n['models'])]
P(f'   of the {len(hum)} on the player frame base: {len(hk)} contain a kit or worn-item model ({100 * len(hk) / max(1, len(hum)):.1f}%); '
  f'several models {sum(len(n["models"]) > 1 for n in hum)}; one model {sum(len(n["models"]) == 1 for n in hum)}')
allk = [n for n in hum if all(m in kit_models or m in item_models for m in n['models'])]
P(f'   ... made ONLY of kit and worn-item models: {len(allk)}')
P(f'   kit/item parts used by NPCs NOT on the player base: {len([n for n in either if n not in hum])}')
c = collections.Counter(len(n['models']) for n in hum); P('   models per player-base NPC:', dict(sorted(c.items())))
P(f'   player-base NPCs with z_buffer cleared: {sum(not n["zbuffer"] for n in hum)}')
# general properties
P('-- recolours: definitions with any', sum(bool(n['recolor']) for n in pop), f'({100 * sum(bool(n["recolor"]) for n in pop) / len(pop):.1f}%); retextured', sum(n['retexture'] > 0 for n in pop))
P('   median recolour pairs where present:', sorted(len(n['recolor']) for n in pop if n['recolor'])[sum(bool(n['recolor']) for n in pop) // 2])
sc = [n for n in pop if n['scale'] != [128, 128]]
P(f'-- scale: not 128/128 in {len(sc)} ({100 * len(sc) / len(pop):.1f}%); range xz {min(n["scale"][0] for n in pop)}..{max(n["scale"][0] for n in pop)}, y {min(n["scale"][1] for n in pop)}..{max(n["scale"][1] for n in pop)}')
q = sorted(n['scale'][1] for n in sc); P('   scaled ones, y scale p10/median/p90:', q[len(q) // 10], q[len(q) // 2], q[9 * len(q) // 10])
P('-- size in tiles:', dict(sorted(collections.Counter(n['size'] for n in pop).items())))
P('-- chat-head models: definitions with any', sum(bool(n['heads']) for n in pop), f'({100 * sum(bool(n["heads"]) for n in pop) / len(pop):.1f}%); head model count', dict(sorted(collections.Counter(len(n['heads']) for n in pop).items())))
P('-- lighting: ambient/contrast not 0 in', sum(n['ambient'] != 0 or n['contrast'] != 0 for n in pop), 'definitions; commonest:', collections.Counter((n['ambient'], n['contrast'] // 5) for n in pop).most_common(8))
P('-- models carrying bone weights: definitions', sum(weights(n) for n in pop), '; stand is a cached (bone) animation', st.get('cached', 0))
P('-- models with per-face priorities (more than one value):', sum(1 for n in msets.values() if len({p for m in n['models'] if m in models for p in models[m]['prio']}) > 1), 'of', len(msets), 'model lists;',
  'with depth offsets:', sum(1 for n in msets.values() if any(models[m]['depth_offsets'] for m in n['models'] if m in models)))
# skeleton sharing
by = collections.defaultdict(list)
for n in pop:
    k = skel(n) if n['anims'][0] >= 0 else skel(n, 3)
    by[k].append(n)
P(f'-- frame bases (by stand sequence, else walk): {len(by)} distinct (including -1 = no sequence: {len(by.get(-1, []))} definitions)')
rows = []
for k, ns in by.items():
    dm = {}
    for n in ns: dm.setdefault(tuple(n['models']), n)
    names = collections.Counter(n['name'] for n in ns if n['name'] != 'null')
    rows.append((len(dm), len(ns), k, names, dm))
rows.sort(key=lambda r: -r[0])
tot = sum(r[0] for r in rows if r[2] >= 0)
P('   distinct model lists per frame base: used by exactly 1:', sum(1 for r in rows if r[0] == 1 and r[2] >= 0), '; 2 to 4:', sum(1 for r in rows if 2 <= r[0] <= 4), '; 5 to 19:', sum(1 for r in rows if 5 <= r[0] <= 19), '; 20 and more:', sum(1 for r in rows if r[0] >= 20 and r[2] >= 0))
P('   share of distinct model lists that sit on a frame base serving 5 or more:', f'{100 * sum(r[0] for r in rows if r[0] >= 5 and r[2] >= 0) / tot:.1f}%', '; on a base of their own:', f'{100 * sum(r[0] for r in rows if r[0] == 1 and r[2] >= 0) / tot:.1f}%')
P('   the 40 most shared (distinct model lists / definitions / skeleton id / groups / sequences on it / commonest names):')
nseq = collections.Counter(s['skeleton'] for s in seqs.values())
for dmn, dn, k, names, dm in rows[:41]:
    if k < 0: continue
    P(f'   {dmn:5d} {dn:5d}  skel {k:5d}  groups {skels.get(k, {}).get("groups", "?"):>3} bones {skels.get(k, {}).get("bones", "?"):>3} seqs {nseq[k]:4d}  ' + ', '.join(f'{nm} x{c}' for nm, c in names.most_common(7)))
json.dump({str(k): [describe(n) for n in dm.values()] for dmn, dn, k, names, dm in rows if k >= 0}, open(D / 'by-skeleton.json', 'w'), indent=0)
