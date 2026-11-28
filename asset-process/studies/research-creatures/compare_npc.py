"""Score a creature rebuild against the real NPC it copies: at rest AND in motion. The Teacher runs this; the author
of a held-out rebuild does not (it reads the target's mesh).

usage: compare_npc.py REAL_NPC_ID OURS.json [--out DIR] [--per N] [--attack SEQ] [--death SEQ] [--json FILE] [--quiet]

At rest (both meshes as the client assembles them, after recolour and scale): size, vertices, faces, colours, colour
shares and colour distance, welding, flat / depth-offset / translucent faces, outline overlap in three orthographic
views in BODY space, the label sets and how far each label's centroid (the pivot it may define) sits from the real one.
In motion: both are drawn by the NPC harness under the SAME sequences (the real creature's stand and walk from its
definition; attack and death from the server tables or rebuild_lib.KNOWN_SEQS or the options), N frames of each
(default 6, spread over the sequence), from the front quarter (yaw 192) and the rear quarter (yaw 1216), and each
pair of cells is scored: outline overlap, share of pixels that differ, mean grey difference where both are drawn.
--out DIR keeps the two grids (real | ours) and a magenta difference strip.
"""
import sys, pathlib, json, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
from PIL import Image
import rebuild_lib as L, cmesh

def compare(real_id, ours_json, per=6, overrides=None, out=None):
    real_id = int(real_id); ours_json = pathlib.Path(ours_json).resolve()
    R = {}
    lo_r, hi_r, mr, scr = L.size_after_scale(real_id); lo_o, hi_o, mo, sco = L.size_after_scale(ours_json)
    d = L.npc_def(real_id); oj = json.loads(ours_json.read_text())
    R['name'] = d['name']
    R['size_real'] = [float(round(x)) for x in hi_r - lo_r]; R['size_ours'] = [float(round(x)) for x in hi_o - lo_o]
    R['box_gap'] = float(max(np.abs(lo_r - lo_o).max(), np.abs(hi_r - hi_o).max()))      # worst face of the bounding box, units
    def counts(m):
        keep = (m['A'] != 255) & (m['T'] < 2); used = np.unique(m['F'])
        pos = collections.Counter(map(tuple, m['V'][used].astype(int).tolist()))
        return dict(vertices=int(len(used)), faces=int(keep.sum()), flat=int((m['T'][keep] & 1).sum()), depth_offset=int((m['D'][keep] != 0).sum()),
                    translucent=int((m['A'][keep] > 0).sum()), undrawn=int((~keep).sum()), split=int(sum(n for n in pos.values() if n > 1)))
    R['real'], R['ours'] = counts(mr), counts(mo)
    cr, co = L.colour_shares(mr), L.colour_shares(mo)
    R['real']['colours'], R['ours']['colours'] = len(cr), len(co)
    R['colour_distance'] = L.colour_distance(co, cr)
    R['palette_real'] = sorted(((k, round(v, 4)) for k, v in cr.items()), key=lambda kv: -kv[1])[:12]
    R['palette_ours'] = sorted(((k, round(v, 4)) for k, v in co.items()), key=lambda kv: -kv[1])[:12]
    R['definition'] = dict(real=dict(scale=d['scale'], ambient=d['ambient'], contrast=d['contrast'] // 5), ours=dict(scale=oj.get('scale', [128, 128]), ambient=oj.get('ambient', 0), contrast=oj.get('contrast', 0)))
    R['outline'] = L.rest_outlines(mr, scr, mo, sco)
    # labels
    def labs(m):
        used = np.unique(m['F']); by = collections.defaultdict(list)
        for v, l in zip(m['V'][used], m['L'][used]): by[int(l)].append(v)
        return {l: np.mean(vs, 0) for l, vs in by.items()}, {l: len(vs) for l, vs in by.items()}
    (pr, nr), (po, no) = labs(mr), labs(mo)
    R['labels_real'], R['labels_ours'] = len(pr), len(po)
    R['labels_not_in_real'] = sorted(set(po) - set(pr)); R['labels_missing'] = sorted(set(pr) - set(po))
    gaps = {l: float(np.linalg.norm(pr[l] - po[l])) for l in pr if l in po}
    R['pivot_gap_median'] = float(np.median(list(gaps.values()))) if gaps else None
    R['pivot_gap_p90'] = float(np.quantile(list(gaps.values()), .9)) if gaps else None
    R['pivot_gap_worst'] = max(gaps.items(), key=lambda kv: kv[1]) if gaps else None
    # style measures of both, for the record (not gated)
    sr, so = cmesh.measure(mr, tuple(scr)), cmesh.measure(mo, tuple(sco))
    R['style'] = {k: (sr[k], so[k]) for k in ('tones_per_hue', 'tone_by_part', 'tone_follows_up', 'tone_step_med', 'valence_6_share', 'valence_sd', 'fold_med_deg',
                                              'detail_islands', 'colour_edge_share', 'bridge_face_share', 'shells', 'faces_for_half_surface') if k in sr and k in so}
    # motion
    seqs = L.sequences(real_id, overrides); rows = L.sample_rows(seqs, per)
    R['sequences'] = seqs; R['rows'] = rows
    work = L.HERE / 'data' / 'defs'
    entries = [L.entry_real(real_id, seqs), L.entry_ours(ours_json, seqs, work)]
    D = float((hi_r - lo_r).max()); cell = (380, 380); zoom = int(512 * D * 1.7 / cell[0]); aim = int((hi_r[1] - lo_r[1]) / 2)
    R['camera'] = dict(zoom=zoom, pitch=128, cell=cell, aim=aim, yaws=[192, 1216])
    per_cell = []
    for yaw in (192, 1216):
        png = (pathlib.Path(out) / f'grid-yaw{yaw}.png') if out else None
        if png: png.parent.mkdir(parents=True, exist_ok=True)
        a, text = L.draw(entries, rows, zoom, yaw, 128, cell, aim, png)
        bad = [l for l in text.splitlines() if 'NOT APPLIED' in l or 'NO `' in l or 'WARNING' in l]
        if bad: R.setdefault('harness_notes', []).extend(bad[:6])
        grid = L.cells(a, 2, len(rows))
        strip = []
        for row, (A, B) in zip(rows, grid):
            m = L.cell_metrics(A, B); m.update(row=row, yaw=yaw); per_cell.append(m)
            if out:
                dm = A.copy(); dm[(A != B).any(2)] = (255, 0, 255); strip.append(np.concatenate([A, B, dm], 1))
        if out: Image.fromarray(np.concatenate(strip, 0)).save(pathlib.Path(out) / f'diff-yaw{yaw}.png')
    R['cells'] = per_cell
    agg = {}
    for name in ['bind'] + [k for k in ('stand', 'walk', 'attack', 'death') if k in seqs]:
        cs = [c for c in per_cell if c['row'].split(':')[0] == name]
        if not cs: continue
        agg[name] = dict(frames=len(cs), overlap_mean=float(np.mean([c['overlap'] for c in cs])), overlap_worst=float(min(c['overlap'] for c in cs)),
                         differ_mean=float(np.mean([c['differ'] for c in cs])), tone_mean=float(np.mean([c['tone'] for c in cs])), tone_worst=float(max(c['tone'] for c in cs)),
                         all_mean=float(np.mean([c['all'] for c in cs])), worst_cell=min(cs, key=lambda c: c['overlap'])['row'])
    R['motion'] = agg
    return R

def show(R):
    P = print
    P(f"real NPC: {R['name']}   sequences {R['sequences']}")
    P(f"size x, height, z (after scale)   real {R['size_real']}   ours {R['size_ours']}   worst box face gap {R['box_gap']:.1f}")
    for k in ('vertices', 'faces', 'colours', 'flat', 'depth_offset', 'translucent', 'undrawn', 'split'):
        P(f"{k:32s} real {R['real'][k]}   ours {R['ours'][k]}")
    P(f"definition                       real {R['definition']['real']}   ours {R['definition']['ours']}")
    P('colour shares (h,s,l: % of surface)')
    P('  real: ' + ', '.join(f'{tuple(k)}:{100 * v:.0f}' for k, v in R['palette_real'][:10]))
    P('  ours: ' + ', '.join(f'{tuple(k)}:{100 * v:.0f}' for k, v in R['palette_ours'][:10]))
    P(f"colour distance (0 = identical; about 1 = one lightness step of 4 over the whole surface): {R['colour_distance']:.2f}")
    P('rest outline overlap in body space (1 = identical): ' + ', '.join(f'{k} {v:.3f}' for k, v in R['outline'].items()))
    P(f"labels: real {R['labels_real']}, ours {R['labels_ours']}; ours uses labels the real does not: {R['labels_not_in_real']}; real labels ours lacks: {R['labels_missing']}")
    if R['pivot_gap_median'] is not None:
        P(f"label centroid gap (units): median {R['pivot_gap_median']:.1f}, 90th centile {R['pivot_gap_p90']:.1f}, worst label {R['pivot_gap_worst'][0]} at {R['pivot_gap_worst'][1]:.1f}")
    P('motion (front and rear quarter views; overlap = outline IoU; differ = share of union pixels that differ; tone = mean grey difference of 255 where both drawn)')
    for name, a in R['motion'].items():
        P(f"  {name:7s} {a['frames']:2d} cells  overlap mean {a['overlap_mean']:.3f} worst {a['overlap_worst']:.3f} ({a['worst_cell']})  differ {a['differ_mean']:.3f}  tone mean {a['tone_mean']:.2f} worst {a['tone_worst']:.2f}")
    for n in R.get('harness_notes', []): P('  harness: ' + n)
    P('style measures, real | ours (not gated): ' + ', '.join(f'{k} {a:.2f} | {b:.2f}' for k, (a, b) in R['style'].items()))

if __name__ == '__main__':
    a = sys.argv
    def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
    ov = {k: int(opt('--' + k)) for k in ('attack', 'death', 'stand', 'walk') if opt('--' + k)}
    R = compare(a[1], a[2], int(opt('--per', 6)), ov, opt('--out'))
    if '--quiet' not in a: show(R)
    if opt('--json'): pathlib.Path(opt('--json')).write_text(json.dumps(R, indent=1, default=lambda o: o.tolist() if hasattr(o, 'tolist') else str(o)))
