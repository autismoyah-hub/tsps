"""Read a real frame base (skeleton) the way its designers meant it: groups, joints, hierarchy, and a drawn rig.

usage: skelread.py list BASE [NPC]            every group: type, labels, joint block, parent; then the joint tree
       skelread.py rig OUT.png BASE NPC       the NPC's rest mesh, vertices coloured by the joint that owns them,
                                              pivots marked (cross = centroid of the origin group's labels on this
                                              mesh), a line from each joint's pivot to its parent's
       skelread.py census                     sizes of every base that serves an NPC: groups, labels, joints by tile size
       skelread.py pick                       candidate bases (small, player, quadruped, dragon, boss) with one NPC each

How the hierarchy is recovered (the format has no parent pointers):
  * a JOINT BLOCK is an origin group (type 0) and the non-origin groups that follow it up to the next origin: the
    frame decoder itself pairs them (animation.rs Frame::decode inserts the nearest preceding origin before a
    non-origin transform), so the pairing is the format's, not a guess;
  * the block MOVES the union of the labels its translate / rotate / scale groups list (its "moved set");
  * block A is an ANCESTOR of block B when B's moved set is a proper subset of A's (a chain is written as shrinking
    cumulative label lists: shoulder lists arm + forearm + hand, elbow lists forearm + hand, wrist lists hand);
    the PARENT is the ancestor with the smallest moved set; equal moved sets are chained in file order;
  * a label is OWNED by the block with the smallest moved set containing it (the last bone it follows).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, subprocess, collections, colorsys
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'research-animation'))
import animlib as A
EXE = _AP.fix(r'<CLIENT_BIN>\asset_npc.exe')
CENSUS = HERE.parent / 'research-npc' / 'census'
KIND = {0: 'origin', 1: 'translate', 2: 'rotate', 3: 'scale', 5: 'alpha'}


def blocks_of(base):
    """[{origin: group index or None, groups: [non-origin indices], pivot: set, moved: set, kinds: [...]}]"""
    out = []
    cur = None
    for i, t in enumerate(base['types']):
        if t == 0:
            cur = dict(origin=i, groups=[], pivot=set(base['labels'][i]), moved=set(), kinds=[])
            out.append(cur)
        else:
            if cur is None:
                cur = dict(origin=None, groups=[], pivot=set(), moved=set(), kinds=[]); out.append(cur)
            cur['groups'].append(i); cur['kinds'].append(t)
            if t in (1, 2, 3): cur['moved'] |= set(base['labels'][i])
    return out


def tree_of(blocks):
    """parent index per block (None for roots), by smallest proper superset; equal sets chain in file order."""
    parent = [None] * len(blocks)
    for b, B in enumerate(blocks):
        if not B['moved']: continue
        best = None
        for a, Ab in enumerate(blocks):
            if a == b or not Ab['moved']: continue
            if B['moved'] < Ab['moved'] or (B['moved'] == Ab['moved'] and a < b):
                if best is None or len(Ab['moved']) < len(blocks[best]['moved']) or (len(Ab['moved']) == len(blocks[best]['moved']) and a > best):
                    best = a
        parent[b] = best
    return parent


def owner_of(blocks):
    own = {}
    for b, B in enumerate(blocks):
        for l in B['moved']:
            if l not in own or len(B['moved']) < len(blocks[own[l]]['moved']): own[l] = b
    return own


def merged(npc):
    t = HERE / '_merged.txt'
    spec = npc if ':' in str(npc) else f'npc:{npc}'
    r = subprocess.run([EXE, str(A.CACHE), 'merged', spec, str(t)], capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    V, L, F = [], [], []
    for line in t.read_text().splitlines():
        p = line.split()
        if p and p[0] == 'v': V.append((int(p[1]), -int(p[2]), int(p[3]))); L.append(int(p[4]))
        elif p and p[0] == 'f': F.append((int(p[1]), int(p[2]), int(p[3])))
    t.unlink()
    return V, L, F, r.stdout.strip()


def seqs_on(bid):
    rows = [json.loads(l) for l in (CENSUS / 'sequences.jsonl').read_text().splitlines() if l.strip()]
    return [r['id'] for r in rows if r['skeleton'] == bid and r['kind'] == 'classic']


def usage(bid, base):
    """per group: in how many sequences it appears, in how many it changes, its commonest value"""
    use = collections.Counter(); move = collections.Counter(); vals = collections.defaultdict(collections.Counter)
    ids = seqs_on(bid); n = 0
    for sid in ids:
        try: s, b, _, frames = A.sequence_frames(sid)
        except Exception: continue
        if b != bid: continue
        n += 1
        for g in {g for f in frames for g in f['groups']}:
            d = [128] * 3 if base['types'][g] == 3 else [0] * 3
            vs = [tuple(f['groups'].get(g, d)) for f in frames]
            use[g] += 1
            if any(v != vs[0] for v in vs): move[g] += 1
            for v in vs: vals[g][v] += 1
    return n, use, move, vals


def cmd_list(bid, npc=None):
    base = A.frame_base(bid)
    T = base['types']
    blocks = blocks_of(base); parent = tree_of(blocks); own = owner_of(blocks)
    n, use, move, vals = usage(bid, base)
    kinds = collections.Counter(KIND.get(t, str(t)) for t in T)
    alll = sorted({l for ls in base['labels'] for l in ls})
    print(f'frame base {bid}: {len(T)} groups {dict(kinds)}; {len(alll)} distinct labels (max {max(alll) if alll else None}); '
          f'{len(blocks)} joint blocks; tail {len(base["tail"])} bytes; {n} classic sequences decoded on it')
    mesh = None
    if npc:
        V, L, F, note = merged(npc); mesh = (V, L)
        have = set(L)
        used_faces = {i for f in F for i in f}
        loose = [i for i in range(len(V)) if i not in used_faces]
        print(f'mesh {note}: {len(have)} labels on the mesh; {len(have - set(alll))} mesh labels in no group {sorted(have - set(alll))[:12]}; '
              f'{len(set(alll) - have)} base labels this mesh does not carry; {len(loose)} vertices in no face (labels {sorted({L[i] for i in loose})[:20]})')
    blk_of = {}
    for b, B in enumerate(blocks):
        if B['origin'] is not None: blk_of[B['origin']] = b
        for g in B['groups']: blk_of[g] = b
    for i, t in enumerate(T):
        ls = base['labels'][i]
        b = blk_of[i]
        extra = ''
        if mesh:
            V, L = mesh
            idx = [k for k, l in enumerate(L) if l in set(ls)]
            if idx:
                c = [sum(V[k][a] for k in idx) / len(idx) for a in range(3)]
                extra = f' | {len(idx):4d} v, centre x {c[0]:6.0f} h {c[1]:5.0f} z {c[2]:6.0f}'
            else: extra = ' |    0 v'
        u = f'used {use[i]:3d} moved {move[i]:3d}' if n else ''
        common = ''
        if vals[i]:
            (v, c), = vals[i].most_common(1); tot = sum(vals[i].values())
            if t == 0: common = f' offset {list(v)} {100 * c // tot}%'
        lab = str(ls) if len(ls) <= 14 else str(ls[:12])[:-1] + f', ... {len(ls)} labels]'
        print(f'  g{i:3d} {KIND.get(t, t):9s} block {b:3d} {u}{common} {lab}{extra}')
    print('joint tree (block: origin group -> groups; own labels = labels this joint is the LAST to move):')
    kids = collections.defaultdict(list)
    for b, p in enumerate(parent): kids[p].append(b)
    def show(b, depth):
        B = blocks[b]
        mine = sorted(l for l, o in own.items() if o == b)
        ks = '+'.join(KIND[k][0].upper() for k in B['kinds'])
        piv = sorted(B['pivot'])
        where = 'own' if B['pivot'] and B['pivot'] <= set(mine) else 'inside' if B['pivot'] and B['pivot'] <= B['moved'] else 'outside' if B['pivot'] and not (B['pivot'] & B['moved']) else 'mixed' if B['pivot'] else 'none'
        print('  ' + '   ' * depth + f'block {b} g{B["origin"]}->{B["groups"]} [{ks}] moves {len(B["moved"])} labels; pivot labels {piv[:8]}{"..." if len(piv) > 8 else ""} ({where}); own {mine[:10]}{"..." if len(mine) > 10 else ""}')
        for k in kids[b]: show(k, depth + 1)
    for b in kids[None]:
        if blocks[b]['moved']: show(b, 0)
    lone = [b for b in kids[None] if not blocks[b]['moved']]
    if lone:
        print('  blocks that move no vertex (alpha / empty): ' + ', '.join(f'block {b} g{blocks[b]["origin"]}->{blocks[b]["groups"]} kinds {[KIND.get(k, k) for k in blocks[b]["kinds"]]}' for b in lone))
    depth = {}
    def dep(b):
        if b not in depth: depth[b] = 0 if parent[b] is None else dep(parent[b]) + 1
        return depth[b]
    ds = [dep(b) for b in range(len(blocks)) if blocks[b]['moved']]
    where = collections.Counter()
    for b, B in enumerate(blocks):
        if not B['moved'] or not B['pivot']: where['no pivot labels' if B['moved'] else 'x'] += 1; continue
        mine = {l for l, o in own.items() if o == b}
        where['pivot = joint\'s own labels' if B['pivot'] <= mine else 'pivot inside the moved set' if B['pivot'] <= B['moved'] else 'pivot outside the moved set' if not (B['pivot'] & B['moved']) else 'pivot straddles'] += 1
    comp = collections.Counter('+'.join(KIND[k][0].upper() for k in B['kinds']) for B in blocks)
    print(f'summary: joints {sum(1 for B in blocks if B["moved"])}, deepest chain {max(ds) + 1 if ds else 0}, block shapes {dict(comp)}, pivots {dict(where)}')
    return base, blocks, parent, own


def colour(k):
    r, g, b = colorsys.hsv_to_rgb((k * 0.381966 + 0.11) % 1.0, 0.72, 1.0); return int(r * 255), int(g * 255), int(b * 255)


def cmd_rig(out, bid, npc):
    from PIL import Image, ImageDraw, ImageFont
    base = A.frame_base(bid)
    blocks = blocks_of(base); parent = tree_of(blocks); own = owner_of(blocks)
    V, L, F, note = merged(npc)
    xs = [v[0] for v in V]; hs = [v[1] for v in V]; zs = [v[2] for v in V]
    span = max(max(xs) - min(xs), max(hs) - min(hs), max(zs) - min(zs))
    s = max(1.0, min(6.0, 900 / span)); pad = 24
    W1 = int((max(zs) - min(zs)) * s) + 2 * pad; W2 = int((max(xs) - min(xs)) * s) + 2 * pad
    H1 = int((max(hs) - min(hs)) * s) + 2 * pad; H2 = int((max(xs) - min(xs)) * s) + 2 * pad
    im = Image.new('RGB', (W1 + W2, H1 + H2 + 40), (20, 20, 24)); d = ImageDraw.Draw(im)
    try: font = ImageFont.truetype('segoeuib.ttf', 13); big = ImageFont.truetype('segoeuib.ttf', 16)
    except Exception: font = big = ImageFont.load_default()
    side = lambda v: (pad + (v[2] - min(zs)) * s, 30 + pad + (max(hs) - v[1]) * s)
    front = lambda v: (W1 + pad + (v[0] - min(xs)) * s, 30 + pad + (max(hs) - v[1]) * s)
    top = lambda v: (pad + (v[2] - min(zs)) * s, 30 + H1 + pad + (v[0] - min(xs)) * s)
    by = collections.defaultdict(list)
    for i, l in enumerate(L): by[l].append(i)
    piv = {}
    for b, B in enumerate(blocks):
        idx = [i for l in B['pivot'] for i in by.get(l, ())]
        if idx and B['moved']: piv[b] = tuple(sum(V[i][a] for i in idx) / len(idx) for a in range(3))
    for proj in (side, front, top):
        for f in F:
            p = [proj(V[i]) for i in f]; d.line(p + [p[0]], fill=(58, 58, 66))
        for v, l in zip(V, L):
            p = proj(v); c = colour(own[l]) if l in own else (120, 120, 120)
            d.rectangle([p[0] - 1, p[1] - 1, p[0] + 1, p[1] + 1], fill=c)
        for b, c in piv.items():
            if parent[b] in piv:
                d.line([proj(c), proj(piv[parent[b]])], fill=(235, 235, 235), width=1)
        for b, c in piv.items():
            p = proj(c); col = colour(b)
            d.line([p[0] - 5, p[1], p[0] + 5, p[1]], fill=col, width=2); d.line([p[0], p[1] - 5, p[0], p[1] + 5], fill=col, width=2)
            d.ellipse([p[0] - 6, p[1] - 6, p[0] + 6, p[1] + 6], outline=(255, 255, 255))
            if proj is side: d.text((p[0] + 7, p[1] - 7), str(blocks[b]['origin']), fill=col, font=font, stroke_width=2, stroke_fill=(0, 0, 0))
    d.text((6, 4), f'frame base {bid} on {note.split(":")[0]}: {len(base["types"])} groups, {sum(1 for B in blocks if B["moved"])} joints. '
                   f'side (z across) | front (x across) | top. Vertex colour = owning joint; cross = pivot (numbered by its origin group); white line = to parent', fill=(230, 230, 230), font=big)
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True)
    im.save(out); print(out, im.size, f'{len(piv)} pivots drawn; {len(V)} vertices')


def cmd_census():
    npcs = [json.loads(l) for l in (CENSUS / 'npcs.jsonl').read_text().splitlines() if l.strip()]
    seqs = {}
    for l in (CENSUS / 'sequences.jsonl').read_text().splitlines():
        if l.strip():
            r = json.loads(l); seqs[r['id']] = r
    models = {}
    for l in (CENSUS / 'models.jsonl').read_text().splitlines():
        if l.strip():
            r = json.loads(l); models[r['id']] = r
    rows = {}
    for n in npcs:
        s = seqs.get(n['anims'][0]) or seqs.get(n['anims'][3])
        if not s or s['kind'] != 'classic' or s['skeleton'] < 0: continue
        key = (s['skeleton'], tuple(n['models']))
        if key in rows: continue
        labels = {l for m in n['models'] if m in models for l in models[m]['labels']}
        faces = sum(models[m]['faces'] for m in n['models'] if m in models)
        rows[key] = (n['size'], s['skeleton'], len(labels), faces)
    stat = {}
    cache = {}
    def info(b):
        if b not in cache:
            base = A.frame_base(b); bl = blocks_of(base); par = tree_of(bl)
            joints = sum(1 for B in bl if B['moved'])
            depth = {}
            def dep(x):
                if x not in depth: depth[x] = 0 if par[x] is None else dep(par[x]) + 1
                return depth[x]
            ds = [dep(i) + 1 for i in range(len(bl)) if bl[i]['moved']]
            kinds = collections.Counter(base['types'])
            cache[b] = (len(base['types']), joints, max(ds) if ds else 0, len({l for ls in base['labels'] for l in ls}), kinds)
        return cache[b]
    q = lambda xs, p: sorted(xs)[min(len(xs) - 1, int(p * len(xs)))]
    print('distinct (frame base, model list) pairs standing on a classic sequence, by tile size (player base 0 excluded in the second line of each size):')
    print('size | n | groups 10/50/90 | joints 10/50/90 | chain depth 50/90 | labels on the mesh 10/50/90 | faces 10/50/90')
    for size in sorted({r[0] for r in rows.values()}):
        for excl in (False, True):
            rs = [r for r in rows.values() if r[0] == size and not (excl and r[1] == 0)]
            if not rs: continue
            g = [info(r[1])[0] for r in rs]; j = [info(r[1])[1] for r in rs]; dp = [info(r[1])[2] for r in rs]; lb = [r[2] for r in rs]; fc = [r[3] for r in rs]
            print(f'{size:4d}{" no player" if excl else "          "} | {len(rs):5d} | {q(g, .1)}/{q(g, .5)}/{q(g, .9)} | {q(j, .1)}/{q(j, .5)}/{q(j, .9)} | {q(dp, .5)}/{q(dp, .9)} | {q(lb, .1)}/{q(lb, .5)}/{q(lb, .9)} | {q(fc, .1)}/{q(fc, .5)}/{q(fc, .9)}')
    bases = sorted({r[1] for r in rows.values()})
    tot = collections.Counter()
    shapes = collections.Counter(); pivkind = collections.Counter(); first = collections.Counter(); alpha = 0; scale = 0; last_kinds = collections.Counter()
    for b in bases:
        base = A.frame_base(b); tot.update(KIND.get(t, str(t)) for t in base['types'])
        bl = blocks_of(base); own = owner_of(bl)
        for B in bl:
            shapes['+'.join(KIND.get(k, str(k))[0].upper() for k in B['kinds']) or '(origin only)'] += 1
            if B['moved'] and B['pivot']:
                mine = {l for l, o in own.items() if o == bl.index(B)}
                pivkind['own' if B['pivot'] <= mine else 'inside' if B['pivot'] <= B['moved'] else 'outside' if not (B['pivot'] & B['moved']) else 'straddles'] += 1
            elif B['moved']: pivkind['no pivot labels'] += 1
        alll = {l for ls in base['labels'] for l in ls}
        if bl and bl[0]['moved'] == alll: first['first block moves every label'] += 1
        else: first['first block does not move every label'] += 1
        alpha += 5 in base['types']; scale += 3 in base['types']
    print(f'{len(bases)} bases serve those NPCs. group types over them: {dict(tot)}')
    print(f'bases with an alpha group: {alpha}; with a scale group: {scale}; {dict(first)}')
    print('joint block shapes (non-origin groups after one origin): ' + ', '.join(f'{k}: {v}' for k, v in shapes.most_common(14)))
    print(f'where the pivot labels sit: {dict(pivkind)}')


def cmd_pick():
    npcs = [json.loads(l) for l in (CENSUS / 'npcs.jsonl').read_text().splitlines() if l.strip()]
    seqs = {}
    for l in (CENSUS / 'sequences.jsonl').read_text().splitlines():
        if l.strip():
            r = json.loads(l); seqs[r['id']] = r
    seen = {}
    for n in npcs:
        s = seqs.get(n['anims'][0])
        if s and s['kind'] == 'classic' and s['skeleton'] >= 0 and n['models']:
            seen.setdefault(s['skeleton'], []).append(n)
    for b in sorted(seen, key=lambda b: len(A.frame_base(b)['types'])):
        base = A.frame_base(b)
        names = collections.Counter(n['name'] for n in seen[b])
        print(f'base {b:5d} groups {len(base["types"]):3d} npcs {len(seen[b]):4d} first {seen[b][0]["id"]} size {seen[b][0]["size"]} {list(names.most_common(4))}')


if __name__ == '__main__':
    c = sys.argv[1]
    if c == 'list': cmd_list(int(sys.argv[2]), sys.argv[3] if len(sys.argv) > 3 else None)
    elif c == 'rig': cmd_rig(sys.argv[2], int(sys.argv[3]), sys.argv[4])
    elif c == 'census': cmd_census()
    elif c == 'pick': cmd_pick()
