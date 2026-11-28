"""pkg_measure.py: measure eight new end-game bosses as PACKAGES (revision 239 cache; read only).

usage: pkg_measure.py            -> data/packages.json, data/packages.txt

A package = every NPC, sequence, spot animation and object whose GAMEVAL name (archive 24: Jagex's own internal names,
groups 1 NPC, 6 LOC, 7 SEQ, 8 SPOTANIM) matches the boss's name stem, plus every sequence on the boss's frame base, plus
the spot animations the owner's server copy names in the boss's plugin folder (<TSPS_CHECKOUT>/server/plugins/bosses/*,
read only; a cross-check for REUSED older effects, which carry another boss's or a generic name).
Sources reused, not re-measured: the NPC census (studies/research-npc/census), the spot census
(studies/research-effects/data/spots.jsonl), the animated-object census (studies/research-effects2/data/objanim.jsonl),
BOSSES.md's per-form measures (studies/research-bosses/data/bosses.json). New here: the grouping into packages, the
colour relation of effects to the boss, object model numbers for static objects, roles from names.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, re, json, pathlib, collections, statistics as st
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
B = pathlib.Path(_AP.fix(r'<ROOT>\builds'))
for p in (B / 'research-effects', B / 'research-effects2', B / 'research-creatures', B / 'research-npc', B / 'research-animation'):
    sys.path.insert(0, str(p))
import fxlib, objlib
import osrscache as oc
import cmesh
cmesh.DATA = HERE / 'data'
(HERE / 'data').mkdir(exist_ok=True)

C = B / 'research-npc' / 'census'
def rd(n): return {j['id']: j for j in (json.loads(l) for l in (C / f'{n}.jsonl').read_text(encoding='utf-8').splitlines() if l.strip())}
NPCS, MODELS, SEQS, SKELS = rd('npcs'), rd('models'), rd('sequences'), rd('skeletons')
SPOTS = {j['id']: j for j in map(json.loads, (B / 'research-effects/data/spots.jsonl').read_text(encoding='utf-8').splitlines())}
OBJAN = {j['id']: j for j in map(json.loads, (B / 'research-effects2/data/objanim.jsonl').read_text(encoding='utf-8').splitlines())}
BJ = json.loads((B / 'research-bosses/data/bosses.json').read_text())
GN = {k: fxlib.names(v) for k, v in dict(npc=1, loc=6, seq=7, spot=8).items()}
TSPS = pathlib.Path(_AP.fix(r'<TSPS_CHECKOUT>\server\plugins\bosses'))

# key, label, release (my knowledge; month), main NPC ids (the fought forms), name stem regex, exclusion regex, plugin folder
PK = [
 ('sol', 'Sol Heredit', '2024-03', [12821], r'colosseum_sol|colossi_finalboss|colosseum_finalboss|colosseum_boss|solheredit|sunfire', r'^$', None),
 ('araxxor', 'Araxxor', '2024-06', [13668], r'araxxor', r'easter_egg', 'araxxor'),
 ('amox', 'Amoxliatl', '2024-09', [13685], r'amoxliatl', r'quest|cutscene', 'amoxliatl'),
 ('huey', 'The Hueycoatl', '2024-09', [14009, 14013, 14014, 14017], r'huey', r'lobby|victory_worker|nelta|seer|quest|cutscene', 'hueycoatl'),
 ('titans', 'Royal Titans (Branda, Eldric)', '2024-11', [12596, 14147], r'^rt_|rtbranda|rteldric|royal_titans|branda|eldric|^rs_loc', r'^$', 'royaltitans'),
 ('yama', 'Yama', '2025-05', [14176], r'(^|_)yama', r'^voice_of|^disciple|^akd_|quest|cutscene', 'yama'),
 ('doom', 'Doom of Mokhaiotl', '2025-07', [14707, 14708, 14709], r'^dom_|vfx_dom_|doom_boss|vfx_doom|_dom_|demonic_grub', r'playtester|chest_loot|scoreboard|doom_scorpion|doomsayer|doomion', 'doom'),
 ('maggot', 'Maggot King', '2026-06', [15742], r'maggot_king|ur_maggot|floorkit_maggot|corpse_maggot|maggot01', r'^$', 'maggotking'),
]

def hsl(c): return (c >> 10) & 63, (c >> 7) & 7, c & 127

def npc_role(gv, nid, mains, combat):
    g = gv or ''
    if nid in mains: return 'boss'
    if 'pet' in g: return 'pet'
    if re.search(r'projectile|timer|hitbox|node|handler|invis|spotanim|placeholder|headbar|safespot|buff|path|clickbox', g): return 'helper'
    if re.search(r'dead|death|corpse|defeated|gib|inactive|seated|throne|cutscene|quest|_base', g): return 'state'
    if combat and combat > 0: return 'add'
    return 'other'

def seq_role(g):
    g = g or ''
    if g.startswith('vfx') or 'spotanim' in g or g.startswith('loc_') or '_loc' in g: return 'effect/object'
    for k, rx in (('idle', r'idle|ready|stand'), ('walk', r'walk|move|run|crawl|fly|slither'), ('death', r'death|die|despawn|defeat|knockout'),
                  ('spawn', r'spawn|emerge|rise|intro|awake|enter|appear|land|jump'), ('hit/block', r'hit|block|flinch|defend|stagger|stun'),
                  ('attack', r'attack|slam|swipe|bite|spit|throw|melee|range|mage|magic|cast|screech|charge|beam|fire|smash|swing|summon|aoe|stomp|breath|grapple|triple|special'),
                  ('phase', r'phase|transition|enrage|shield|burrow|teleport|inactive|tail')):
        if re.search(rx, g): return k
    return 'other'

def spot_role(g):
    g = g or ''
    for k, rx in (('telegraph', r'telegraph|warn|indicat|shadow|target|marker|safezone'), ('impact', r'impact|hit|explo|splat|burst|land'),
                  ('projectile', r'proj|travel|missile|bolt|orb_|_orb|flight'), ('ground', r'ground|pool|floor|tile|aoe|area|vent|puddle|crack|fire|ice'),
                  ('cast/aura', r'cast|charge|aura|shield|glow|summon|spawn|screech|buff|immun|beam|channel')):
        if re.search(rx, g): return k
    return 'other'

def loc_role(g, d):
    g = g or ''
    if re.search(r'entrance|exit|door|barrier|gate|ladder|stairs|hole|shortcut|enter', g): return 'entry/barrier'
    if re.search(r'chest|loot|scoreboard|bank|altar|lockbox|reward|bones_\d|playtester', g): return 'utility/memorial'
    if re.search(r'floorkit|floor0|corpse|detritus|pile|soil|decor|statue|pillar|throne|kit|wall|rock0|rubble|pile', g) and not re.search(r'vfx|attack|hazard', g): return 'dressing'
    if re.search(r'vfx|pool|acid|glyph|tile|fire|ice|spawn|death|attack|hazard|bile|blood|flare|rock|crystal|cloud|shock|beam|web|egg', g): return 'mechanic'
    return 'other'

def colour_table(nid):
    """Area-weighted families of the boss's rest mesh (cmesh's own family rule)."""
    m = cmesh.load(f'npc:{nid}')
    area, unit, cen = cmesh.face_geometry(m)
    keep = m['A'] < 254
    by = collections.Counter()
    for c, a in zip(m['C'][keep], area[keep]): by[int(c)] += float(a)
    tot = sum(by.values()) or 1
    cmesh.HMAP.clear(); cmesh.HMAP.update(cmesh.hue_clusters([c for c, a in by.items() if a >= .004 * tot]))
    fam = collections.defaultdict(float); famc = collections.defaultdict(collections.Counter)
    for c, a in by.items(): f = cmesh.family(c); fam[f] += a; famc[f][c] += a
    out = []
    for f, a in sorted(fam.items(), key=lambda kv: -kv[1]):
        cs = famc[f]; w = sum(cs.values())
        hl = [hsl(c) for c in cs]; ws = list(cs.values())
        out.append(dict(family=str(f), share=round(a / tot, 4), tones=len([c for c, x in cs.items() if x >= .004 * tot]),
                        hue=round(sum(h * x for (h, s, l), x in zip(hl, ws)) / w, 1), sat=round(sum(s * x for (h, s, l), x in zip(hl, ws)) / w, 1),
                        light=round(sum(l * x for (h, s, l), x in zip(hl, ws)) / w, 1)))
    alpha = float(area[(m['A'] > 0) & (m['A'] < 254)].sum() / max(area.sum(), 1e-9))
    # fixed hue sectors (8 of 45 degrees; saturation 0-1 = neutral), the second colour-plan measure; accent placement
    V, F = m['V'], m['F']
    y = -cen[:, 1]; lo, hi = y.min(), y.max(); hz = (y - lo) / max(hi - lo, 1e-9)
    sect = collections.defaultdict(float); sat_area = collections.defaultdict(float); zone = collections.defaultdict(lambda: [0.0, 0.0, 0.0])
    for i in range(len(F)):
        if not keep[i]: continue
        h, s, l = hsl(int(m['C'][i]))
        k = 'n-dark' if (s <= 1 and l < 40) else 'n-light' if (s <= 1 and l >= 80) else 'n-mid' if s <= 1 else f'h{h // 8}'
        sect[k] += area[i]; sat_area[k] += area[i] * s
        zone[k][0 if hz[i] < 1 / 3 else 1 if hz[i] < 2 / 3 else 2] += area[i]
    tt = sum(sect.values()) or 1
    sectors = [dict(sector=k, share=round(v / tt, 4), sat=round(sat_area[k] / v, 2), low_mid_top=[round(z / v, 2) for z in zone[k]])
               for k, v in sorted(sect.items(), key=lambda kv: -kv[1])]
    # accent = the most saturated sector holding 1 to 15% of the area
    cand = [x for x in sectors if .01 <= x['share'] <= .15 and not x['sector'].startswith('n')]
    accent = max(cand, key=lambda x: x['sat']) if cand else None
    return out, dict(HMAP=dict(cmesh.HMAP), sectors=sectors, accent=accent), round(alpha, 4)

def fam_of(h, s, l, hmap):
    if l <= 6: return 'k'
    if l >= 122: return 'w'
    if s == 0: return 'g'
    for d in (0, 1, -1, 2, -2):
        if (h + d) % 64 in hmap: return str(hmap[(h + d) % 64])
    return None

def spot_relation(sp, fams, hmap):
    """Share of the effect's top colours in a family the boss has (>=3% = body or accent) and which."""
    top = sp.get('m', {}).get('top', [])
    if not top: return None
    big = {f['family'] for f in fams if f['share'] >= .03}; small = {f['family'] for f in fams if f['share'] < .03}
    main = fams[0]['family'] if fams else None
    tot = sum(t[3] for t in top) or 1
    r = collections.Counter()
    for h, s, l, sh in top:
        f = fam_of(h, s, l, hmap)
        k = 'main' if f == main else 'body' if f in big else 'trace' if f in small else 'neutral' if f in ('k', 'w', 'g') else 'new'
        r[k] += sh / tot
    return {k: round(v, 3) for k, v in r.items()}

