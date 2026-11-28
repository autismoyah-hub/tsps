"""Era study: measure real creatures in dated groups and ours, and say which measures separate the eras.

usage: era.py measure            measure every creature in GROUPS and OURS -> data/era.json   (slow: about 3 minutes)
       era.py table [--csv]      per-era medians, the separation of each measure, and our four scored
       era.py list               the groups with names, model ids and faces as the cache has them (the verification)
       era.py one ENTRY          one creature's numbers beside the era medians

DATING. Measured: the model ids of the definition (ids are handed out in order of creation; calibration below).
Knowledge (the author's, not the cache's): the creature's release year. A creature is in a group only if BOTH agree.
  calibration, name -> model id as measured -> release as known:
    TzTok-Jad 9319 (Oct 2005) | Dagannoth Rex 9941 (2005) | dragons 17407.., demons 17375.. (2006-07 remakes)
    Giant rat 18888, bears 18882, spiders 20309, cows 23889, goblins 24433, Kalphite Queen 24597, wolves and
    hellhounds 26252..26266 (2006-07 remakes) | General Graardor 27660 (Aug 2007: the end of the 2007 backup)
    Kraken 28231 (2014) | Cerberus 29270 (2015) | Skotizo 31653 (2016) | Vorkath 35023 (Jan 2018)
    Alchemical Hydra 36182.. (Jan 2019) | Sarachnis 37292 (Jul 2019) | The Nightmare 39196 (Feb 2020)
    Tempoross 41812 (2021) | Nex 43209 (Jan 2022) | Phantom Muspah 47149 (Jan 2023) | Vardorvis 49298 (Jul 2023)
    Scurrius 50144 (Jan 2024) | Sol Heredit 52580 (Mar 2024) | Araxxor 54289 (Aug 2024) | Doom 56469 (2025)
  EXCEPTIONS seen: later models were sometimes put into old gaps (Zulrah 14407-9, Corporeal Beast 11056, Snakeling
  10415, Lizardman shaman 4039, Yama 10468/10338, Veiled kraken 3854): a low id alone does not prove age, which is why
  the name must also be an old creature. The file format does NOT date a model: every model in this cache is stored
  in the newest format.
Groups: E1 = RuneScape 2 launch style, model ids under 12,500 (2004-2005) | E2 = late classic remakes, 13,000-27,700
(2006-Aug 2007) | R = 2019 onward, every model id 36,000 or more.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json, statistics as st
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
HERE = pathlib.Path(__file__).parent
B = _AP.fix('<ROOT>/builds/')
# (npc id, body plan)   body plan: q quadruped, s spider/many-legged, h humanoid monster, b boss, o other
E1 = [(8986, 'q'), (4652, 'q'), (417, 'q'), (410, 'q'), (4184, 'q'), (1273, 'q'), (484, 'q'), (5779, 'q'), (477, 'q'),
      (677, 'h'), (1153, 'h'), (2847, 'h'), (936, 'h'), (415, 'h'), (412, 'h'), (8, 'h'), (2241, 'h'), (880, 'h'), (1464, 'h'), (426, 'h'),
      (100, 's'), (970, 'o'), (419, 'o'), (7620, 'q'), (2193, 'h'),
      (3127, 'b'), (3125, 'b'), (2267, 'b')]
E2 = [(106, 'q'), (104, 'q'), (3133, 'q'), (2790, 'q'), (2510, 'q'), (2839, 'q'), (2909, 'q'), (2907, 'q'), (6473, 'q'), (2849, 'q'), (5726, 'q'),
      (2477, 's'), (134, 's'), (2479, 's'), (138, 's'),
      (2005, 'h'), (2025, 'h'), (240, 'h'), (655, 'h'), (2098, 'h'), (2090, 'h'), (6434, 'h'), (680, 'h'), (3134, 'h'),
      (260, 'b'), (239, 'b'), (137, 'q'), (963, 'b'), (2215, 'b'), (763, 'b')]
R = [(11068, 'q'), (12976, 'q'), (13021, 'q'), (10435, 'q'), (12289, 'q'), (13132, 'q'), (8614, 'q'), (8612, 'q'), (12490, 'q'), (15625, 'q'),
     (6609, 'q'), (12818, 'q'), (6613, 'q'), (14857, 'q'), (15759, 'q'), (14545, 'q'),
     (8713, 's'), (11175, 's'), (13680, 's'), (6610, 's'), (11697, 's'), (626, 's'),
     (9293, 'h'), (9454, 'h'), (9756, 'h'), (11273, 'h'), (11709, 'h'), (12879, 'h'), (13599, 'h'), (12812, 'h'), (10693, 'h'), (14125, 'h'),
     (15004, 'h'), (14702, 'h'), (15497, 'h'), (12025, 'h'), (13796, 'h'),
     (8615, 'b'), (8583, 'b'), (378, 'b'), (9021, 'b'), (11278, 'b'), (12077, 'b'), (12166, 'b'), (12223, 'b'), (7221, 'b'), (12821, 'b'),
     (13685, 'b'), (14009, 'b'), (14707, 'b'), (14147, 'b'), (9049, 'b'), (13668, 'b')]
GROUPS = dict(E1=E1, E2=E2, R=R)
OURS = {'hound': ('def:' + B + 'creature-star-hound-2/star-hound.json', 'q'),
        'hound-1': ('def:' + B + 'research-npc/hound/hound.json', 'q'),
        'spider': ('def:' + B + 'creature-orrery-spider/orrery-spider.json', 's'),
        'boss-1': ('def:' + B + 'boss-warden-general/boss-warden-general.json', 'b'),
        'boss-2': ('def:' + B + 'boss-warden-general-2/eclipse.json', 'b')}
# measures reported, in order: key, short description, digits
MEASURES = [
    ('faces', 'faces drawn', 0), ('vertices', 'vertices', 0), ('faces_per_1000sq', 'faces per 1,000 square units of surface', 1),
    ('face_med_share_e4', 'median face, ten-thousandths of the surface', 1), ('faces_for_half_surface', 'share of faces that carry half the surface', 3),
    ('face_p90_p10', 'face area, 90th / 10th centile', 1), ('edge_med_over_height', 'median face edge / body height', 3),
    ('colours', 'colours', 0), ('colours_for_90', 'colours covering 90% of surface', 0), ('hue_families', 'hue families (3% or more)', 0),
    ('tones_per_hue', 'tones per hue family (area-weighted)', 1), ('tones_main_hue', 'tones in the main hue', 0),
    ('main_hue_light_span', 'lightness span of the main hue', 0), ('main_hue_light_step', 'median step between its tones', 1),
    ('light_med', 'median painted lightness', 0), ('sat_med', 'median saturation', 0), ('accent_share', 'surface in minor hues (under 8% each)', 3),
    ('tone_follows_up', 'tone follows facing: correlation lightness~up', 2), ('tone_follows_light', 'correlation lightness~lit side', 2),
    ('tone_by_part', 'tone by part: lightness variance explained by label', 2), ('tone_follows_height', 'correlation lightness~height', 2),
    ('colour_edge_share', 'edge length that is a colour boundary', 3), ('insurface_colour_edge_share', 'of in-surface edges (fold<20), share that are colour boundaries', 3),
    ('colour_edges_in_surface', 'of colour boundaries, share lying in a surface', 3), ('tone_step_edges_same_hue', 'of colour boundaries, share that are tone steps of one hue', 3),
    ('tone_step_med', 'median lightness step across such an edge', 0),
    ('regions_per_100_faces', 'colour patches per 100 faces', 1), ('faces_per_region_med', 'median faces per colour patch', 1), ('largest_region_share', 'largest one-colour patch, share of surface', 3),
    ('detail_islands', 'small contrasting islands (eyes, teeth, claws, marks)', 0), ('detail_islands_per_100_faces', 'islands per 100 faces', 2),
    ('detail_area_share', 'surface in such islands', 3), ('detail_inlaid', 'islands inlaid in the skin (shared vertices)', 0), ('detail_own_shell', 'islands that are their own small shell', 0), ('detail_offset', 'islands carrying depth offsets', 0),
    ('flat_face_share', 'flat-shaded faces', 3), ('depth_offset_face_share', 'faces with a depth offset', 3), ('alpha_face_share', 'translucent faces', 3), ('hidden_vote_faces', 'undrawn faces kept for votes', 0),
    ('textured_faces', 'textured faces', 0), ('priority_levels', 'priority levels (no effect on an ordinary NPC)', 0),
    ('split_vertex_share', 'vertices on a shared position', 3), ('shells', 'separate shells', 0), ('largest_shell_face_share', 'largest shell, share of faces', 3),
    ('open_edge_share', 'open edges', 3),
    ('fold_med_deg', 'median fold between neighbouring faces, degrees', 1), ('fold_over30_share', 'edge length folded over 30 degrees', 3), ('fold_over60_share', 'edge length folded over 60 degrees', 3),
    ('fold_under5_share', 'edge length nearly flat (under 5 degrees)', 3), ('sliver_face_share', 'sliver faces (aspect over 5)', 3), ('aspect_med', 'median face aspect (1 = equilateral)', 2),
    ('valence_sd', 'spread of faces per vertex', 2), ('valence_6_share', 'vertices with exactly six faces', 3), ('mirror_vertex_share', 'vertices with a mirror twin', 3),
    ('trunk_face_share', 'faces on the trunk (thick body)', 3), ('trunk_area_share', 'surface on the trunk', 3), ('limb_face_share', 'faces on middling parts (head, neck, thighs)', 3),
    ('thin_face_share', 'faces on thin parts (shins, feet, hands, horns, tail)', 3), ('thin_area_share', 'surface on thin parts', 3), ('thin_to_trunk_density', 'face density, thin parts / trunk', 2),
    ('big_faces_on_trunk', 'of the largest tenth of faces, share on the trunk', 2), ('feet_zone_face_share', 'faces in the lowest 12% of height', 3),
    ('ragged', 'outline length / hull length, mean of three views', 3), ('ragged_side', 'the same, side view', 3), ('solidity', 'outline area / hull area', 3), ('ragged_game', 'outline / hull at game size', 3),
    ('labels', 'vertex labels', 0), ('vertices_per_label', 'vertices per label', 1), ('bridge_face_share', 'faces spanning two or more labels', 3), ('weights', 'moves by bone weights', 2),
    ('ambient', 'definition ambient', 0), ('contrast', 'definition contrast (stored / 5)', 0), ('size_y', 'height in units, after scale', 0),
]

def census():
    import find
    return {n['id']: n for n in find.npcs}, find.models

def measure_all():
    import cmesh
    defs, models = census()
    rows = {}
    def one(key, entry, group, plan, d=None):
        m = cmesh.load(entry)
        scale = tuple(d['scale']) if d else tuple(json.loads(pathlib.Path(entry[4:]).read_text()).get('scale', [128, 128]))
        r = cmesh.measure(m, scale)
        if d:
            ms = [x for x in d['models'] if x in models]
            r.update(name=d['name'], model_ids=d['models'], textured_faces=sum(models[x]['textures'] for x in ms), weights=float(any(models[x]['weights'] for x in ms)),
                     ambient=d['ambient'], contrast=d['contrast'] // 5, combat=d['combat'], size_tiles=d['size'])
        else:
            j = json.loads(pathlib.Path(entry[4:]).read_text())
            r.update(name=j.get('name', key), model_ids=[], textured_faces=0, weights=0.0, ambient=j.get('ambient', 0), contrast=j.get('contrast', 0), combat=0, size_tiles=j.get('size', 1))
        r.update(group=group, plan=plan, entry=entry)
        rows[key] = r
        print(f"{group:4s} {key:8s} {r['name'][:26]:26s} faces {r['faces']:5d} models {r['model_ids']}", flush=True)
    for g, lst in GROUPS.items():
        for i, plan in lst: one(str(i), f'npc:{i}', g, plan, defs[i])
    for k, (entry, plan) in OURS.items(): one(k, entry, 'OURS', plan)
    (HERE / 'data').mkdir(exist_ok=True)
    (HERE / 'data' / 'era.json').write_text(json.dumps(rows, indent=1))

def auc(a, b):
    """Probability that a random member of b exceeds a random member of a (0.5 = no separation)."""
    a, b = np.asarray(a, float), np.asarray(b, float)
    return float(((b[None, :] > a[:, None]).sum() + .5 * (b[None, :] == a[:, None]).sum()) / (len(a) * len(b)))

def q(xs, f): xs = sorted(xs); return xs[int(round(f * (len(xs) - 1)))]

def table(plan=None, out=print):
    rows = json.loads((HERE / 'data' / 'era.json').read_text())
    sel = lambda g: [r for r in rows.values() if r['group'] == g and (plan is None or r['plan'] in plan)]
    e1, e2, rr = sel('E1'), sel('E2'), sel('R')
    out(f"creatures: E1 {len(e1)}, E2 {len(e2)}, R {len(rr)}" + (f"  (body plans {plan})" if plan else ''))
    out('measure | E1 median (10th-90th) | E2 median (10th-90th) | R median (10th-90th) | sep E1:R | sep E2:R | ' + ' | '.join(OURS))
    res = {}
    for key, desc, dg in MEASURES:
        vals = [[r.get(key) for r in g if r.get(key) is not None] for g in (e1, e2, rr)]
        if not all(vals): continue
        f = lambda x: f'{x:.{dg}f}'
        cells = [f'{f(st.median(v))} ({f(q(v, .1))}-{f(q(v, .9))})' for v in vals]
        a1, a2 = auc(vals[0], vals[2]), auc(vals[1], vals[2])
        ours = []
        for k in OURS:
            x = rows[k].get(key)
            if x is None: ours.append('-'); continue
            # where ours sits: share of each era's creatures below it
            pe = np.mean([v < x for v in vals[0] + vals[1]]) ; pr = np.mean([v < x for v in vals[2]])
            ours.append(f'{f(x)}')
        res[key] = dict(desc=desc, med=[st.median(v) for v in vals], p10=[q(v, .1) for v in vals], p90=[q(v, .9) for v in vals], auc_e1=a1, auc_e2=a2)
        out(f'{desc} [{key}] | {cells[0]} | {cells[1]} | {cells[2]} | {a1:.2f} | {a2:.2f} | ' + ' | '.join(ours))
    return rows, res

def verdicts(plan_of=None):
    """For each of ours and each measure that separates the eras (separation <= 0.25 or >= 0.75 for E1:R AND
    the same side for E2:R at <= 0.35 / >= 0.65), say which side ours falls on: nearer the early median or the
    recent median, measured in units of the pooled spread."""
    rows = json.loads((HERE / 'data' / 'era.json').read_text())
    out = {}
    for k, (entry, plan) in OURS.items():
        plans = {'q': 'q', 's': 'sq', 'b': 'bh', 'h': 'hb'}[plan]
        res_all = _res(rows, None); res_like = _res(rows, plans)
        lines = []
        for key, desc, dg in MEASURES:
            if key not in res_all: continue
            ra = res_all[key]
            strong = ra['auc_e1'] >= .72 or ra['auc_e1'] <= .28
            if not strong: continue
            x = rows[k].get(key)
            rl = res_like.get(key, ra)
            side = ['E1', 'E2', 'R'][int(np.argmin([abs(x - v) for v in rl['med']]))]
            inside_r = rl['p10'][2] <= x <= rl['p90'][2]
            lines.append((key, desc, dg, x, rl['med'][0], rl['med'][1], rl['med'][2], rl['p10'][2], rl['p90'][2], side, inside_r, ra['auc_e1'], ra['auc_e2']))
        out[k] = lines
    return out

def _res(rows, plan):
    sel = lambda g: [r for r in rows.values() if r['group'] == g and (plan is None or r['plan'] in plan)]
    gs = [sel('E1'), sel('E2'), sel('R')]; res = {}
    for key, desc, dg in MEASURES:
        vals = [[r.get(key) for r in g if r.get(key) is not None] for g in gs]
        if not all(vals): continue
        res[key] = dict(med=[st.median(v) for v in vals], p10=[q(v, .1) for v in vals], p90=[q(v, .9) for v in vals], auc_e1=auc(vals[0], vals[2]), auc_e2=auc(vals[1], vals[2]))
    return res

if __name__ == '__main__':
    a = sys.argv
    if a[1] == 'measure': measure_all()
    elif a[1] == 'list':
        defs, models = census()
        for g, lst in GROUPS.items():
            for i, plan in lst:
                d = defs[i]; ms = [x for x in d['models'] if x in models]
                print(f"{g} {plan} npc {i:6d} {d['name'][:28]:28s} models {d['models']} faces {sum(models[x]['faces'] for x in ms)}")
    elif a[1] == 'table':
        plan = a[a.index('--plan') + 1] if '--plan' in a else None
        table(plan)
    elif a[1] == 'verdicts':
        for k, lines in verdicts().items():
            print(f'== {k}')
            for key, desc, dg, x, m1, m2, mr, lo, hi, side, inside, a1, a2 in lines:
                print(f'  {side:6s} {"in" if inside else "OUT"} {desc} [{key}]: ours {x:.{dg}f}; E1 {m1:.{dg}f}, E2 {m2:.{dg}f}, R {mr:.{dg}f} (R 10th-90th {lo:.{dg}f}-{hi:.{dg}f}); sep {a1:.2f}/{a2:.2f}')
