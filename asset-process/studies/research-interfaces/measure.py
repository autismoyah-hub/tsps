"""measure.py: conventions of the game's interfaces, measured over every group of the revision-239 cache.

Inputs: the cache definitions (static) and state/<group>.jsonl (what the client draws after each group's own onLoad
scripts, from the tooling client). Output: harness/measure.json, used to write INTERFACES.md."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, collections
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifcache as C

GAMEVAL = Path(_AP.fix(r'<TSPS_CHECKOUT>/client/runelite/api/gameval'))
names_if = {int(i): n for n, i in re.findall(r'export const (\w+) = (\d+);', (GAMEVAL / 'InterfaceID.ts').read_text())}
names_sp = {int(i): n for n, i in re.findall(r'export const (\w+) = (\d+);', (GAMEVAL / 'SpriteID.ts').read_text())}
Cnt = collections.Counter


def top(counter, n=25):
    return [[k if not isinstance(k, tuple) else list(k), v] for k, v in counter.most_common(n)]


def main():
    cache = C.Cache()
    fonts = C.font_names(cache)
    groups = sorted(cache.reference(3))
    defs = {g: C.load_interface(cache, g) for g in groups}
    states = {}
    for g in groups:
        p = HERE / 'state' / f'{g}.jsonl'
        if p.exists():
            states[g] = [json.loads(l) for l in p.read_text().splitlines() if l.strip()]
    M = {}
    # ---- overall
    kinds = Cnt(); if1_groups = set(); listen = 0; onload = Cnt(); onload_groups = collections.defaultdict(set)
    for g, ws in defs.items():
        for w in ws:
            kinds[(('if3' if w['if3'] else 'if1'), C.KIND_NAMES.get(w['kind']))] += 1
            if not w['if3']:
                if1_groups.add(g)
            if w['listeners'] or w['triggers']:
                listen += 1
            if 'onLoad' in w['listeners']:
                s = w['listeners']['onLoad'][0]; onload[s] += 1; onload_groups[s].add(g)
    ncomp = sum(len(ws) for ws in defs.values())
    dyn = sum(1 for st in states.values() for r in st if r['child_index'] >= 0)
    drawn_dyn = sum(1 for st in states.values() for r in st if r['child_index'] >= 0 and not r['hidden'])
    M['overall'] = dict(groups=len(groups), components=ncomp, if1_components=sum(v for k, v in kinds.items() if k[0] == 'if1'),
                        if1_groups=len(if1_groups), kinds=top(kinds, 30), components_with_listeners=listen,
                        groups_with_state=len(states), script_created_components_after_onload=dyn,
                        script_created_visible=drawn_dyn,
                        onload_scripts=[[s, n, len(onload_groups[s]), sorted(onload_groups[s])[:15]] for s, n in onload.most_common(30)])
    # ---- per onLoad script: what it creates (sprite sets, texts) relative to the component it runs on
    created = collections.defaultdict(lambda: dict(groups=set(), sprites=Cnt(), texts=Cnt(), rects=Cnt(), geo=Cnt()))
    for g, st in states.items():
        byid = {r['id']: r for r in st}
        wdef = {w['id']: w for w in defs[g]}
        for r in st:
            if r['child_index'] < 0 or r['hidden']:
                continue
            host = r['parent']
            while host in byid and byid[host]['child_index'] >= 0:
                host = byid[host]['parent']
            w = wdef.get(host)
            if not w or 'onLoad' not in w['listeners']:
                continue
            s = w['listeners']['onLoad'][0]
            h = byid[host]
            rel = (r['abs'][0] - h['abs'][0], r['abs'][1] - h['abs'][1])
            e = created[s]; e['groups'].add(g)
            if r['kind'] == 5:
                e['sprites'][r['sprites'][0]] += 1
                e['geo'][('sprite', r['sprites'][0], rel[0] if rel[0] < 40 else 'W' + str(rel[0] - h['size'][0]),
                          rel[1] if rel[1] < 40 else 'H' + str(rel[1] - h['size'][1]),
                          r['size'][0] if r['size'][0] < 60 else 'W' + str(r['size'][0] - h['size'][0]),
                          r['size'][1] if r['size'][1] < 60 else 'H' + str(r['size'][1] - h['size'][1]), r['sprite_tiling'])] += 1
            elif r['kind'] == 4:
                e['texts'][(fonts.get(r['font'], r['font']), '%06x' % r['colors'][0], r['text_shadow'], tuple(r['text_alignment']))] += 1
                e['geo'][('text', rel[0], rel[1], 'W' + str(r['size'][0] - h['size'][0]), r['size'][1])] += 1
            elif r['kind'] == 3:
                e['rects'][('%06x' % r['colors'][0], r['filled'], r['transparency'])] += 1
    M['created_by_onload'] = {str(s): dict(groups=len(e['groups']), sample=sorted(e['groups'])[:12],
                                            sprites=[[k, v, names_sp.get(k), list(_dims(cache, k))] for k, v in e['sprites'].most_common(16)],
                                            texts=top(e['texts'], 8), rects=top(e['rects'], 8), geometry=top(e['geo'], 24))
                              for s, e in sorted(created.items(), key=lambda kv: -len(kv[1]['groups']))[:25]}
    # ---- steelborder panels (onLoad 227): sizes, content margins
    sizes = Cnt(); margins = Cnt(); titles = []
    for g in sorted(onload_groups[227]):
        st = states.get(g)
        if not st:
            continue
        byid = {r['id']: r for r in st}
        for w in defs[g]:
            if w['listeners'].get('onLoad', [None])[0] == 227:
                h = byid.get(w['id'])
                if not h:
                    continue
                sizes[tuple(h['size'])] += 1
                titles.append([g, names_if.get(g), w['listeners']['onLoad'][2] if len(w['listeners']['onLoad']) > 2 else None, h['size']])
                par = h['parent']
                sib = [r for r in st if r['parent'] == par and r['id'] != h['id'] and r['child_index'] < 0 and not r['hidden']
                       and r['size'][0] > 0 and r['size'][1] > 0 and r['size'] != h['size']]
                if sib:
                    x0 = min(r['abs'][0] for r in sib) - h['abs'][0]; y0 = min(r['abs'][1] for r in sib) - h['abs'][1]
                    x1 = h['abs'][0] + h['size'][0] - max(r['abs'][0] + r['size'][0] for r in sib)
                    y1 = h['abs'][1] + h['size'][1] - max(r['abs'][1] + r['size'][1] for r in sib)
                    margins[(x0, y0, x1, y1)] += 1
    M['steelborder'] = dict(panels=sum(sizes.values()), sizes=top(sizes, 30), content_margins_left_top_right_bottom=top(margins, 20),
                            widths=_dist([k[0] for k, v in sizes.items() for _ in range(v)]),
                            heights=_dist([k[1] for k, v in sizes.items() for _ in range(v)]), titles=titles)
    # ---- text census (everything drawn after onLoad, static and script-made)
    tf = Cnt(); tc = Cnt(); tsh = Cnt(); role = collections.defaultdict(Cnt); lh = Cnt(); al = Cnt(); heights = collections.defaultdict(Cnt)
    for g, st in states.items():
        for r in st:
            if r['kind'] != 4 or r['hidden'] or not r['text'].strip():
                continue
            f = fonts.get(r['font'], r['font']); col = '%06x' % r['colors'][0]
            tf[f] += 1; tc[col] += 1; tsh[(col, r['text_shadow'])] += 1; lh[(f, r['line_height'])] += 1
            al[tuple(r['text_alignment'])] += 1; heights[f][r['size'][1]] += 1
            t = re.sub(r'<[^>]+>', '', r['text'])
            kind = ('number' if re.fullmatch(r'[\d,.:/%+\-xkKmM ()]+', t.strip()) else
                    'title' if (r['child_index'] >= 0 and f == 'b12_full' and r['size'][1] == 24) else
                    'body' if len(t) > 40 else 'label')
            role[kind][(f, col, r['text_shadow'])] += 1
    M['text'] = dict(fonts=top(tf, 20), colours=top(tc, 30), colour_shadow=top(tsh, 30), line_heights=top(lh, 20),
                     alignment=top(al, 12), by_role={k: top(v, 12) for k, v in role.items()},
                     box_heights={f: top(v, 8) for f, v in heights.items()},
                     font_metrics={fonts.get(fid, fid): _fontm(cache, fid) for fid in fonts})
    # ---- rectangles
    rf = Cnt(); ro = Cnt(); rt = Cnt()
    for st in states.values():
        for r in st:
            if r['kind'] == 3 and not r['hidden']:
                (rf if r['filled'] else ro)['%06x' % r['colors'][0]] += 1
                rt[r['transparency']] += 1
    M['rectangles'] = dict(filled=top(rf, 20), outline=top(ro, 20), transparency=top(rt, 15))
    # ---- sprites most used (drawn after onLoad)
    sp = Cnt(); spg = collections.defaultdict(set)
    for g, st in states.items():
        for r in st:
            if r['kind'] == 5 and not r['hidden'] and r['sprites'][0] >= 0:
                sp[r['sprites'][0]] += 1; spg[r['sprites'][0]].add(g)
    M['sprites'] = [[s, n, len(spg[s]), names_sp.get(s), list(_dims(cache, s))] for s, n in sp.most_common(60)]
    # ---- buttons: static components a player can operate
    bt = Cnt(); bsz = Cnt(); bsp = Cnt()
    for ws in defs.values():
        for w in ws:
            if w['if3'] and (w['actions'] and any(w['actions']) or 'onOp' in w['listeners']):
                bt[C.KIND_NAMES.get(w['kind'])] += 1
                if w['size_mode'] == [0, 0]:
                    bsz[tuple(w['raw_size'])] += 1
                if w['kind'] == 5:
                    bsp[w['sprites'][0]] += 1
    M['buttons'] = dict(kinds=top(bt), fixed_sizes=top(bsz, 25),
                        sprites=[[s, n, names_sp.get(s), list(_dims(cache, s))] for s, n in bsp.most_common(20)])
    # ---- boss and status overlays
    ov = []
    for g, ws in defs.items():
        nm = names_if.get(g, '')
        if not re.search(r'HUD|OVERLAY|HPBAR|BOSS|TIMER|TOTEM', nm):
            continue
        bars = [w for w in ws if w['kind'] == 3 and w['filled'] and (w['raw_size'][0] >= 40 or w['size_mode'][0] in (1, 2)) and w['raw_size'][1] <= 30]
        texts = [(fonts.get(w['font'], w['font']), '%06x' % w['colors'][0], w['text_shadow']) for w in ws if w['kind'] == 4]
        roots = [w for w in ws if w['parent'] is None]
        ov.append(dict(group=g, name=nm, components=len(ws), if1=sum(not w['if3'] for w in ws),
                       bars=[dict(file=w['file'], size=w['raw_size'], size_mode=w['size_mode'], colour='%06x' % w['colors'][0],
                                  transparency=w['transparency']) for w in bars][:6],
                       texts=Cnt(texts).most_common(4),
                       root_modes=[(w['position_mode'], w['size_mode'], w['raw_size']) for w in roots][:3]))
    M['overlays'] = ov
    # ---- old against new (group id is roughly the order groups were added)
    era = []
    for lo in range(0, 1000, 100):
        gs = [g for g in groups if lo <= g < lo + 100]
        ws = [w for g in gs for w in defs[g]]
        tf2 = Cnt(fonts.get(w['font'], w['font']) for w in ws if w['kind'] == 4)
        era.append(dict(ids=f'{lo}-{lo + 99}', groups=len(gs), components=len(ws),
                        if1_share=round(sum(not w['if3'] for w in ws) / max(len(ws), 1), 3),
                        steelborder_groups=sum(1 for g in gs if g in onload_groups[227]),
                        script_built_share=round(sum(1 for g in gs for r in states.get(g, []) if r['child_index'] >= 0) /
                                                 max(1, sum(len(states.get(g, [])) for g in gs)), 3),
                        fonts=tf2.most_common(5),
                        static_sprite_frames=sum(1 for w in ws if w['kind'] == 5 and (w['sprites'][0] in (310, 311, 312, 313) or w['sprites'][1] in (310, 311, 312, 313)))))
    M['eras'] = era
    (HERE / 'harness' / 'measure.json').write_text(json.dumps(M, indent=1, default=list))
    print('ok', len(states), 'states')


_dimcache = {}


def _dims(cache, s):
    if s in _dimcache:
        return _dimcache[s]
    try:
        sh = C.sprite_sheet(cache, s); f = sh.frames[0]
        v = (sh.width, sh.height, f['x'], f['y'], f['w'], f['h'])
    except Exception:
        v = ()
    _dimcache[s] = v
    return v


def _fontm(cache, fid):
    f = C.font(cache, fid)
    digits = [f.metrics.advances[ord(c)] for c in '0123456789']
    return dict(id=fid, ascent=f.metrics.ascent, max_ascent=f.max_ascent, max_descent=f.max_descent,
                kerning=f.metrics.kerning is not None, digit_advance=sorted(set(digits)),
                space=f.metrics.advances[32], x_advance=f.metrics.advances[ord('x')], M_advance=f.metrics.advances[ord('M')])


def _dist(v):
    v = sorted(v)
    if not v:
        return {}
    q = lambda p: v[min(len(v) - 1, int(p * len(v)))]
    return dict(n=len(v), min=v[0], p25=q(.25), median=q(.5), p75=q(.75), max=v[-1])


if __name__ == '__main__':
    main()
