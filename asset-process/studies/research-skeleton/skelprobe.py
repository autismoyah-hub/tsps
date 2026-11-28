"""skelprobe.py: numbers for an animator on a skeleton of ours, from the integer port of the client's deform code.

usage: skelprobe.py axes RIG.json MODEL.bin [--amount 32]
           for every turn channel: where the centroid of the joint's own vertices goes for +AMOUNT on x, y, z
           (dx, dheight, dz; front is -z, the creature's left is +x)
       skelprobe.py clip RIG.json MODEL.bin CLIPDIR [--floor 0]
           per frame of a loose clip: lowest point (height), box, and per joint the centroid of its own vertices;
           flags any vertex below the floor
       skelprobe.py clear RIG.json MODEL.bin CLIPDIR --centre a,b --inner x,y --outer p,q
           per frame: the largest distance of the INNER labels' vertices from the centroid of the CENTRE labels, and
           the smallest distance of the OUTER labels' vertices (name prefixes allowed: l_ matches l_upper ...); a
           frame where an outer part comes inside the inner sphere is flagged (parts passing through each other)
       skelprobe.py rigid RIG.json MODEL.bin CLIPDIR [CLIPDIR...]
           the stretch and tear audit: over every frame, the largest change in length of any mesh edge, split into
           edges inside one label (a rigid part: should stay within rounding) and edges joining two labels (a skin
           that spans a joint: this is where a limb tears)
"""
import sys, json, pathlib
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent / 'research-animation')); sys.path.insert(0, str(HERE.parent / 'research-npc'))
import animlib as A, skelanim, meshio


def load(rigpath, model):
    rig, base, chan = skelanim.load_rig(rigpath)
    m = meshio.load(model)
    return rig, base, chan, [list(v) for v in m['vertices']], list(m['vertex_skins']), m


def cen(V, idx):
    return [sum(V[i][a] for i in idx) / len(idx) for a in range(3)]


if __name__ == '__main__':
    cmd = sys.argv[1]
    rig, base, chan, V, L, m = load(sys.argv[2], sys.argv[3])
    if cmd == 'axes':
        amount = int(sys.argv[sys.argv.index('--amount') + 1]) if '--amount' in sys.argv else 32
        for j, e in rig['joints'].items():
            if 'turn' not in e['channels']: continue
            own = set(e['own_labels']) - set(e['pivot_labels']) or set(e['own_labels'])
            idx = [i for i, l in enumerate(L) if l in set(base['labels'][e['channels']['turn']['group']]) and (l in own or not own)]
            if not idx: continue
            c0 = cen(V, idx); out = []
            for a, n in enumerate('xyz'):
                v = [0, 0, 0]; v[a] = amount
                P = A.apply_frame([list(x) for x in V], L, dict(base=0, groups={e['channels']['turn']['group']: v}), base)
                c = cen(P, idx); out.append(f'{n}+{amount}: dx {c[0] - c0[0]:+6.1f} dh {-(c[1] - c0[1]):+6.1f} dz {c[2] - c0[2]:+6.1f}')
            print(f'{j:11s} own centre x {c0[0]:6.1f} h {-c0[1]:6.1f} z {c0[2]:6.1f} | ' + ' | '.join(out))
    elif cmd == 'clip':
        d = pathlib.Path(sys.argv[4]); floor = int(sys.argv[sys.argv.index('--floor') + 1]) if '--floor' in sys.argv else 0
        s = A.decode_sequence((d / 'seq.dat').read_bytes())
        for k, fid in enumerate(s['frames']):
            f = A.decode_frame((d / f'frame_{fid >> 16}_{fid & 65535}.dat').read_bytes(), base)
            P = A.apply_frame([list(x) for x in V], L, f, base)
            lo = [min(p[a] for p in P) for a in range(3)]; hi = [max(p[a] for p in P) for a in range(3)]
            under = sum(1 for p in P if -p[1] < floor)
            print(f'f{k:2d} len {s["lengths"][k]:3d}: x {lo[0]:5d}..{hi[0]:5d} height {-hi[1]:5d}..{-lo[1]:5d} z {lo[2]:5d}..{hi[2]:5d}' + (f'  {under} VERTICES BELOW THE FLOOR' if under else ''))
    elif cmd == 'clear':
        import math
        d = pathlib.Path(sys.argv[4])
        def labs(flag):
            pre = sys.argv[sys.argv.index(flag) + 1].split(',')
            return {v for n, v in rig['labels'].items() if any(n == q or (q.endswith('_') and n.startswith(q)) for q in pre)}
        cl, il, ol = labs('--centre'), labs('--inner'), labs('--outer')
        s = A.decode_sequence((d / 'seq.dat').read_bytes()); worst = 1e9; bad = 0
        for k, fid in enumerate(s['frames']):
            f = A.decode_frame((d / f'frame_{fid >> 16}_{fid & 65535}.dat').read_bytes(), base)
            P = A.apply_frame([list(x) for x in V], L, f, base)
            c = cen(P, [i for i, l in enumerate(L) if l in cl])
            inner = max(math.dist(P[i], c) for i, l in enumerate(L) if l in il)
            outer = min(math.dist(P[i], c) for i, l in enumerate(L) if l in ol)
            worst = min(worst, outer - inner); bad += outer < inner
            if outer < inner: print(f'f{k:2d}: outer part {outer:.1f} from the centre, inner sphere reaches {inner:.1f}: THROUGH by {inner - outer:.1f}')
        print(f'{d.name}: {len(s["frames"])} frames, {bad} with an outer vertex inside the inner sphere; smallest clearance {worst:.1f} units')
    elif cmd == 'rigid':
        import math
        E = set()
        for f in m['faces']:
            for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): E.add((min(a, b), max(a, b)))
        same = [(a, b) for a, b in E if L[a] == L[b]]; cross = [(a, b) for a, b in E if L[a] != L[b]]
        inv = {v: n for n, v in rig['labels'].items()}
        print(f'{len(E)} edges: {len(same)} inside one label, {len(cross)} joining two labels ' + str(sorted({tuple(sorted((inv.get(L[a], L[a]), inv.get(L[b], L[b])))) for a, b in cross})[:8]))
        for dname in sys.argv[4:]:
            d = pathlib.Path(dname); sq = A.decode_sequence((d / 'seq.dat').read_bytes()); w1 = w2 = 0.0; where = None
            scaled = {g for g, t in enumerate(base['types']) if t == 3}
            for k, fid in enumerate(sq['frames']):
                f = A.decode_frame((d / f'frame_{fid >> 16}_{fid & 65535}.dat').read_bytes(), base)
                sc = {l for g in f['groups'] if g in scaled for l in base['labels'][g]}
                P = A.apply_frame([list(x) for x in V], L, f, base)
                for a, b in same:
                    if L[a] in sc: continue
                    e = abs(math.dist(P[a], P[b]) - math.dist(V[a], V[b]))
                    if e > w1: w1, where = e, (k, inv.get(L[a], L[a]))
                for a, b in cross:
                    w2 = max(w2, abs(math.dist(P[a], P[b]) - math.dist(V[a], V[b])))
            print(f'{d.name}: {len(sq["frames"])} frames; largest length change of an edge inside one label {w1:.2f} units (frame, part {where}); of an edge joining two labels {w2:.2f}')
