"""Predict how the game lights an exported model, without rendering it.

A port of `normals` and `light_with_normals` in rust-client/src/model_light.rs (osrs-workspace f745d18),
integer arithmetic included. Scene objects are lit with
    ambient = 64 + definition ambient,  contrast = 768 + 5 x definition contrast,  direction (-50, -10, -50)
(scene_light.rs / scene_normal.rs). Pass the DEFINITION values, the same numbers you give lineup.py.

usage:
  predict_light.py MODEL.bin --ambient A --contrast C              table per face group (colour x facing)
  predict_light.py MODEL.bin --ambient A --contrast C --faces      one line per face
  predict_light.py MODEL.bin --ambient A --contrast C --picture OUT.png [--view 0..3] [--dist D --elev E --up U]
        a flat picture of the prediction from the line-up camera, 1280x900 like a capture eKvR.png
        (use the same --dist/--elev as the line-up; z-buffered, so draw order by priority is NOT imitated)

as a module:
  m = load(path)                      decoded model (dict from osrscache.decode_model)
  lit = light(m, ambient, contrast)   per face: [c0, c1, c2] lit 16-bit colours at the three corners (smooth),
                                      [c, c, c] (flat face), or None (face not drawn: alpha 255 etc.)
  lightness(c), gamecol.rgb(h, s, l)  to go on to lightness and screen RGB
  project(m, view, dist, elev, up)    screen x, y, depth of every vertex in a capture
  picture(m, lit, ...)                the flat picture and a face-index buffer (which face shows at each pixel)
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, math, pathlib, argparse
sys.path.insert(0, str(pathlib.Path(__file__).parent))
sys.path.insert(1, _AP.fix(r'<ROOT>\tools'))
import numpy as np
import gamecol

DIRECTION = (-50, -10, -50)
YAWS = [0, 2048 * 3, 2048 * 5, 8192]      # lineup.py's views
W, H, FOCAL = 1280, 900, 1000.0            # the snapshot's frame and focal length


def tdiv(a, b):
    """Integer division that rounds toward zero, as Rust's `/` and wrapping_div do."""
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b > 0) else -q


def load(path):
    import osrscache
    return osrscache.decode_model(pathlib.Path(path).read_bytes())


def normals(m):
    """model_light.rs `normals`: per vertex (sum of 256-length face normals, count) over SMOOTH faces only;
    per flat face its own 256-length normal."""
    V = m['vertices']
    vert = [[0, 0, 0, 0] for _ in V]
    face = [(0, 0, 0)] * len(m['faces'])
    for i, (ia, ib, ic) in enumerate(m['faces']):
        a, b, c = V[ia], V[ib], V[ic]
        d = [b[k] - a[k] for k in range(3)]
        e = [c[k] - a[k] for k in range(3)]
        n = [d[1] * e[2] - e[1] * d[2], d[2] * e[0] - e[2] * d[0], d[0] * e[1] - e[0] * d[1]]
        while any(not -8192 <= v <= 8192 for v in n):
            n = [v >> 1 for v in n]                       # arithmetic shift, as in the source
        length = max(int(math.sqrt(sum(v * v for v in n))), 1)
        n = [tdiv(v * 256, length) for v in n]
        kind = m['render_types'][i]
        if kind == 0:
            for v in (ia, ib, ic):
                for k in range(3):
                    vert[v][k] += n[k]
                vert[v][3] += 1
        elif kind == 1:
            face[i] = tuple(n)
    return vert, face


def light(m, ambient=0, contrast=0, direction=DIRECTION, raw=False):
    """model_light.rs `light_with_normals` for a scene object standing alone (no normals merged with neighbours).
    ambient, contrast: the definition's values. raw=True takes the engine's own values instead (64+a, 768+5c)."""
    amb, con = (ambient, contrast) if raw else (64 + ambient, 768 + 5 * contrast)
    vert, face = normals(m)
    length = int(math.sqrt(sum(v * v for v in direction)))            # 71
    den = (con * length) >> 8                                          # 213 at contrast 0
    dot = lambda n: sum(n[k] * direction[k] for k in range(3))
    out = []
    for i, f in enumerate(m['faces']):
        kind = m['render_types'][i]
        alpha = m['alpha'][i]
        kind = 3 if alpha == 254 else 2 if alpha == 255 else kind
        col = m['colors'][i]
        textured = m['face_textures'][i] != -1
        if textured:
            shade = lambda b: min(max(b, 2), 126)
        else:
            shade = lambda b: (col & 65408) + min(max(((col & 127) * b) >> 7, 2), 126)
        if kind == 0:
            row = []
            for v in f:
                n = vert[v]
                row.append(shade(amb + tdiv(dot(n), den * n[3])))
            out.append(row)
        elif kind == 1:
            c = shade(amb + tdiv(dot(face[i]), den + tdiv(den, 2)))
            out.append([c, c, c])
        else:
            out.append(None)
    return out


