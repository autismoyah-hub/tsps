"""meshkit.py: creature_kit.Kit with face labels, part names for labels, and shapes in ANY direction (a creature with its
own skeleton is not built along the z axis of a borrowed one).

    k = MKit(labels)                    labels: {name: number} from skelc's rig.json; k.lab(name) -> number
    k.fl = 1                            the face label given to faces made from now on (alpha groups act on these)
    k.loop(centre, axis, rx, ry, n, label, up=..., phase=..)   a ring of n points in the plane across `axis`
    k.tube(points, radii, n, label, paint, caps=True, labels=[per ring])   a loft along a polyline, optionally closed
    k.torus(centre, normal, R, w, t, n, label, paints)         a faceted band ring: diamond section, 4 quads a segment
    k.ball(centre, r, label, paint)                            an octahedron (8 faces): a joint ball
    k.bipyramid(centre, axis, r, up, down, n, label, paint)    a gem / link / plumb bob
Coordinates are (x, height, z), height up, front = -z, as creature_kit's.
"""
import sys, math, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent / 'research-npc'))
import creature_kit as CK, meshio
from creature_kit import sub, cross, dot, norm, mean
col = meshio.col


def add(a, b): return (a[0] + b[0], a[1] + b[1], a[2] + b[2])
def mul(a, s): return (a[0] * s, a[1] * s, a[2] * s)


def basis(axis, up=(0, 1, 0)):
    """two unit vectors across `axis`; the first as close to `up` as it can be"""
    a = norm(axis)
    u = sub(up, mul(a, dot(up, a)))
    if dot(u, u) < 1e-6:
        up = (0, 0, -1); u = sub(up, mul(a, dot(up, a)))
    u = norm(u)
    return u, norm(cross(a, u))


