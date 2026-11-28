"""Mesh numbers for the earlier study selections (pairs.json), beside the OSRS 239 controls. Writes study-mesh.json."""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import numpy as np
from rs3flat import ROOT
import rs3mesh

def numbers(ps):
    faces = verts = alpha_faces = 0
    welded = 0; mats = set(); tex = set(); bones = collections.Counter(); uv_faces = 0
    for m in ps:
        p = np.asarray(m['positions'], float)
        verts += len(p); welded += len(np.unique(p, axis=0))
        a = m.get('alpha', np.full(len(p), 255))
        for g in m['groups']:
            t = g['triangles']; faces += len(t)
            alpha_faces += int((a[t].min(1) < 255).sum())
            if g['material'] >= 0:
                mats.add(g['material'])
                row = rs3mesh.materials().get(str(g['material']), {})
                tex.update(r[2] for r in row.get('references', []))
        for b in m.get('bones', []): bones[len(b)] += 1
    return dict(faces=faces, vertices=verts, welded_positions=welded, materials=len(mats), textures=len(tex),
                alpha_faces=alpha_faces, max_bones=max(bones, default=0),
                multi_bone_vertex_share=round(sum(v for k, v in bones.items() if k > 1)/max(1, verts), 3),
                bone_histogram=dict(sorted(bones.items())))

if __name__ == '__main__':
    rows = json.loads((ROOT/'pairs.json').read_text())
    out = []
    for r in rows:
        kind = {'Creatures': 'npc', 'Weapons': 'item', 'Armour': 'item', 'Effects': 'model'}[r['category']]
        spec = f"{kind}:{r['id']}" if kind != 'model' else f"model:{r['models'][0]}"
        try:
            d, ps = rs3mesh.parts(spec)
            n = numbers(ps)
        except Exception as e:
            n = dict(error=repr(e))
        o = dict(zip(['faces', 'vertices', 'textures', 'alpha_faces', 'max_bones'], r['osrs']))
        out.append(dict(category=r['category'], id=r['id'], name=r['name'], models=r['models'], rs3=n,
                        control=r['control'], control_name=r['control_name'], osrs=o))
        print(r['name'], n, o, flush=True)
    # worn forms for the armour rows
    worn = {}
    for r in rows:
        if r['category'] == 'Armour':
            try:
                worn[r['id']] = numbers(rs3mesh.parts(f"item-worn:{r['id']}")[1])
            except Exception as e:
                worn[r['id']] = dict(error=repr(e))
    (ROOT/'study-mesh.json').write_text(json.dumps(dict(rows=out, armour_worn_male=worn), indent=1))