def lightness(c):
    return c & 127


def facing(m, i):
    """Rough compass word for a face, y up: for grouping the table only."""
    V = m['vertices']; a, b, c = (V[k] for k in m['faces'][i])
    d = [b[k] - a[k] for k in range(3)]; e = [c[k] - a[k] for k in range(3)]
    n = (d[1] * e[2] - e[1] * d[2], -(d[2] * e[0] - e[2] * d[0]), d[0] * e[1] - e[0] * d[1])    # y flipped to up
    k = max(range(3), key=lambda j: abs(n[j]))
    return (('+x', '-x'), ('up', 'down'), ('+z', '-z'))[k][n[k] < 0]


def area(m, i):
    V = m['vertices']; a, b, c = (V[k] for k in m['faces'][i])
    d = [b[k] - a[k] for k in range(3)]; e = [c[k] - a[k] for k in range(3)]
    n = (d[1] * e[2] - e[1] * d[2], d[2] * e[0] - e[2] * d[0], d[0] * e[1] - e[0] * d[1])
    return math.sqrt(sum(v * v for v in n)) / 2


def project(m, view=0, dist=330.0, elev=250.0, up=45.0):
    """Screen position of every vertex in lineup.py's capture number `view`: arrays x, y (pixels in the
    1280x900 capture) and depth. From examples/authored_scene_snapshot.rs (Pose::camera) and render3d.rs."""
    yaw = YAWS[view] * math.tau / 16384
    P = np.array(m['vertices'], dtype=float)                 # model origin = tile centre on the floor, y down
    target = np.array([0.0, -up, 0.0])
    pos = target + np.array([math.sin(yaw) * dist, -elev, -math.cos(yaw) * dist])
    fwd = target - pos; fwd /= np.linalg.norm(fwd)
    right = np.cross(fwd, [0.0, -1.0, 0.0]); right /= np.linalg.norm(right)
    upv = np.cross(right, fwd)
    d = P - pos
    z = d @ fwd
    return W // 2 + (d @ right) * FOCAL / z, H // 2 - (d @ upv) * FOCAL / z, z


def picture(m, lit, view=0, dist=330.0, elev=250.0, up=45.0):
    """Flat picture of the prediction as the capture would show it. Returns (rgb [H, W, 3] uint8,
    face [H, W] int: index of the face showing at each pixel, -1 = none).
    Smooth faces interpolate the lit lightness between corners in screen space, as the game's rasteriser does.
    Hidden surfaces by z-buffer; the game sorts by priority and depth instead, so overlaps can differ."""
    x, y, z = project(m, view, dist, elev, up)
    pal = gamecol.table()
    img = np.zeros((H, W, 3), dtype=np.uint8)
    who = np.full((H, W), -1, dtype=np.int32)
    zb = np.full((H, W), np.inf)
    for i, (a, b, c) in enumerate(m['faces']):
        if lit[i] is None:
            continue
        x0, y0, x1, y1, x2, y2 = x[a], y[a], x[b], y[b], x[c], y[c]
        det = (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0)
        if det >= 0:                                           # back face (found by test against captures)
            continue
        # legacy_raster.rs `gouraud` + `spans`: whole-pixel corners, 14-bit edge slopes, 8-bit colour steps
        v = [(int(x0), int(y0)), (int(x1), int(y1)), (int(x2), int(y2))]
        col = lit[i]
        dx1, dy1, dx2, dy2 = v[1][0] - v[0][0], v[1][1] - v[0][1], v[2][0] - v[0][0], v[2][1] - v[0][1]
        idet = dx1 * dy2 - dx2 * dy1
        if idet == 0:
            continue
        dc1, dc2 = col[1] - col[0], col[2] - col[0]
        cx = tdiv((dc1 * dy2 - dc2 * dy1) << 8, idet)
        cy = tdiv((dc2 * dx1 - dc1 * dx2) << 8, idet)
        top = min(range(3), key=lambda k: v[k][1])
        initial = (col[top] << 8) - cx * v[top][0] + cx
        A, B, C = sorted(v, key=lambda p: p[1])
        slope = lambda p, q: 0 if q[1] == p[1] else tdiv((q[0] - p[0]) << 14, q[1] - p[1])
        ac, ab, bc = slope(A, C), slope(A, B), slope(B, C)
        for row in range(max(A[1], 0), min(C[1], H)):
            long = (A[0] << 14) + ac * (row - A[1])
            short = (A[0] << 14) + ab * (row - A[1]) if row < B[1] else (B[0] << 14) + bc * (row - B[1])
            s, e = max(min(long, short) >> 14, 0), min(max(long, short) >> 14, W)
            if s >= e:
                continue
            px = np.arange(s, e)
            w1 = ((px + 0.5 - x0) * (y2 - y0) - (x2 - x0) * (row + 0.5 - y0)) / det
            w2 = ((x1 - x0) * (row + 0.5 - y0) - (px + 0.5 - x0) * (y1 - y0)) / det
            depth = (1 - w1 - w2) * z[a] + w1 * z[b] + w2 * z[c]
            ok = depth < zb[row, s:e]
            if m['render_types'][i] == 1:
                colour = np.full(e - s, col[0])
            else:
                sample = s + (px - s) // 4 * 4                 # render3d.rs coarse shading: one colour per 4 pixels of a span
                colour = np.maximum(initial + cy * (row - v[top][1]) + cx * sample, 0) >> 8
            zb[row, s:e][ok] = depth[ok]
            img[row, s:e][ok] = pal[np.clip(colour[ok], 0, 65535)]
            who[row, s:e][ok] = i
    return img, who


