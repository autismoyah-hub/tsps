"""census_frames.py: infer and verify a RECIPE for every sprite frame family in use (held-out panels excluded).

A family is named by its four corner sprites (top-left, top-right, bottom-left, bottom-right) and the other sprites
it may draw (edges, backing, divider, ornaments, close). An INSTANCE is a top-left corner with the three others
under the same parent that close a box (the smallest one). Its BOX is top-left corner's top-left to the bottom-right
corner's bottom-right: W x H. Its PIECES are the family's sprites under the same parent that overlap the box
grown by 24px (and are not part of another instance).

Each piece's rectangle is written relative to the box, every number in the form that holds most often across
instances: a constant (k), or from the far side (W-k / H-k). The RECIPE of a family is, per sprite, the commonest
(x, y, w, h, tiled, transparency, flips) in those forms, plus the commonest DRAW ORDER of the sprites.

Verification, per instance: (1) EXACT: the recipe for the instance's W x H gives exactly the set of the instance's
pieces (same sprite, rectangle, tiling, transparency, flips; optional pieces may be absent: divider, close,
ornaments, backing, header band, by their presence in the instance); (2) PIXELS: the recipe's pieces and the real
pieces are drawn with iface.py (cache 242, TSPS blend) with the real parent's clip, and every pixel compared.
Writes recipes/out/frames.json and prints a summary per family."""
import sys, json, collections
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K
sys.path.insert(0, str(K.HERE))
import ifcache as C
import iface
iface.TSPS_BLEND = True

FAMILIES = {
    'steel': dict(corners=(310, 311, 312, 313), parts={314: 'top edge', 173: 'bottom edge', 172: 'left edge',
                  315: 'right edge', 297: 'backing', 2546: 'title divider', 535: 'close', 536: 'close (hover)'},
                  optional={297, 2546, 535, 536}),
    'stone': dict(corners=(824, 825, 826, 827), parts={820: 'top edge', 822: 'bottom edge', 821: 'left edge',
                  823: 'right edge', 297: 'backing', 828: 'title divider', 829: 'left ornament', 830: 'right ornament',
                  831: 'close'}, optional={297, 828, 829, 830, 831}),
    'stone2': dict(corners=(846, 847, 848, 849), parts={820: 'top edge', 822: 'bottom edge', 821: 'left edge',
                   823: 'right edge', 828: 'top edge (2)', 841: 'edge (841)', 297: 'backing', 829: 'left ornament',
                   830: 'right ornament', 831: 'close'}, optional={297, 828, 829, 830, 831, 841}),
    'bevel-dark': dict(corners=(913, 914, 915, 916), parts={917: 'left edge', 918: 'top edge', 919: 'right edge',
                       920: 'bottom edge', 297: 'backing', 897: 'dark backing', 1040: 'light backing'},
                       optional={297, 897, 1040}),
    'bevel-mid': dict(corners=(921, 922, 923, 924), parts={925: 'left edge', 926: 'top edge', 927: 'right edge',
                      928: 'bottom edge', 297: 'backing', 897: 'dark backing', 1040: 'light backing'},
                      optional={297, 897, 1040}),
    'bevel-light': dict(corners=(929, 930, 931, 932), parts={933: 'left edge', 934: 'top edge', 935: 'right edge',
                        936: 'bottom edge', 297: 'backing', 897: 'dark backing', 1040: 'light backing'},
                        optional={297, 897, 1040}),
    'bevel-beige': dict(corners=(1554, 1555, 1556, 1557), parts={1558: 'left edge', 1559: 'top edge',
                        1560: 'right edge', 1561: 'bottom edge', 1545: 'beige backing', 297: 'backing'},
                        optional={1545, 297}),
    'thin': dict(corners=(991, 992, 993, 994), parts={987: 'top edge', 989: 'bottom edge', 988: 'left edge',
                 990: 'right edge', 995: 'header band', 996: 'header band left end', 997: 'header band right end',
                 297: 'backing', 1040: 'light backing', 897: 'dark backing'},
                 optional={995, 996, 997, 297, 1040, 897}, free={995: 'y'}, tied={996: (995, 2), 997: (995, 2)}),
    'tile-grey': dict(corners=(1141, 1143, 1147, 1149), parts={1142: 'top edge', 1144: 'left edge', 1145: 'middle',
                      1146: 'right edge', 1148: 'bottom edge'}, optional=set()),
    'tile-red': dict(corners=(1150, 1152, 1156, 1158), parts={1151: 'top edge', 1153: 'left edge', 1154: 'middle',
                     1155: 'right edge', 1157: 'bottom edge'}, optional=set()),
}
BIG = 24
GROW = 20
BACKS = (297, 897, 1040, 1545)


