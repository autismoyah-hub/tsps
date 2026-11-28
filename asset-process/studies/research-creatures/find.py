"""Find NPC definitions by name with the facts used for dating: model ids, model file format, faces.

usage: find.py NAME [NAME...] [--exact] [--all]
Prints one line per distinct model list: npc id, name, combat, size, model ids, format of each model file
(L legacy, X extended ff ff, S skinned-legacy ff fe, T skinned-extended ff fd), faces, skeleton, labels, weights.
Model id and file format are MEASUREMENTS from the cache; the release year of a creature is the author's KNOWLEDGE.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib
sys.path.insert(0, _AP.fix(r'<ROOT>\tools'))
import osrscache as o
D = pathlib.Path(_AP.fix(r'<ROOT>\studies\research-npc\census'))
def load(n): return [json.loads(l) for l in (D / n).read_text().splitlines() if l.strip()]
npcs = load('npcs.jsonl'); seqs = {s['id']: s for s in load('sequences.jsonl')}; models = {m['id']: m for m in load('models.jsonl')}
_fmt = {}
def fmt(mid):
    if mid not in _fmt:
        try:
            d = o.unpack(o.group(7, mid)); _fmt[mid] = {b'\xff\xfd': 'T', b'\xff\xfe': 'S', b'\xff\xff': 'X'}.get(d[-2:], 'L')
        except Exception: _fmt[mid] = '?'
    return _fmt[mid]
def skel(n):
    for slot in (0, 3):
        s = seqs.get(n['anims'][slot])
        if s: return s['skeleton'], s['kind']
    return -1, 'none'
def line(n):
    ms = [m for m in n['models'] if m in models]
    f = sum(models[m]['faces'] for m in ms); lab = sorted({l for m in ms for l in models[m]['labels']})
    w = any(models[m]['weights'] for m in ms); sk, kind = skel(n)
    return (f"{n['id']:6d} {n['name'][:28]:28s} cb {n['combat']:4d} size {n['size']} models {n['models']} fmt {''.join(fmt(m) for m in ms)} "
            f"faces {f} skel {sk} {kind} labels {len(lab)} weights {w} recol {len(n['recolor'])} amb {n['ambient']} con {n['contrast'] // 5} scale {n['scale']}")
if __name__ == '__main__':
    a = sys.argv[1:]; exact = '--exact' in a; allv = '--all' in a
    for q in [x for x in a if not x.startswith('--')]:
        seen = set()
        for n in npcs:
            nm = n['name'].lower()
            if not n['models'] or not (nm == q.lower() if exact else q.lower() in nm): continue
            if tuple(n['models']) in seen and not allv: continue
            seen.add(tuple(n['models'])); print(line(n))