class MKit(CK.Kit):
    def __init__(self, labels):
        super().__init__()
        self.names = labels; self.fl = 1; self.FL = []

    def lab(self, name): return name if isinstance(name, int) else self.names[name]

    def face(self, a, b, c, paint, away_from, flat=False, depth=0, prio=0):
        n = len(self.F)
        super().face(a, b, c, paint, away_from, flat=flat, depth=depth, prio=prio)
        if len(self.F) > n: self.FL.append(self.fl)

    def model(self):
        m = super().model(); m['face_skins'] = list(self.FL); return m

    def loop(self, centre, axis, rx, ry, n, label, up=(0, 1, 0), phase=0.5):
        u, w = basis(axis, up)
        out = []
        for i in range(n):
            a = 2 * math.pi * (i + phase) / n
            out.append(self.v(add(centre, add(mul(u, rx * math.cos(a)), mul(w, ry * math.sin(a)))), self.lab(label)))
        return out

    def tube(self, points, radii, n, label, paint, caps=True, labels=None, up=(0, 1, 0), phase=0.5, flat=False, squash=1.0):
        """radii: one number or (rx, ry) per point; labels: a label per ring (default `label` for all).
        A radius of 0 at an end closes it to a point."""
        rings = []
        for i, p in enumerate(points):
            a = sub(points[min(i + 1, len(points) - 1)], points[max(i - 1, 0)])
            r = radii[i]; rx, ry = (r, r * squash) if not isinstance(r, tuple) else r
            lab = labels[i] if labels else label
            if rx == 0: rings.append(('tip', self.v(p, self.lab(lab))))
            else: rings.append(('ring', self.loop(p, a, rx, ry, n, lab, up=up, phase=phase)))
        for (ka, a), (kb, b) in zip(rings, rings[1:]):
            if ka == 'ring' and kb == 'ring': self.band(a, b, paint, flat=flat)
            elif ka == 'ring':
                axis = mean([self.V[i] for i in a] + [self.V[b]])
                for i in range(n): self.face(a[i], a[(i + 1) % n], b, paint, axis, flat=flat)
            elif kb == 'ring':
                axis = mean([self.V[i] for i in b] + [self.V[a]])
                for i in range(n): self.face(b[i], b[(i + 1) % n], a, paint, axis, flat=flat)
        if caps:
            inside = mean(points)
            for kind, r in (rings[0], rings[-1]):
                if kind == 'ring': self.cap(r, paint, inside, flat=flat)
        return rings

    def torus(self, centre, normal, R, w, t, n, label, paints, up=(0, 1, 0), phase=0.0, flat=True):
        """A band ring about `centre` in the plane across `normal`. Section: outer (R + w), upper (R, +t), inner
        (R - w), lower (R, -t). paints = (outer-upper, inner-upper, inner-lower, outer-lower): a colour or a list
        cycled round the ring (so the tone can change segment by segment)."""
        a_ = norm(normal); u, v = basis(a_, up)
        sec = []
        for i in range(n):
            ang = 2 * math.pi * (i + phase) / n
            d = add(mul(u, math.cos(ang)), mul(v, math.sin(ang)))
            c = add(centre, mul(d, R))
            sec.append([self.v(add(centre, mul(d, R + w)), self.lab(label)), self.v(add(c, mul(a_, t)), self.lab(label)),
                        self.v(add(centre, mul(d, R - w)), self.lab(label)), self.v(add(c, mul(a_, -t)), self.lab(label))])
        for i in range(n):
            s0, s1 = sec[i], sec[(i + 1) % n]
            axis = mean([self.V[x] for x in s0 + s1])
            for q in range(4):
                p = paints[q]; p = p[i % len(p)] if isinstance(p, (list, tuple)) else p
                a, b, c, d = s0[q], s0[(q + 1) % 4], s1[(q + 1) % 4], s1[q]
                self.face(a, b, c, p, axis, flat=flat); self.face(a, c, d, p, axis, flat=flat)
        return sec

    def ball(self, centre, r, label, paint, flat=False, squash=1.0):
        c = centre
        pts = [(r, 0, 0), (-r, 0, 0), (0, r * squash, 0), (0, -r * squash, 0), (0, 0, r), (0, 0, -r)]
        ix = [self.v(add(c, p), self.lab(label)) for p in pts]
        for x in (0, 1):
            for y in (2, 3):
                for z in (4, 5): self.face(ix[x], ix[y], ix[z], paint, c, flat=flat)
        return ix

    def bipyramid(self, centre, axis, r, up_len, down_len, n, label, paint, flat=True, phase=0.5, labels=None, up=(0, 1, 0), girdle=0.0):
        """apex at centre + axis * up_len, apex at centre - axis * down_len, an n-gon of radius r between; girdle > 0
        gives two n-gons that far either side of the centre (a band)."""
        a_ = norm(axis)
        la = labels or (label, label, label)
        top = self.v(add(centre, mul(a_, up_len)), self.lab(la[0])); bot = self.v(add(centre, mul(a_, -down_len)), self.lab(la[2]))
        if girdle:
            r1 = self.loop(add(centre, mul(a_, girdle)), a_, r, r, n, la[1], up=up, phase=phase)
            r2 = self.loop(add(centre, mul(a_, -girdle)), a_, r, r, n, la[1], up=up, phase=phase)
            self.band(r1, r2, paint, flat=flat)
        else:
            r1 = r2 = self.loop(centre, a_, r, r, n, la[1], up=up, phase=phase)
        for i in range(n):
            self.face(r1[i], r1[(i + 1) % n], top, paint, centre, flat=flat)
            self.face(r2[i], r2[(i + 1) % n], bot, paint, centre, flat=flat)
        return top, r1, r2, bot

    def edges_report(self):
        """open or doubled edges per label region: every closed solid should give 0"""
        import collections
        e = collections.Counter()
        for f in self.F:
            for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): e[(a, b)] += 1
        bad = sum(1 for (a, b), n in e.items() if e.get((b, a), 0) != n)
        return bad
