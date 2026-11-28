"""Decode every texture group in archives 52 (DDS) and 53 (PNG), report 54; write diffuse PNGs.

Diffuse = the first texture reference of a material record (layout 0 bit 0x01/0x10, layout 1 bit 0x20),
checked here by colour statistics per reference slot (normal maps average near (128,128,255) in 53).
Writes textures/diffuse/<id>.png (archive-53 PNG bytes, verified decodable) and textures.json.
The DDS copy carries a wrap border; its interior is compared with the PNG for every diffuse texture.
"""
import sys, pathlib, json, collections
from multiprocessing import Pool
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import numpy as np
from PIL import Image
from rs3flat import Cache, ROOT
import textures951 as tx

OUT = ROOT/'textures'/'diffuse'
_c = None
def job(args):
    t, diffuse = args
    global _c
    if _c is None: _c = Cache()
    r = dict(id=t)
    ims = {}
    for a in (52, 53):
        try:
            im, info = tx.image(_c, a, t)
            ims[a] = im
            r[a] = dict(size=im.size, mode=im.mode, fourcc=info.get('fourcc'), trailing=info['trailing'], count=info['extra'])
        except KeyError:
            r[a] = 'absent'
        except Exception as e:
            r[a] = 'error ' + repr(e)[:120]
    if 53 in ims:
        rgb = np.asarray(ims[53].convert('RGBA'), dtype=np.float32)
        r['mean'] = rgb.reshape(-1, 4).mean(0).round(1).tolist()
    if diffuse and 53 in ims:
        body, _, _ = tx.payload(_c, 53, t)
        (OUT/f'{t}.png').write_bytes(body)
        if 52 in ims:
            a = np.asarray(ims[52].convert('RGBA'), dtype=np.float32)
            b = np.asarray(ims[53].convert('RGBA'), dtype=np.float32)
            # DDS copies carry a 32-pixel wrap border; the interior may be at a different resolution.
            bd = 32 if min(a.shape[:2]) > 64 and min(a.shape[:2]) - 64 >= 16 else 0
            inner = ims[52].convert('RGBA').crop((bd, bd, a.shape[1]-bd, a.shape[0]-bd))
            r['border'] = bd; r['dds_size'] = list(ims[52].size); r['png_size'] = list(ims[53].size)
            inner = np.asarray(inner.resize(ims[53].size, Image.Resampling.BILINEAR), dtype=np.float32)
            r['dds_interior_rgb_mae'] = float(np.abs(inner[..., :3]-b[..., :3]).mean())
    return r

if __name__ == '__main__':
    OUT.mkdir(parents=True, exist_ok=True)
    c = Cache()
    mats = json.loads((ROOT/'materials.json').read_text())
    slot = {}
    diffuse = set()
    for k, row in mats.items():
        for i, (bit, tag, t) in enumerate(row.get('references', [])):
            slot.setdefault(t, set()).add((row['layout'], i, bit))
            if i == 0: diffuse.add(t)
    ids = sorted(set(c.references(52)) | set(c.references(53)))
    rows = []
    with Pool(9) as pool:
        for n, r in enumerate(pool.imap_unordered(job, [(t, t in diffuse) for t in ids], chunksize=32)):
            rows.append(r)
            if n % 5000 == 0: print(n, flush=True)
    rows.sort(key=lambda r: r['id'])
    summary = {}
    for a in (52, 53):
        cnt = collections.Counter('ok' if isinstance(r[a], dict) else r[a].split(' ')[0] for r in rows)
        modes = collections.Counter((r[a]['mode'], r[a].get('fourcc')) for r in rows if isinstance(r[a], dict))
        summary[a] = dict(status=dict(cnt), modes={str(k): v for k, v in modes.items()},
                          errors=[(r['id'], r[a]) for r in rows if isinstance(r[a], str) and r[a].startswith('error')][:20])
    present54 = c.present(54)
    ok54 = 0; err54 = []
    for t in present54[::50]:
        try: tx.image(c, 54, t); ok54 += 1
        except Exception as e: err54.append((t, repr(e)[:80]))
    summary[54] = dict(reference_groups=len(c.references(54)), present=len(present54), sampled_every_50th=len(present54[::50]),
                       sampled_decoded=ok54, errors=err54[:10])
    # slot statistics: blue-dominant mean marks a normal map
    by_slot = collections.defaultdict(list)
    for r in rows:
        if 'mean' in r:
            for s in slot.get(r['id'], ()):
                by_slot[s].append(r['mean'])
    slots = {f'layout{l} position{i} bit0x{b:x}': dict(textures=len(v), mean_rgba=np.mean(v, 0).round(1).tolist(),
             share_normal_like=round(float(np.mean([(m[2] > 200 and abs(m[0]-128) < 40 and abs(m[1]-128) < 40) for m in v])), 3))
             for (l, i, b), v in sorted(by_slot.items()) if len(v) >= 50}
    dif = [r for r in rows if r['id'] in diffuse]
    maes = [r['dds_interior_rgb_mae'] for r in dif if r.get('dds_interior_rgb_mae') is not None]
    summary['diffuse'] = dict(referenced=len(diffuse), written_png=sum(1 for r in dif if isinstance(r[53], dict)),
                              missing_in_53=[r['id'] for r in dif if not isinstance(r[53], dict)][:30],
                              dds_compared=len(maes), dds_interior_mae_median=float(np.median(maes)) if maes else None,
                              dds_interior_mae_p99=float(np.percentile(maes, 99)) if maes else None,
                              dds_interior_mae_max=float(max(maes)) if maes else None,
                              size_pairs=dict(collections.Counter(f"{r.get('dds_size')}->{r.get('png_size')}" for r in dif).most_common(12)),
                              image_counts=dict(collections.Counter(str(r[53].get('count')) for r in rows if isinstance(r[53], dict))))
    summary['slots'] = slots
    (ROOT/'textures.json').write_text(json.dumps(dict(summary=summary, rows=rows), separators=(',', ':')))
    print(json.dumps(summary, indent=1)[:6000])