def sector_of(h, s, l):
    return 'n-dark' if (s <= 1 and l < 40) else 'n-light' if (s <= 1 and l >= 80) else 'n-mid' if s <= 1 else f'h{h // 8}'

def spot_sector_relation(sp, extra):
    top = sp.get('m', {}).get('top', [])
    if not top or not extra: return None
    secs = {x['sector']: x['share'] for x in extra['sectors']}
    main = extra['sectors'][0]['sector']; acc = extra['accent']['sector'] if extra['accent'] else None
    tot = sum(t[3] for t in top) or 1; r = collections.Counter()
    for h, s, l, sh in top:
        k0 = sector_of(h, s, l)
        k = 'main' if k0 == main else 'accent' if k0 == acc else 'body' if secs.get(k0, 0) >= .03 else 'trace' if k0 in secs else 'new'
        if k0.startswith('n') and k == 'new': k = 'neutral'
        r[k] += sh / tot
    return {k: round(v, 3) for k, v in r.items()}

def plugin_spots(folder):
    """Spot ids named in the server copy's plugin for this boss (lines that mention graphic/projectile/gfx/spot)."""
    if not folder or not (TSPS / folder).exists(): return {}
    ids = collections.Counter()
    kw = r'(?:graphic|gfx|projectile|spotanim|impact|splash|launch|telegraph|proj)\w*'
    after = re.compile(kw + r'["\']?\s*(?:[:=(]|\bid\b)?\s*[\[{(]?\s*((?:\d{2,5}\s*(?:,|/|or|and)?\s*){1,6})', re.I)
    for f in (TSPS / folder).rglob('*'):
        if f.suffix not in ('.js', '.json', '.ts'): continue
        for line in f.read_text(encoding='utf-8', errors='replace').splitlines():
            for grp in after.findall(line):
                for n in re.findall(r'\d{2,5}', grp):
                    n = int(n)
                    if n in SPOTS and n >= 100: ids[n] += 1
    return ids

