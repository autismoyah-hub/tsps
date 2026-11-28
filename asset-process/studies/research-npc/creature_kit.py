"""A small kit for building a creature mesh by script, in game units, labelled part by part for a borrowed frame base.

Coordinates given to the kit are (x, height, z): height up, the creature's length along z. Faces are oriented
outward automatically (away from the axis of the loft they belong to), every vertex is rounded to a whole unit and
welded by position (the first label given to a position wins), and the file is written through meshio.save.

    k = Kit()
    a = k.ring_z(z, h, rx, ry, n, label)        a ring of n points in a plane across the body (around the z axis)
    b = k.ring_y(x, h, z, rx, rz, n, label)     a ring in a level plane (around a leg)
    k.band(a, b, paint)                         faces joining two rings of equal count
    k.cap(a, paint, towards)                    close a ring with a fan to its centre; `towards` is the outward side
    k.tip(a, point, label, paint)               close a ring to a point
    k.tri / k.quad                              single faces with an explicit outward reference point
    paint is a colour (16-bit) or a function(normal, centre) -> colour, so tone can follow the facing.
    k.save(path)  ;  k.report()
"""
import math, sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import meshio
col = meshio.col
def sub(a, b): return (a[0] - b[0], a[1] - b[1], a[2] - b[2])
def cross(a, b): return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])
def dot(a, b): return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
def norm(a):
    l = math.sqrt(dot(a, a)) or 1.0; return (a[0] / l, a[1] / l, a[2] / l)
def mean(ps): return tuple(sum(p[i] for p in ps) / len(ps) for i in range(3))
class Kit:
    FLIP = True   # set by test: which winding the client treats as the front (see hound/notes.md round 0)
    def __init__(self):
        self.V = []; self.L = []; self.index = {}; self.F = []; self.C = []; self.T = []; self.D = []; self.P = []
    def v(self, p, label):
        key = (int(round(p[0])), int(round(p[1])), int(round(p[2])))
        if key not in self.index:
            self.index[key] = len(self.V); self.V.append(key); self.L.append(label)
        return self.index[key]
    def ring_z(self, z, h, rx, ry, n, label, phase=0.5, squash_top=1.0, squash_bottom=1.0, x0=0.0):
        out = []
        for i in range(n):
            a = 2 * math.pi * (i + phase) / n; s = math.sin(a)
            out.append(self.v((x0 + rx * math.cos(a), h + ry * s * (squash_top if s > 0 else squash_bottom), z), label))
        return out
    def ring_y(self, x, h, z, rx, rz, n, label, phase=0.5, tilt=0.0):
        """A ring around a leg; `tilt` raises the front (-z) edge by that many units and lowers the back."""
        out = []
        for i in range(n):
            a = 2 * math.pi * (i + phase) / n
            out.append(self.v((x + rx * math.cos(a), h - tilt * math.sin(a), z + rz * math.sin(a)), label))
        return out
    def ring_x(self, x, h, z, ry, rz, n, label, phase=0.5):
        return [self.v((x, h + ry * math.cos(2 * math.pi * (i + phase) / n), z + rz * math.sin(2 * math.pi * (i + phase) / n)), label) for i in range(n)]
    def face(self, a, b, c, paint, away_from, flat=False, depth=0, prio=0):
        if len({a, b, c}) < 3: return
        pa, pb, pc = self.V[a], self.V[b], self.V[c]
        n = cross(sub(pb, pa), sub(pc, pa))
        if dot(n, n) == 0: return
        centre = mean([pa, pb, pc])
        if dot(n, sub(centre, away_from)) < 0: b, c = c, b; n = (-n[0], -n[1], -n[2])
        colour = paint(norm(n), centre) if callable(paint) else paint
        self.F.append((a, c, b) if self.FLIP else (a, b, c)); self.C.append(colour); self.T.append(1 if flat else 0); self.D.append(depth); self.P.append(prio)
    def tri(self, a, b, c, paint, away_from, **kw): self.face(a, b, c, paint, away_from, **kw)
    def quad(self, a, b, c, d, paint, away_from, **kw):
        self.face(a, b, c, paint, away_from, **kw); self.face(a, c, d, paint, away_from, **kw)
    def band(self, r0, r1, paint, **kw):
        n = len(r0); axis = mean([self.V[i] for i in r0 + r1])
        for i in range(n):
            j = (i + 1) % n
            # split each quad along its shorter diagonal
            d1 = dot(sub(self.V[r0[i]], self.V[r1[j]]), sub(self.V[r0[i]], self.V[r1[j]])); d2 = dot(sub(self.V[r0[j]], self.V[r1[i]]), sub(self.V[r0[j]], self.V[r1[i]]))
            if d1 <= d2:
                self.face(r0[i], r0[j], r1[j], paint, axis, **kw); self.face(r0[i], r1[j], r1[i], paint, axis, **kw)
            else:
                self.face(r0[i], r0[j], r1[i], paint, axis, **kw); self.face(r0[j], r1[j], r1[i], paint, axis, **kw)
    def loft(self, rings, paint, **kw):
        for a, b in zip(rings, rings[1:]): self.band(a, b, paint, **kw)
    def cap(self, r, paint, inside, label=None, **kw):
        """Fan a ring to its own centre; `inside` is a point on the side that should NOT see it."""
        c = mean([self.V[i] for i in r]); ci = self.v(c, self.L[r[0]] if label is None else label)
        for i in range(len(r)): self.face(r[i], r[(i + 1) % len(r)], ci, paint, inside, **kw)
    def tip(self, r, point, label, paint, **kw):
        t = self.v(point, label); axis = mean([self.V[i] for i in r] + [self.V[t]])
        for i in range(len(r)): self.face(r[i], r[(i + 1) % len(r)], t, paint, axis, **kw)
        return t
    def pivot(self, label):
        ps = [self.V[i] for i in range(len(self.V)) if self.L[i] == label]
        return tuple(round(c, 1) for c in mean(ps)) if ps else None
    def model(self):
        return dict(vertices=[(x, -h, z) for x, h, z in self.V], faces=list(self.F), colors=list(self.C), priorities=list(self.P),
                    render_types=list(self.T), alpha=[0] * len(self.F), depth=list(self.D), vertex_skins=list(self.L), face_skins=[0] * len(self.F))
    def save(self, path): meshio.save(self.model(), path)
    def report(self):
        import collections
        xs, hs, zs = zip(*self.V)
        print(f'{len(self.V)} vertices {len(self.F)} faces; x {min(xs)}..{max(xs)} height {min(hs)}..{max(hs)} z {min(zs)}..{max(zs)}')
        area = collections.Counter()
        for f, c in zip(self.F, self.C):
            n = cross(sub(self.V[f[1]], self.V[f[0]]), sub(self.V[f[2]], self.V[f[0]])); area[c] += math.sqrt(dot(n, n)) / 2
        tot = sum(area.values())
        print('   colours by surface: ' + ', '.join(f'{meshio.hsl(c)} {100 * a / tot:.0f}%' for c, a in area.most_common()))
        print('   labels: ' + ' '.join(f'{l}:{n}' for l, n in sorted(collections.Counter(self.L).items())))
