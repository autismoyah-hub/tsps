"""measure_items2.py: follow-up measurements for ITEMS.md (run after measure_items.py; prints tables).

a. When the inventory model is a "different mesh" from the male worn model, is it still the same design re-built
   and laid down? Compared: face counts, the three principal extents (sorted, so pose does not matter), the colour
   sets, and which axis of the inventory model is the thin one (how it lies).
b. How the inventory model lies, by kind of weapon and armour: which axis carries its length, where its box sits.
c. The icon view values real items use, by kind (zoom, xan, yan, zan, offsets), and how much of the 36 x 32 cell the
   drawing fills (from the client's icon, via research_items.exe icons).
d. Chat-head models: scale against the worn helm, labels, and the kits' head models.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections, statistics as st, subprocess, re
import numpy as np
import itemlib as L

IT = L.items()
M = json.loads((L.HERE / 'data' / 'measure-items.json').read_text())
med = lambda v: round(float(st.median(v)), 2) if len(v) else None
q = lambda v, p: round(float(np.percentile(v, p)), 2) if len(v) else None


def pca_extents(V):
    V = V.astype(float) - V.mean(0)
    _, _, Vt = np.linalg.svd(V, full_matrices=False)
    P = V @ Vt.T
    return sorted((P.max(0) - P.min(0)).tolist(), reverse=True)


print('a. "different mesh" inventory models against the male worn model')
print('slot     n     | same face count | faces inv/worn 10th..90th | principal extents inv/worn (long, mid, short) medians | same colour set | inv thin axis x/height/z')
for k, rs in M['inventory_vs_worn'].items():
    diff = [r for r in rs if r['kind'] == 'different mesh']
    if not diff: continue
    same_n = 0; ratios = []; ext = []; samecol = 0; thin = collections.Counter()
    for r in diff:
        d = IT[r['id']]
        inv = L.model(d['model']); worn = L.concat(L.worn_ids(d, 0))
        same_n += r['inv_faces'] == r['worn_faces']
        ratios.append(r['inv_faces'] / r['worn_faces'])
        a, b = pca_extents(inv['V']), pca_extents(worn['V'])
        ext.append([a[i] / max(b[i], 1) for i in range(3)])
        samecol += set(inv['colors']) == set(worn['colors'])
        thin['x height z'.split()[int(np.argmin(r['inv_extent']))]] += 1
    E = np.array(ext)
    print(f'{L.SLOT_NAMES.get(int(k), k):8s} {len(diff):5d} | {same_n * 100 / len(diff):5.1f}% | {q(ratios, 10)}..{q(ratios, 90)} | {[med(E[:, i]) for i in range(3)]} | {samecol * 100 / len(diff):5.1f}% | {dict(thin)}')

KINDS = [('longsword', r'longsword$'), ('scimitar', r'scimitar$'), ('2h sword', r'2h sword$|godsword$'), ('dagger', r'dagger$'), ('mace', r' mace$'),
         ('battleaxe', r'battleaxe$'), ('halberd', r'halberd$'), ('spear', r'spear$'), ('staff', r'staff$|^staff of'), ('bow', r'bow$'),
         ('platebody', r'platebody$'), ('chainbody', r'chainbody$'), ('platelegs', r'platelegs$'), ('plateskirt', r'plateskirt$'),
         ('full helm', r'full helm$'), ('med helm', r'med helm$'), ('helm (any)', r'helm$'), ('boots', r'boots$'), ('gloves/gauntlets', r'gloves$|gauntlets$|vambraces$'),
         ('kiteshield', r'kiteshield$'), ('cape', r'cape$|cloak$')]
byname = collections.defaultdict(list)
for d in IT.values():
    if not L.plain(d) or d['wear_positions'][0] == -1 or d['wear_models'][0][0] == -1: continue
    for kind, pat in KINDS:
        if re.search(pat, d['name'].lower()): byname[kind].append(d)
print('\nb/c. inventory model pose and icon view values by kind (medians; 10th..90th for zoom). Extents are x, height, z of the inventory model.')
print('kind              n  | extent x, h, z   | box centre x,h,z | zoom (10..90)        | xan  yan  zan | xoff yoff | most common (xan, yan, zan)')
kinds_out = {}
for kind, _ in KINDS:
    ds = byname[kind]
    if not ds: continue
    E = []; C = []
    for d in ds:
        m = L.model(d['model'])
        if m is None or not len(m['V']): continue
        V = m['V'] * (np.array(d['scale']) / 128.0)
        E.append(V.max(0) - V.min(0)); C.append((V.max(0) + V.min(0)) / 2)
    E = np.array(E); C = np.array(C)
    z = [d['zoom'] for d in ds]
    ang = collections.Counter(tuple(d['angles']) for d in ds).most_common(2)
    print(f'{kind:16s} {len(ds):4d} | {[med(E[:, i]) for i in range(3)]} | {[med(C[:, 0]), med(-C[:, 1]), med(C[:, 2])]} | {med(z)} ({q(z, 10)}..{q(z, 90)}) | '
          f'{med([d["angles"][0] for d in ds])} {med([d["angles"][1] for d in ds])} {med([d["angles"][2] for d in ds])} | {med([d["offset"][0] for d in ds])} {med([d["offset"][1] for d in ds])} | {ang}')
    kinds_out[kind] = [d['id'] for d in ds]
L.save_models()
(L.HERE / 'data' / 'kinds.json').write_text(json.dumps(kinds_out))

print('\nc2. how much of the 36 x 32 cell real icons fill, by kind (client icons, outline 1, shadow 302020): drawn pixels and box, medians')
for kind, _ in KINDS:
    ds = byname[kind][:120]
    if not ds: continue
    r = subprocess.run([L.RESEARCH_ITEMS, L.CACHE, 'icons', str(L.HERE / 'data' / '_tmp.bmp')] + [f'item:{d["id"]}' for d in ds], capture_output=True, text=True)
    rows = re.findall(r'item:(\d+): (\d+) pixels drawn, box x (\d+)\.\.(\d+) y (\d+)\.\.(\d+)', r.stdout)
    if not rows: print(kind, r.stderr[:200]); continue
    A = np.array(rows, dtype=float)
    print(f'{kind:16s} n={len(rows):3d} drawn {med(A[:, 1])} px; box x {med(A[:, 2])}..{med(A[:, 3])} y {med(A[:, 4])}..{med(A[:, 5])}; width {med(A[:, 3] - A[:, 2] + 1)} height {med(A[:, 5] - A[:, 4] + 1)}; '
          f'touches an edge {np.mean((A[:, 2] == 0) | (A[:, 3] == 35) | (A[:, 4] == 0) | (A[:, 5] == 31)) * 100:.0f}%')
(L.HERE / 'data' / '_tmp.bmp').unlink(missing_ok=True)

print('\nd. chat-head models (male) against the worn helm')
rows = [r for r in M['head_vs_worn'] if 'head_faces' in r]
sc = []; cen = []
for r in rows:
    d = IT[r['id']]
    h = L.concat(L.head_ids(d, 0)); w = L.concat(L.worn_ids(d, 0))
    eh = h['V'].max(0) - h['V'].min(0); ew = w['V'].max(0) - w['V'].min(0)
    sc.append(eh / np.maximum(ew, 1)); cen.append(((h['V'].max(0) + h['V'].min(0)) / 2).tolist() + ((w['V'].max(0) + w['V'].min(0)) / 2).tolist())
S = np.array(sc); Cn = np.array(cen)
print(f'head extent / worn extent (x, y, z): median {[med(S[:, i]) for i in range(3)]}, 10th {[q(S[:, i], 10) for i in range(3)]}, 90th {[q(S[:, i], 90) for i in range(3)]}')
print(f'head box centre median {[med(Cn[:, i]) for i in range(3)]}; worn box centre median {[med(Cn[:, i]) for i in range(3, 6)]}')
full = [r for r in rows if r['hides'] == [8, 11]]
print(f'helms that blank hair and jaw (n={len(full)}): head faces median {med([r["head_faces"] for r in full])}, worn faces median {med([r["worn_faces"] for r in full])}; '
      f'label sets {collections.Counter(tuple(r["head_labels"]) for r in full).most_common(4)}')
print('  priorities:', collections.Counter(tuple(r['head_priorities']) for r in full).most_common(5))
for mid, what in ((63, 'male hair kit 0 head model'), (81, 'jaw kit head model'), (120, 'female hair kit 45 head model')):
    m = L.model(mid)
    print(f'  model {mid} ({what}): {len(m["V"])} vertices {len(m["F"])} faces, box {m["V"].min(0).tolist()}..{m["V"].max(0).tolist()}, '
          f'labels {dict(sorted(collections.Counter(m["vertex_skins"]).items()))}, priorities {dict(collections.Counter(m["priorities"]))}')
L.save_models()