def find_instances(rows, by, fam):
    tl, tr, bl, br = fam['corners']
    spr = lambda r: r['sprites'][0] if r['kind'] == 5 else None
    vis = [r for r in rows if r['kind'] == 5 and r['visible']]
    used = set()
    out = []
    for a in [r for r in vis if spr(r) == tl]:
        best = None
        for d in [r for r in vis if spr(r) == br and r['parent'] == a['parent'] and r['id'] not in used]:
            if d['abs'][0] < a['abs'][0] or d['abs'][1] < a['abs'][1]:
                continue
            b = next((r for r in vis if spr(r) == tr and r['parent'] == a['parent'] and r['abs'] == [d['abs'][0] + d['size'][0] - r['size'][0], a['abs'][1]] and r['id'] not in used), None)
            c = next((r for r in vis if spr(r) == bl and r['parent'] == a['parent'] and r['abs'] == [a['abs'][0], d['abs'][1] + d['size'][1] - r['size'][1]] and r['id'] not in used), None)
            if b is None or c is None:
                continue
            area = (d['abs'][0] - a['abs'][0]) * (d['abs'][1] - a['abs'][1])
            if best is None or area < best[0]:
                best = (area, b, c, d)
        if best is None:
            continue
        _, b, c, d = best
        box = [a['abs'][0], a['abs'][1], d['abs'][0] + d['size'][0] - a['abs'][0], d['abs'][1] + d['size'][1] - a['abs'][1]]
        members = [a, b, c, d]
        used.update(m['id'] for m in members)
        out.append(dict(box=box, members=members, parent=a['parent']))
    # parts: a piece belongs to the smallest instance (same parent) whose box grown by GROW px holds it entirely;
    # a backing only if it fills the box or the box inset by 1-3px on every side (otherwise it is a separate well)
    for r in vis:
        if r['id'] in used or spr(r) not in fam['parts']:
            continue
        rx, ry, rw, rh = r['abs'] + r['size']
        cands = []
        for inst in out:
            if inst['parent'] != r['parent']:
                continue
            x, y, w, h = inst['box']
            if spr(r) in BACKS:
                d = rx - x
                if not (0 <= d <= 3 and ry - y == d and rw == w - 2 * d and rh == h - 2 * d):
                    continue
            elif not (rx >= x - GROW and ry >= y - GROW and rx + rw <= x + w + GROW and ry + rh <= y + h + GROW):
                continue
            cands.append((w * h, inst))
        if cands:
            inst = min(cands, key=lambda c: c[0])[1]
            inst['members'].append(r); used.add(r['id'])
    for inst in out:
        inst['members'].sort(key=lambda r: r['order'])
        # piece key: the sprite, or sprite + 100000 x k for its k-th repeat in the instance (stone2 draws 828 twice)
        seen = collections.Counter()
        for r in inst['members']:
            s = r['sprites'][0]
            r['pk'] = s + PK * seen[s]
            seen[s] += 1
    return out


PK = 100000


def spr_of(pk):
    return pk % PK


def rel(inst, r):
    x, y, W, H = inst['box']
    return [r['abs'][0] - x, r['abs'][1] - y, r['size'][0], r['size'][1]]


def forms(v, full):
    return {('k', v), ('F', v - full)}