GENERIC = re.compile(r'^vfx_(standard|area_slam|beam_charge|beam_impact|rock_projectile|generic|shared)|^vfx_\w*_generic', re.I)
def reuse_class(name, own):
    if own: return 'own'
    if GENERIC.search(name or ''): return 'shared library (new era)'
    return 'reused from other content'

def model_numbers(mids, recolor=None):
    f = a = t = v = 0; cols = set()
    for mid in mids:
        try: m = oc.model(mid)
        except Exception: continue
        f += len(m['faces']); v += len(m['vertices'])
        a += sum(1 for x in m['alpha'] if x); t += sum(1 for x in m['face_textures'] if x != -1)
        cols |= set(m['colors'])
    return dict(faces=f, verts=v, alpha_faces=a, textured=t, colours=len(cols))

def obj_numbers(oid):
    try: d = objlib.obj(oid)
    except Exception as e: return dict(error=str(e)[:50])
    a = OBJAN.get(oid)
    mn = model_numbers(d['models'])
    return dict(name=d['name'], size=d['size'], models=len(d['models']), **mn,
                anim=d['anim'], walk_through=d['interact'] == 0, animated=bool(a))

def main():
    out = {}
    for key, label, rel, mains, rx, ex, folder in PK:
        R = re.compile(rx, re.I); X = re.compile(ex + r'|league|deadman|^poh_|transmog|breach', re.I)
        sel = {k: sorted(i for i, n in GN[k].items() if R.search(n) and not X.search(n)) for k in GN}
        # NPCs
        npcs = []
        for i in sorted(set(sel['npc']) | set(mains)):
            n = NPCS.get(i)
            if not n: continue
            st_ = n['anims'][0]; sk = SEQS.get(st_, {}).get('skeleton')
            npcs.append(dict(id=i, gv=GN['npc'].get(i), name=n['name'], size=n['size'], combat=n['combat'], models=n['models'], stand=st_,
                             walk=n['anims'][3] if len(n['anims']) > 3 else -1, skeleton=sk, scale=n['scale'], ambient=n['ambient'], contrast=n['contrast'],
                             role=npc_role(GN['npc'].get(i), i, mains, n['combat'])))
        bosses = [n for n in npcs if n['role'] == 'boss']
        bases = sorted({n['skeleton'] for n in bosses if n['skeleton'] is not None})
        # models of the boss forms
        mids = sorted({m for n in bosses for m in n['models']})
        models = [dict(id=m, faces=MODELS[m]['faces'], verts=MODELS[m]['verts'], alpha_faces=MODELS[m]['alpha'], textured=MODELS[m]['textures'],
                       weights=MODELS[m]['weights'], colours=MODELS[m]['ncolours'], height=MODELS[m]['max'][1] - MODELS[m]['min'][1] if MODELS[m].get('min') else None)
                  for m in mids if m in MODELS]
        model_lists = sorted({tuple(n['models']) for n in bosses})
        # skeletons: who else stands on them
        skel = []
        for b in bases:
            on = [n for n in NPCS.values() if SEQS.get(n['anims'][0], {}).get('skeleton') == b]
            others = [dict(id=n['id'], gv=GN['npc'].get(n['id']), name=n['name']) for n in on if not R.search(GN['npc'].get(n['id']) or '')]
            older = [o for o in others if o['id'] < min(mains) - 300]
            seqs_on = [s for s in SEQS.values() if s['skeleton'] == b]
            skel.append(dict(base=b, groups=SKELS[b]['groups'], bones=SKELS[b]['bones'], npcs_on=len(on), npcs_outside_name=others[:12],
                             n_outside=len(others), older_creatures=older[:8], seqs_on=len(seqs_on)))
        # sequences: named + on the base(s)
        sids = sorted(set(sel['seq']) | {s['id'] for s in SEQS.values() if s['skeleton'] in bases})
        seqs = []
        for s in sids:
            q = SEQS.get(s)
            if not q: continue
            cyc = q['frames'] if q['kind'] == 'cached' else q['length']
            seqs.append(dict(id=s, gv=GN['seq'].get(s), named=bool(R.search(GN['seq'].get(s) or '')), kind=q['kind'], skeleton=q['skeleton'], cycles=cyc, on_base=q['skeleton'] in bases,
                             role=seq_role(GN['seq'].get(s))))
        # colours of the boss forms
        cols = {}
        for n in bosses:
            try: cols[n['id']] = colour_table(n['id'])
            except SystemExit as e: cols[n['id']] = (None, None, None); print('colour', n['id'], str(e)[:100])
        f0, hm0, _ = cols[mains[0]]
        hmap = {int(k): v for k, v in hm0['HMAP'].items()} if hm0 else {}
        # spots: named + server-used
        pu = plugin_spots(folder)
        spot_ids = sorted((set(sel['spot']) | set(pu)) - {i for i in sel['spot'] if False})
        spots = []
        for s in spot_ids:
            sp = SPOTS.get(s)
            if not sp: continue
            m = sp.get('m', {}); q = sp.get('s', {})
            spots.append(dict(id=s, gv=sp['name'], named_for_boss=bool(R.search(sp['name'] or '')) and not X.search(sp['name'] or ''), server_uses=s in pu,
                              role=spot_role(sp['name']), faces=m.get('faces'), colours90=m.get('colours90'), colours=m.get('colours'),
                              alpha_share=m.get('alpha_share'), textured=m.get('textured'), weights=m.get('weights'), size=max(m.get('box', [0])) if m.get('box') else 0,
                              scale=sp.get('scale'), kind=q.get('kind'), cycles=q.get('cycles'), grows=(q.get('size_max', 0) / max(1, q.get('size_min', 1))) if q.get('size_min') else None,
                              relation=spot_relation(sp, f0 or [], hmap), sector_relation=max((spot_sector_relation(sp, cols[b['id']][1]) or {} for b in bosses if cols[b['id']][1] and cols[b['id']][1].get('sectors')), key=lambda r: sum(v for k, v in r.items() if k in ('main', 'accent', 'body')), default=None), top=m.get('top', [])[:3]))
            spots[-1]['reuse'] = reuse_class(sp['name'], spots[-1]['named_for_boss']); spots[-1]['model'] = sp.get('model'); spots[-1]['sequence'] = sp.get('sequence')
        # objects
        locs = []
        for o in sel['loc']:
            d = obj_numbers(o)
            locs.append(dict(id=o, gv=GN['loc'][o], role=loc_role(GN['loc'][o], d), **d))
        out[key] = dict(key=key, label=label, release=rel, rx=rx, ex=ex, mains=mains, npcs=npcs, model_lists=[list(x) for x in model_lists], models=models,
                        bases=bases, skeleton=skel, seqs=seqs, colours={str(k): v[0] for k, v in cols.items()}, sectors={str(k): (v[1] or {}).get('sectors') for k, v in cols.items()}, accent={str(k): (v[1] or {}).get('accent') for k, v in cols.items()}, alpha_area={str(k): v[2] for k, v in cols.items()},
                        boss_measure={str(n['id']): {k: BJ[str(n['id'])].get(k) for k in ('faces_files', 'verts_files', 'weighted', 'skel_groups', 'skel_bones', 'seqs', 'alpha_faces', 'textured_faces', 'forms_ids', 'forms_model_lists')}
                                      | {k: BJ[str(n['id'])]['measure'].get(k) for k in ('faces', 'colours', 'colours_for_90', 'hue_families', 'main_hue_share', 'accent_share', 'tones_per_hue', 'shells', 'largest_shell_face_share', 'detail_islands', 'alpha_face_share', 'size_x', 'size_y', 'size_z')}
                                      for n in bosses if str(n['id']) in BJ},
                        spots=spots, plugin_spot_counts=dict(pu), locs=locs)
        print(key, 'npcs', len(npcs), 'boss forms', len(bosses), 'models', len(models), 'bases', bases, 'seqs', len(seqs), 'spots', len(spots), 'locs', len(locs))
    (HERE / 'data' / 'packages.json').write_text(json.dumps(out, indent=1), encoding='utf-8')

if __name__ == '__main__':
    main()
