"""Minimal authoring kit for the native format: welded meshes, HSL face colours, smooth shading.

Coordinates here are x, y (up, positive), z in game units; encode() flips y to the cache's convention
and orients every face outward using the winding the real models use.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, math, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
sys.path.insert(1, _AP.fix(r'<NATIVE_ASSETS>'))
import native_codec


def hsl(h, s, l):
    return (h << 10) | (s << 7) | l


class Mesh:
    def __init__(self):
        self.v, self.index, self.f, self.c, self.flat, self.prio, self.depth = [], {}, [], [], [], [], []

    def vert(self, x, y, z):
        k = (round(x), round(y), round(z))
        if k not in self.index:           # one vertex per position: the real assets never split a position
            self.index[k] = len(self.v); self.v.append(k)
        return self.index[k]

    out = None   # function(centroid) -> direction the next faces must face, in y-up coordinates

    def tri(self, a, b, c, colour, flat=False, prio=None, depth=0):
        if len({a, b, c}) != 3:
            return
        if self.out is not None:
            A, B, C = self.v[a], self.v[b], self.v[c]
            n = ((B[1]-A[1])*(C[2]-A[2])-(B[2]-A[2])*(C[1]-A[1]), (B[2]-A[2])*(C[0]-A[0])-(B[0]-A[0])*(C[2]-A[2]), (B[0]-A[0])*(C[1]-A[1])-(B[1]-A[1])*(C[0]-A[0]))
            if not any(n):
                return                     # degenerate after rounding to whole units
            w = self.out(tuple((A[i] + B[i] + C[i]) / 3 for i in range(3)))
            if n[0]*w[0] + n[1]*w[1] + n[2]*w[2] < 0:
                b, c = c, b
        self.f.append((a, b, c)); self.c.append(colour); self.flat.append(1 if flat else 0); self.prio.append(prio); self.depth.append(depth)

    def quad(self, a, b, c, d, colour, flat=False, prio=None, depth=0):
        self.tri(a, b, c, colour, flat, prio, depth); self.tri(a, c, d, colour, flat, prio, depth)

    def ring(self, y, radius, n, phase=0.0, squash=1.0):
        return [self.vert(radius * math.cos(phase + 2 * math.pi * i / n), y, squash * radius * math.sin(phase + 2 * math.pi * i / n)) for i in range(n)]

    def band(self, lower, upper, colour):
        """Quads between two rings of equal count, facing away from the y axis."""
        n = len(lower)
        for i in range(n):
            j = (i + 1) % n
            c = colour[i % len(colour)] if isinstance(colour, (list, tuple)) else colour
            self.quad(lower[i], lower[j], upper[j], upper[i], c)

    def encode(self, priority=4, vertex_label=None, face_label=None, face_textures=None, alpha=None):
        # y is mirrored for the cache, which reverses winding; the real models' cross product points
        # outward in cache coordinates, so faces oriented outward here are swapped once.
        verts = [[x, -y, z] for x, y, z in self.v]
        faces = [(a, c, b) for a, b, c in self.f]
        g = dict(vertices=verts, faces=faces, colors=list(self.c), priority=priority)
        if face_textures is not None and any(t >= 0 for t in face_textures):
            g['face_textures'] = face_textures
        if alpha is not None and any(alpha):
            g['alpha'] = alpha
        if any(p is not None for p in self.prio):
            # per-face draw priority: within one model, higher is drawn later, so a strip lying on a surface shows
            g['priorities'] = [priority if p is None else p for p in self.prio]
        # animation labels: the game moves vertices by label. A weapon in the right hand carries vertex label 50
        # and face label 29 on everything; without them it would not follow the hand in any animation.
        if vertex_label is not None:
            g['vertex_skins'] = [vertex_label] * len(verts)
        if face_label is not None:
            g['face_skins'] = [face_label] * len(faces)
        # per-face depth offset: on SCENERY (drawn with a depth buffer, priority ignored) this is what makes a face
        # lying exactly in another's plane show; 3 is enough. On a player it does nothing: priority decides there.
        if any(self.depth):
            g['face_flags'] = [d & 255 for d in self.depth]
        if any(self.flat):
            g['render_types'] = list(self.flat)      # 1 = flat-shaded face, 0 = smooth
        return native_codec.model(g)


def object_def(model_id, name='Authored', sx=1, sz=1, ambient=10, contrast=30):
    # Lighting values come from the real neighbours' definitions (they differ by region: 10/30, 20/30, none).
    # The object is left solid, as real furniture is, so the game darkens the ground around it as it does theirs.
    return (bytes([39, contrast & 255, 29, ambient & 255]) + bytes([7, 1]) + model_id.to_bytes(4, 'big') + bytes([2]) + name.encode() + bytes([0, 14, sx, 15, sz, 19, 0, 0]))