def key_of(r):
    return (bool(r['sprite_tiling']), r['transparency'], bool(r['flip_h']), bool(r['flip_v']))


def infer(instances):
    """Per sprite: for each of x, y, w, h the form with the most instances agreeing, and the commonest key."""
    # each GROUP weighs 1 in total (a group repeating one box 18 times must not outvote 10 groups)
    per_group = collections.Counter(i['group'] for i in instances)
    by_s = collections.defaultdict(list)
    for inst in instances:
        W, H = inst['box'][2:]
        wt = 1.0 / per_group[inst['group']]
        for r in inst['members']:
            by_s[r['pk']].append((rel(inst, r), W, H, key_of(r), wt))
    recipe = {}
    for s, xs in by_s.items():
        spec = []
        for k in range(4):
            full = lambda W, H: W if k in (0, 2) else H
            cnt = collections.Counter()
            for rr, W, H, _, wt in xs:
                for f in forms(rr[k], full(W, H)):
                    cnt[f] += wt
            spec.append(list(cnt.most_common(1)[0][0]))
        kcc = collections.Counter()
        for x in xs:
            kcc[x[3]] += x[4]
        kc = kcc.most_common(1)[0][0]
        recipe[s] = dict(rect=spec, tiled=kc[0], transparency=kc[1], flip_h=kc[2], flip_v=kc[3], uses=len(xs))
    return recipe


def realise(spec, W, H):
    out = []
    for k, (f, v) in enumerate(spec):
        full = W if k in (0, 2) else H
        out.append(v if f == 'k' else full + v)
    return out


def fmt(spec):
    names = ['x', 'y', 'w', 'h']
    s = []
    for k, (f, v) in enumerate(spec):
        F = 'W' if k in (0, 2) else 'H'
        s.append(str(v) if f == 'k' else (F if v == 0 else f'{F}{v:+d}'))
    return '(' + ', '.join(s) + ')'


def order_pattern(inst):
    return tuple(r['sprites'][0] for r in inst['members'])


_cache = None


def draw_pieces(pieces, parent_clip, box):
    """pieces: [(sprite, [x,y,w,h], key)] absolute; returns RGB of the box grown by BIG on two backgrounds."""
    global _cache
    if _cache is None:
        _cache = C.Cache(C.CACHE_242)
    x0, y0 = box[0] - BIG, box[1] - BIG
    Wc, Hc = box[2] + 2 * BIG, box[3] + 2 * BIG
    imgs = []
    for bg in (0xff000000, 0xffffffff):
        lay = {0: dict(parent=None, abs=[0, 0], size=[Wc, Hc], clip=[parent_clip[0] - x0, parent_clip[1] - y0,
                                                                      parent_clip[2] - x0, parent_clip[3] - y0], hidden=False)}
        byid = {0: dict(kind=0, if3=True, content=0, transparency=0, colors=[0, 0, 0, 0], listeners={}, triggers={},
                        file=0)}
        order = [0]
        for n, (s, (x, y, w, h), key) in enumerate(pieces, 1):
            lay[n] = dict(parent=0, abs=[x - x0, y - y0], size=[w, h], clip=[x - x0, y - y0, x - x0 + w, y - y0 + h],
                          hidden=False)
            byid[n] = dict(kind=5, if3=True, content=0, sprites=[s, -1], sprite_tiling=key[0], transparency=key[1],
                           flip_h=key[2], flip_v=key[3], outline=0, sprite_shadow=0, sprite_angle=0, colors=[0, 0, 0, 0],
                           listeners={}, triggers={}, file=n)
            order.append(n)
        ras = iface.Raster(Wc, Hc, bg)
        ras, _ = iface.draw(_cache, order, lay, byid, ras, (Wc, Hc))
        imgs.append(ras.rgb().astype(np.int16))
    return imgs


