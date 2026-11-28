"""Step 2: how much of the real boss's surface and paint boundaries does each sheet resolve? (Teacher-side.)
usage: sheet_measure.py NPC_ID [NPC_ID..]
Configurations:
  first sheet     six axis views, 1.25 px per unit (the creature sheet's rule gave 1.25 on both bosses), no winding cull,
                  translucent faces drawn as if opaque (that is how sheet_npc.py draws)
  v2 ortho        eighteen views at the v2 resolution, cull, opaque layer and translucent layer apart
  v2 ortho+pose   plus the pose set (harness pictures, rest pose and 10 frames, 8 directions)
A face is SEEN with >= 1 pixel in some picture, RESOLVED with >= 6 (a patch a reader can place). A paint boundary edge is
resolved when BOTH faces across it are resolved in the SAME picture.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections
import numpy as np
import tl, ortho, poseset
from tl import L
from decomp import islands
def run(npc):
    m, kv = tl.real_mesh(npc); a = tl.arrays(m); d = tl.real_def(npc); dr = tl.drawn(a); F, V, C = a['F'], a['V'], a['C']; nf = len(F)
    used = np.unique(F[dr]); lo, hi = V[used].min(0), V[used].max(0)
    area = np.linalg.norm(np.cross(V[F[:, 1]] - V[F[:, 0]], V[F[:, 2]] - V[F[:, 0]]), axis=1) / 2; tot = area[dr].sum()
    isl, nisl, _ = islands(a)
    edges = collections.defaultdict(list)
    for i in np.nonzero(dr)[0]:
        f = F[i]
        for x, y in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(x, y), max(x, y))].append(i)
    bnd = [(k, v) for k, v in edges.items() if len(v) == 2 and C[v[0]] != C[v[1]]]; blen = np.array([np.linalg.norm(V[k[0]] - V[k[1]]) for k, v in bnd]); bf = np.array([v for k, v in bnd])
    drawn_idx = np.nonzero(dr)[0]
    def counts_ortho(views, px, cull, layered):
        per = []
        for name, c in views:
            fr = ortho.frame(lo, hi, c, px)
            if layered:
                op = np.nonzero(dr & (a['A'] == 0))[0]; tr = np.nonzero(dr & (a['A'] > 0))[0]
                ido, _, zo = ortho.raster(V, F, c, fr, px, op, None, cull); cnt = np.bincount(ido[ido >= 0], minlength=nf)
                if len(tr): idt, _, zt = ortho.raster(V, F, c, fr, px, tr, None, cull); idt[zt <= zo] = -1; cnt = cnt + np.bincount(idt[idt >= 0], minlength=nf)
            else:
                ido, _, _ = ortho.raster(V, F, c, fr, px, drawn_idx, None, cull); cnt = np.bincount(ido[ido >= 0], minlength=nf)
            per.append(cnt)
        return per
    def counts_pose():
        sheet = tl.B / 'sheets' / {7541: 'tekton', 11278: 'nex'}[npc]; info = json.loads((sheet / 'pose.json').read_text()); seqs = info['sequences']
        cam = dict(zoom=info['zoom'], aim=info['aim'], cell=tuple(info['cell'])); var = poseset.layer_variants(f'real-{npc}', m, d, 'ids'); ents = [tl.entry(p, seqs) for p in var]; per = []
        for yaw, pitch, rws in info['cameras']:
            for cs in poseset.draw_layers(ents, rws, cam, yaw, pitch):
                fA, fO, ln = poseset.decode(cs, nf); per.append(np.bincount(fO[fO >= 0], minlength=nf) + np.bincount(fA[(fA >= 0) & (fA != fO)], minlength=nf))
        return per
    def stats(per, tag, px_note):
        P = np.array(per); best = P.max(0); seen = (best >= 1) & dr; res = (best >= 6) & dr
        both = ((P[:, bf[:, 0]] >= 6) & (P[:, bf[:, 1]] >= 6)).any(0)
        ic = [best[isl & dr]];
        print(f"  {tag:15s} {px_note:22s} faces seen {seen.sum() / dr.sum():.3f}  resolved {res.sum() / dr.sum():.3f} | surface seen {area[seen].sum() / tot:.3f}  resolved {area[res].sum() / tot:.3f} | "
              f"paint-boundary length resolved {blen[both].sum() / blen.sum():.3f} | island faces resolved {(res & isl).sum() / max((isl & dr).sum(), 1):.3f} | median pixels of a face in its best picture {np.median(best[dr]):.0f}", flush=True)
        return dict(faces_seen=float(seen.sum() / dr.sum()), faces_resolved=float(res.sum() / dr.sum()), surface_seen=float(area[seen].sum() / tot), surface_resolved=float(area[res].sum() / tot),
                    boundary_resolved=float(blen[both].sum() / blen.sum()), island_faces_resolved=float((res & isl).sum() / max((isl & dr).sum(), 1)), median_px=float(np.median(best[dr])))
    v2px = json.loads((tl.B / 'sheets' / {7541: 'tekton', 11278: 'nex'}[npc] / 'sheet.json').read_text())['pixels_per_unit']
    print(f"NPC {npc} {d['name']}: {int(dr.sum())} drawn faces, {len(bnd)} paint-boundary edges, {nisl} small contrasting islands ({int((isl & dr).sum())} faces); median face area {np.median(area[dr]):.0f} square units (edge about {np.sqrt(2 * np.median(area[dr])):.0f} units); "
          f"island faces median area {np.median(area[isl & dr]):.0f}; translucent faces {int((dr & (a['A'] > 0)).sum())}")
    out = {}
    old = counts_ortho(ortho.SIX, 1.25, False, False); out['first'] = stats(old, 'first sheet', '6 views, 1.25 px/unit')
    out['first_culled'] = stats(counts_ortho(ortho.SIX, 1.25, True, True), 'first, as drawn', '6 views, cull, layers')
    six = counts_ortho(ortho.SIX, v2px, True, True); out['six_hi'] = stats(six, 'six views, v2 px', f'6 views, {v2px:.2f} px/unit')
    new = counts_ortho(ortho.VIEWS, v2px, True, True); out['v2_ortho'] = stats(new, 'v2 ortho', f'18 views, {v2px:.2f} px/unit')
    pose = counts_pose(); out['pose'] = stats(pose, 'pose set alone', '96 harness cells')
    out['v2'] = stats(new + pose, 'v2 ortho+pose', '')
    # the gate's own pictures, for scale
    seqs = L.sequences(npc); cam = tl.camera(npc); var = poseset.layer_variants(f'real-{npc}', m, d, 'ids'); per = []
    for yaw in (192, 1216):
        for cs in poseset.draw_layers([tl.entry(p, seqs) for p in var], L.sample_rows(seqs, 6), cam, yaw, 128):
            fA, fO, ln = poseset.decode(cs, nf); per.append(np.bincount(fA[fA >= 0], minlength=nf))
    out['gate'] = stats(per, 'the gate itself', '50 cells, 380 px')
    (tl.WORK / 'sheet-measure').mkdir(exist_ok=True); (tl.WORK / 'sheet-measure' / f'{npc}.json').write_text(json.dumps(out, indent=1))
if __name__ == '__main__':
    for n in sys.argv[1:]: run(int(n))
