"""Orthographic views with a depth buffer that keep, per pixel, WHICH face is in front and where on it.

Used by the boss sheet (on the real mesh, Teacher-side) and by the author-side tools (on the author's own mesh):
the same function with the same view table, so a sheet pixel and a pixel of the author's mesh mean the same place.

Coordinates are the cache's: x left-right, y DOWN, z along the body (head toward -z).
A view is (name, c): c = unit vector from the model toward the viewer. Screen axes: right = f x up, page-down = -up,
with f = -c and up = world up (0,-1,0) made square to f (for the two vertical views up = +z for TOP, -z for BOTTOM).
No view is mirrored. pixel = (dot(p, right) - origin_x) * px, (dot(p, down) - origin_y) * px.
"""
import numpy as np
S2, S3 = 2 ** -0.5, 3 ** -0.5
VIEWS = [('front', (0, 0, -1)), ('back', (0, 0, 1)), ('left', (-1, 0, 0)), ('right', (1, 0, 0)), ('top', (0, -1, 0)), ('bottom', (0, 1, 0)),
         ('front-left', (-S2, 0, -S2)), ('front-right', (S2, 0, -S2)), ('back-left', (-S2, 0, S2)), ('back-right', (S2, 0, S2)),
         ('up-front-left', (-S3, -S3, -S3)), ('up-front-right', (S3, -S3, -S3)), ('up-back-left', (-S3, -S3, S3)), ('up-back-right', (S3, -S3, S3)),
         ('low-front-left', (-S3, S3, -S3)), ('low-front-right', (S3, S3, -S3)), ('low-back-left', (-S3, S3, S3)), ('low-back-right', (S3, S3, S3))]
SIX = VIEWS[:6]
def basis(c):
    c = np.array(c, float); c /= np.linalg.norm(c); f = -c
    up = np.array([0.0, -1.0, 0.0])
    if abs(c[1]) > 0.999: up = np.array([0.0, 0.0, 1.0]) if c[1] < 0 else np.array([0.0, 0.0, -1.0])
    right = np.cross(f, up); right /= np.linalg.norm(right); up = np.cross(right, f)
    return right, -up, c                      # right, page-down, toward the viewer
def frame(lo, hi, c, px, pad=8):
    """Picture frame of the bounding box lo..hi in view c: (origin_x, origin_y, width, height)."""
    r, d, t = basis(c); corners = np.array([[x, y, z] for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])], float)
    X, Y = corners @ r, corners @ d
    return float(X.min()) - pad / px, float(Y.min()) - pad / px, int(np.ceil((X.max() - X.min()) * px)) + 2 * pad, int(np.ceil((Y.max() - Y.min()) * px)) + 2 * pad
def raster(V, F, c, fr, px, faces=None, bias=None, cull=True):
    """-> ids [h,w] (face index in F, -1 none), bary [h,w,2] (weights of corners 0 and 1), depth [h,w].
    faces = optional index array of the faces to draw; bias = per-face amount (units) added toward the viewer."""
    r, d, t = basis(c); ox, oy, w, h = fr
    X = (V @ r - ox) * px; Y = (V @ d - oy) * px; Z = V @ t
    ids = np.full((h, w), -1, np.int32); zb = np.full((h, w), -1e18); ba = np.zeros((h, w, 2), np.float32)
    it = range(len(F)) if faces is None else faces
    fc = facing(V, F, c) if cull else None      # the client culls faces wound away from the viewer (probe_cull.py)
    for i in it:
        if cull and fc[i] <= 1e-6: continue
        f = F[i]; x, y, z = X[f], Y[f], Z[f]
        if bias is not None: z = z + bias[i]
        x0, x1 = max(int(np.floor(x.min())), 0), min(int(np.ceil(x.max())), w - 1); y0, y1 = max(int(np.floor(y.min())), 0), min(int(np.ceil(y.max())), h - 1)
        if x1 < x0 or y1 < y0: continue
        den = (y[1] - y[2]) * (x[0] - x[2]) + (x[2] - x[1]) * (y[0] - y[2])
        if abs(den) < 1e-9: continue
        gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + .5, np.arange(y0, y1 + 1) + .5)
        a = ((y[1] - y[2]) * (gx - x[2]) + (x[2] - x[1]) * (gy - y[2])) / den; b = ((y[2] - y[0]) * (gx - x[2]) + (x[0] - x[2]) * (gy - y[2])) / den; g = 1 - a - b
        zz = a * z[0] + b * z[1] + g * z[2]; sub = zb[y0:y1 + 1, x0:x1 + 1]
        win = (a >= -1e-6) & (b >= -1e-6) & (g >= -1e-6) & (zz > sub)
        if not win.any(): continue
        sub[win] = zz[win]; ids[y0:y1 + 1, x0:x1 + 1][win] = i
        sb = ba[y0:y1 + 1, x0:x1 + 1]; sb[win, 0] = a[win]; sb[win, 1] = b[win]
    return ids, ba, zb
def interp(ids, ba, corner_vals):
    """Per pixel value from per-face corner values [nf,3] (Gouraud, as the client interpolates lit lightness)."""
    ok = ids >= 0; cv = corner_vals[np.clip(ids, 0, None)]
    a, b = ba[..., 0], ba[..., 1]; v = a * cv[..., 0] + b * cv[..., 1] + (1 - a - b) * cv[..., 2]
    return np.where(ok, v, np.nan)
def facing(V, F, c):
    """Per face: cosine between its normal and the direction to the viewer (sign by winding)."""
    n = np.cross(V[F[:, 1]] - V[F[:, 0]], V[F[:, 2]] - V[F[:, 0]]); n /= np.maximum(np.linalg.norm(n, axis=1), 1e-9)[:, None]
    return n @ basis(c)[2]