def canon_order(rec, L):
    """Draw order: for every pair of sprites, which is drawn first in most groups (weighted per group); sorted by the
    number of sprites each one is drawn after."""
    rank = collections.defaultdict(list)
    for i in L:
        for n, r in enumerate(i['members']):
            rank[r['pk']].append(n / max(len(i['members']) - 1, 1))
    pg = collections.Counter(i['group'] for i in L)
    before = collections.Counter()
    for i in L:
        seq = [r['pk'] for r in i['members']]
        for a_ in range(len(seq)):
            for b_ in range(a_ + 1, len(seq)):
                before[(seq[a_], seq[b_])] += 1.0 / pg[i['group']]
    return sorted(rec, key=lambda s: (sum(1 for t in rec if t != s and before[(t, s)] > before[(s, t)]),
                                      np.mean(rank[s]) if rank[s] else 0))


def recipe_pieces(fam, rec, canon, W, H, present, have=None):
    """The pieces a recipe draws for a W x H box, in draw order: [(sprite, [x, y, w, h] rel. box, key)]. Optional
    sprites only if present; free parameters (the header band's y) taken from `have` when given."""
    want = [(s, realise(rec[s]['rect'], W, H), (rec[s]['tiled'], rec[s]['transparency'], rec[s]['flip_h'],
                                                 rec[s]['flip_v']))
            for s in canon if s in present or spr_of(s) not in fam['optional']]
    for s, ax in fam.get('free', {}).items():
        act = next((rr for t, rr, _ in (have or []) if t == s), None)
        if act is None:
            continue
        for n, (t, rr, k) in enumerate(want):
            if t == s:
                want[n] = (t, [rr[0], act[1], rr[2], rr[3]], k)
            elif fam.get('tied', {}).get(t, (None,))[0] == s:
                want[n] = (t, [rr[0], act[1] + fam['tied'][t][1], rr[2], rr[3]], k)
    return want


def evaluate(fam, rec, canon, L):
    out = []
    for i in L:
        W, H = i['box'][2:]
        present = {r['pk'] for r in i['members']}
        have = [(r['pk'], rel(i, r), key_of(r)) for r in i['members']]
        if not present <= set(rec):
            out.append((i, False, -1, dict(extra_sprites=sorted(present - set(rec)))))
            continue
        want = recipe_pieces(fam, rec, canon, W, H, present, have)
        ok = sorted(map(str, want)) == sorted(map(str, have))
        bx, by_ = i['box'][:2]
        wa = [(spr_of(s), [rr[0] + bx, rr[1] + by_, rr[2], rr[3]], k) for s, rr, k in want]
        ha = [(spr_of(s), [rr[0] + bx, rr[1] + by_, rr[2], rr[3]], k) for s, rr, k in have]
        A = draw_pieces(wa, i['parent_clip'], i['box'])
        B = draw_pieces(ha, i['parent_clip'], i['box'])
        nd = int(((A[0] != B[0]).any(-1) | (A[1] != B[1]).any(-1)).sum())
        ws = {str(x) for x in want}; hs = {str(x) for x in have}
        out.append((i, ok, nd, dict(missing_in_real=sorted(ws - hs)[:4], extra_in_real=sorted(hs - ws)[:4])))
    return out


