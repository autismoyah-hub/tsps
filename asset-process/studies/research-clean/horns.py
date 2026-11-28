"""Horns (and other long rigid features) measured as objects. Selectors are given by hand (colours and a height band).
Per horn: faces, length / creature height, sides (faces cut by a plane across the middle), share of its inner edge
length that is hard (50 degrees or more) / firm (30-50) / soft, tip angle, how it joins (welded or a separate shell,
closed?), the fold where it meets the skull, and the lit tone step there."""
import sys, pathlib, json, collections, math
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl
W = cl.B
# entry, name, colours (h, s, l) of the horn, up band, side (x sign) to take
SETS = [
    ('npc:12812', 'Minotaur horn', [(5, 7, 55), (5, 7, 43), (5, 2, 16), (5, 2, 12)], (0.86, 1.0), -1),
    ('npc:2025', 'Greater demon horn', [(0, 0, 0)], (0.70, 0.9), -1),
    ('npc:15625', 'Bull horn', None, None, -1),
    ('npc:13599', 'Tormented Demon horn', None, None, -1),
    ('npc:14176', 'Yama horn', None, None, -1),
    ('npc:2215', 'Graardor tusk', [(7, 1, 82)], (0.8, 0.95), -1),
    ('def:' + str(W / 'research-paint' / 'repaint' / 'watcher-2-repaint.json'), 'WATCHER horn', 'shell', (0.8, 1.0), -1),
    ('def:' + str(W / 'creature-star-hound-3' / 'creature-star-hound-3.json'), 'STAR HOUND horn', 'shell', (0.7, 1.0), -1),
]

def horn_stats(c, fs):
    cl.topology(c); V, F = c['V'], c['F']; fs = np.asarray(fs); fset = set(int(i) for i in fs)
    skin = c['drawn'] & (c['A'] == 0); used = np.unique(F[skin]); h = np.ptp(V[used][:, 1])
    vs = np.unique(F[fs]); P = V[vs]; ctr = P.mean(0)
    _, s, vt = np.linalg.svd(P - ctr); ax = vt[0]; t = (P - ctr) @ ax
    length = float(np.ptp(t)); mid = (t.max() + t.min()) / 2
    tf = (V[F[fs]] - ctr) @ ax
    def sides(at): return int(((tf.min(1) < at) & (tf.max(1) > at)).sum())
    out = dict(faces=len(fs), length_h=length / h, sides_mid=sides(mid), sides_q1=sides(t.min() + 0.25 * length), sides_q3=sides(t.min() + 0.75 * length))
    width = float(2 * np.median(np.linalg.norm((P - ctr) - np.outer(t, ax), axis=1))); out['slender'] = length / max(width, 1e-9)
    ina = np.array([int(a) in fset and int(b) in fset for a, b in zip(c['e_a'], c['e_b'])])
    el = c['e_len'][ina]; ad = np.abs(c['e_dih'][ina]); tot = max(el.sum(), 1e-9)
    out['hard'] = float(el[ad >= 50].sum() / tot); out['firm'] = float(el[(ad >= 30) & (ad < 50)].sum() / tot); out['soft'] = float(el[ad < 30].sum() / tot)
    # junction: edges with one face in the horn and one outside
    jn = np.array([(int(a) in fset) != (int(b) in fset) for a, b in zip(c['e_a'], c['e_b'])])
    out['welded'] = bool(jn.any())
    if jn.any():
        out['join_fold'] = float((np.abs(c['e_dih'][jn]) * c['e_len'][jn]).sum() / c['e_len'][jn].sum())
        lit = c['LC'].mean(1)
        a, b = c['e_a'][jn], c['e_b'][jn]
        g = lambda i: cl.grey(cl.TABLE[(int(c['C'][i]) & 0xFF80) + int(round(lit[i]))])
        out['join_step_grey'] = float(np.mean([abs(g(x) - g(y)) for x, y in zip(a, b)]))
    else:
        opened = sum(1 for (a, b) in c['open'] if any(f in fset for f in c['vf'][a]))
        out['closed'] = opened == 0
    out['colours'] = len(set(c['C'][fs].tolist()))
    return out

def select(c, cols, band, side):
    cl.topology(c); V, F = c['V'], c['F']; skin = c['drawn'] & (c['A'] == 0); used = np.unique(F[skin]); top = V[used][:, 1].max(); h = np.ptp(V[used][:, 1])
    up = (top - c['cen'][:, 1]) / h
    if cols == 'shell':
        import collections
        groups = collections.defaultdict(list)
        for i in np.nonzero(skin)[0]: groups[c['shell'][int(i)]].append(int(i))
        cand = [fs for fs in groups.values() if 40 <= len(fs) <= 120 and band[0] <= up[fs].mean() <= band[1] and c['cen'][fs][:, 0].mean() * side > 10]
        cand.sort(key=lambda fs: -np.ptp(V[np.unique(F[fs])], axis=0).max())
        return cand[0]
    want = {(h_ << 10) | (s << 7) | l for h_, s, l in cols}
    sel = [int(i) for i in np.nonzero(skin)[0] if int(c['C'][i]) in want and band[0] <= up[i] <= band[1] and c['cen'][i][0] * side > 0]
    # keep the largest connected group
    sset = set(sel); adj = c['adj']; seen = set(); best = []
    for s0 in sel:
        if s0 in seen: continue
        st = [s0]; g = []
        while st:
            x = st.pop()
            if x in seen: continue
            seen.add(x); g.append(x); st.extend(o for o, _ in adj[x] if o in sset and o not in seen)
        if len(g) > len(best): best = g
    return best

if __name__ == '__main__':
    rows = {}
    for entry, name, cols, band, side in SETS:
        if cols is None: continue
        c = cl.load(entry); fs = select(c, cols, band, side); r = horn_stats(c, fs); rows[name] = r
        print(name, json.dumps({k: (round(v, 3) if isinstance(v, float) else v) for k, v in r.items()}))
    (pathlib.Path(__file__).resolve().parent / 'data' / 'horns.json').write_text(json.dumps(rows, indent=0))