def table(m, lit, per_face=False):
    """Print the prediction: per face, or grouped by painted colour, flat flag and facing."""
    rows = {}
    for i, f in enumerate(m['faces']):
        if lit[i] is None:
            continue
        h, s, l = gamecol.split(m['colors'][i])
        ls = [lightness(c) for c in lit[i]]
        flat = m['render_types'][i] == 1
        if per_face:
            V = m['vertices']
            cen = [round(sum(V[v][k] for v in f) / 3) for k in range(3)]
            print(f'face {i:4d}  paint {h:2d}/{s}/{l:3d} {"flat  " if flat else "smooth"} {facing(m, i):>4}  '
                  f'centre x {cen[0]:5d} y {-cen[1]:5d} z {cen[2]:5d}  area {area(m, i):7.0f}  lit {ls[0]:3d} {ls[1]:3d} {ls[2]:3d}')
            continue
        r = rows.setdefault((h, s, l, flat, facing(m, i)), [0, 0.0, 0.0, 127, 0])
        A = area(m, i)
        r[0] += 1; r[1] += A; r[2] += A * sum(ls) / 3; r[3] = min(r[3], *ls); r[4] = max(r[4], *ls)
    if per_face:
        return
    total = sum(r[1] for r in rows.values()) or 1
    print('paint h/s/l   shading facing faces  share   lit lightness: mean (min..max)   screen RGB at the mean')
    for (h, s, l, flat, face), r in sorted(rows.items(), key=lambda kv: -kv[1][1]):
        mean = r[2] / r[1] if r[1] else 0
        print(f'{h:3d}/{s}/{l:<3d}    {"flat  " if flat else "smooth"}  {face:>5} {r[0]:6d} {100 * r[1] / total:5.1f}%   '
              f'{mean:5.1f} ({r[3]:3d}..{r[4]:3d})                   {gamecol.rgb(h, s, round(mean))}')


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('model')
    ap.add_argument('--ambient', type=int, default=10, help="definition ambient (lineup.py's default is 10)")
    ap.add_argument('--contrast', type=int, default=30, help="definition contrast (lineup.py's default is 30)")
    ap.add_argument('--faces', action='store_true'); ap.add_argument('--picture')
    ap.add_argument('--view', type=int, default=0)
    ap.add_argument('--dist', type=float, default=330); ap.add_argument('--elev', type=float, default=250)
    ap.add_argument('--up', type=float, default=45)
    a = ap.parse_args()
    m = load(a.model)
    lit = light(m, a.ambient, a.contrast)
    print(f'{a.model}: {len(m["vertices"])} vertices, {len(m["faces"])} faces; light = {64 + a.ambient} + n.L / '
          f'{((768 + 5 * a.contrast) * 71) >> 8} (flat faces: / {(((768 + 5 * a.contrast) * 71) >> 8) * 3 // 2}); lit = paint x light >> 7, kept in 2..126')
    if a.picture:
        from PIL import Image
        img, who = picture(m, lit, a.view, a.dist, a.elev, a.up)
        img[who < 0] = (74, 72, 67)                            # the line-up's ground tone, for easy comparison
        Image.fromarray(img).save(a.picture)
        print(a.picture, 'view', a.view)
    else:
        table(m, lit, a.faces)