def run_all(insts, max_variants=4):
    """Per family: variant 1 = the group-weighted majority recipe over all instances; the instances it draws
    pixel-exactly are its holders; variant 2 = the majority over the rest; and so on while at least 2 groups
    remain and the new variant holds for at least 2 groups."""
    result = {}
    for name, L in insts.items():
        fam = FAMILIES[name]
        rest = list(L)
        variants = []
        while rest and len({i['group'] for i in rest}) >= 2 and len(variants) < max_variants:
            # seed: the instances sharing the commonest (group-weighted) set of (sprite, tiling, transparency, flips)
            pg = collections.Counter(i['group'] for i in rest)
            ksig = lambda i: tuple(sorted({(r['pk'],) + key_of(r) for r in i['members']
                                           if r['sprites'][0] not in fam['optional']}))
            kc = collections.Counter()
            for i in rest:
                kc[ksig(i)] += 1.0 / pg[i['group']]
            top = kc.most_common(1)[0][0]
            seed = [i for i in rest if ksig(i) == top]
            rec = infer(seed)
            # optional sprites seen only outside the seed still need a recipe line
            extra = infer(rest)
            for s, v in extra.items():
                if s < PK and s in fam['optional']:
                    rec.setdefault(s, v)
            # draw order from the seed; sprites seen only outside it go where the rest puts them
            canon = canon_order({s: v for s, v in rec.items() if any(r['pk'] == s for i in seed for r in i['members'])}, seed)
            for s in canon_order(rec, rest):
                if s not in canon:
                    canon.append(s)
            ev = evaluate(fam, rec, canon, rest)
            hold = [i for i, ok, nd, _ in ev if nd == 0]
            hg = {i['group'] for i in hold}
            if len(hg) < 2 and variants:
                break
            if not hold:
                break
            ex = [dict(group=i['group'], box=i['box'], pixels=nd, **d) for i, ok, nd, d in ev if nd != 0]
            variants.append(dict(
                holds_instances=len(hold), holds_groups=len(hg), group_ids=sorted(hg),
                exact_rects=sum(1 for _, ok, nd, _ in ev if ok),
                sizes=collections.Counter(tuple(i['box'][2:]) for i in hold).most_common(6),
                optional_present=dict(collections.Counter(r['sprites'][0] for i in hold for r in i['members']
                                                          if r['sprites'][0] in fam['optional'])),
                draw_order=canon,
                recipe={str(s): dict(role=(['top-left corner', 'top-right corner', 'bottom-left corner',
                                            'bottom-right corner'][fam['corners'].index(s)] if s in fam['corners']
                                           else fam['parts'].get(spr_of(s), '?') + (f' (repeat {s // PK})' if s >= PK else '')),
                                     rect=fmt(rec[s]['rect']), raw=rec[s]['rect'],
                                     **{k: v for k, v in rec[s].items() if k != 'rect'}) for s in canon}))
            rest = [i for i, ok, nd, _ in ev if nd != 0]
        groups = sorted({i['group'] for i in L})
        result[name] = dict(corners=fam['corners'], instances=len(L), groups=len(groups), variants=variants,
                            unexplained=[dict(group=i['group'], box=i['box']) for i in rest],
                            unexplained_groups=sorted({i['group'] for i in rest}))
        print(f'== {name}: {len(L)} instances in {len(groups)} groups; {len(variants)} variant(s); unexplained '
              f'{len(rest)} instances in groups {result[name]["unexplained_groups"]}')
        for n, v in enumerate(variants, 1):
            print(f'  -- variant {n}: pixel-exact for {v["holds_instances"]} instances in {v["holds_groups"]} groups '
                  f'{v["group_ids"][:14]}{"..." if len(v["group_ids"]) > 14 else ""}; rect-exact {v["exact_rects"]}; '
                  f'optional {v["optional_present"]}; sizes {v["sizes"][:3]}')
            for s in v['draw_order']:
                e = v['recipe'][str(s)]
                print(f'     {s:5d} {e["role"]:<24} {e["rect"]:<28} tiled {int(e["tiled"])} tr {e["transparency"]} '
                      f'flip {int(e["flip_h"])}{int(e["flip_v"])} uses {e["uses"]}')
    return result


def main():
    only = sys.argv[1:]
    insts = collections.defaultdict(list)
    for g, rows, by in K.all_states():
        for name, fam in FAMILIES.items():
            if only and name not in only:
                continue
            for inst in find_instances(rows, by, fam):
                inst['group'] = g
                inst['parent_clip'] = by[inst['parent']]['clip'] if inst['parent'] in by else [0, 0, 765, 503]
                insts[name].append(inst)
    result = run_all(insts)
    od = K.HERE / 'recipes' / 'out'
    od.mkdir(exist_ok=True)
    tag = '-'.join(only) if only else 'all'
    (od / f'frames-{tag}.json').write_text(json.dumps(result, indent=1, default=str))


if __name__ == '__main__':
    main()
