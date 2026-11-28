"""softkit.py: a small kit for SOFT TRANSLUCENT MASS in a format that has no textures and no blending but per-face alpha.
Runs INSIDE Blender (5.2, headless):  blender -b --python-exit-code 1 --python YOUR_BUILD.py
Your build script does `import softkit as K`, makes a `K.Kit()`, calls the builders, and `kit.export(PATH.json)`.
The JSON is turned into a model, a skeleton, a sequence and a definition by fxkit.py (ordinary Python).

Units and axes are the GAME's inside this kit (so numbers match the sheets and rules): x east, y DOWN (height h is y = -h),
z north; whole units; a tile is 128; a projectile flies with -z forward. Blender is used for what it is good at: icospheres,
noise displacement, convex hulls, subdivision, smooth normals; nothing is kept in Blender's own axes.

Every builder takes
  vl   vertex label NAME (what a frame moves, turns, scales)           fl   face label NAME (what a frame fades)
  paint  (h, s, l) or a list of them (h 0..63, s 0..7, l 0..127)        alpha  0 opaque .. 253 (never 254; 255 = not drawn)
and returns the list of face indices it added. All faces are SMOOTH shaded unless flat=True: smooth shading IS the first
half of softness (real soft effects: 100% smooth faces at the median, EFFECTS2.md section 3).

Builders (rules and the real effects they were measured on are in EFFECTS2.md section 3 and 4)
  lump(centre, r, ...)                       one rounded, slightly irregular lump: 20 faces (level 1) or 80 (level 2)
  lump_cloud(centre, n, spread, r, ...)      n lumps scattered in a squashed ball; labels cycle through `sets` so sets can move apart
  glow_shell(centre, r, layers, ...)         nested shells: a bright core and 2..4 translucent shells, alpha rising outward
  trail(z0, z1, r0, r1, ...)                 a tapering tail behind a projectile: segments whose alpha rises to the tip
  dust_burst(centre, n, ring_r, r, ...)      a ring of lumps on the floor, label sets by sector, for a slam's dust
  pool(centre, r, ...)                       a liquid pool: blobby outline, tone rings dark centre to bright rim, a translucent skirt
                                             (the soft edge), ripple rings and bubbles on their own labels (the moving surface)
  disc_falloff(centre, r, rings, ...)        a flat disc whose alpha steps from centre to edge (shadow, glow on the floor, telegraph)
  anchor(point, vl)                          an undrawn pivot (alpha 255 triangle) to scale or turn other parts about
"""
import bpy, bmesh, json, math, random, pathlib
from mathutils import Vector, noise


def col(h, s, l): return (int(h) << 10) | (int(s) << 7) | int(l)


class Kit:
    def __init__(self, seed=1):
        self.V = []; self.F = []; self.rng = random.Random(seed)
        # per vertex: label name; per face: paint, alpha, face label name, flat
        self.VL = []; self.FP = []; self.FA = []; self.FL = []; self.FF = []

    # ------------------------------------------------------------------ plumbing
    def _add(self, verts, faces, vl, fl, paint, alpha, flat=False, face_paint=None, face_alpha=None, vls=None):
        """verts: [(x, y, z)] game units; faces: [(a, b, c)] local indices, wound so the normal (b-a) x (c-a) points OUT in
        game axes as the client wants it (checked by `outward`)."""
        o = len(self.V); out = []
        for i, p in enumerate(verts):
            self.V.append(tuple(int(round(c)) for c in p)); self.VL.append(vls[i] if vls else vl)
        for k, f in enumerate(faces):
            a, b, c = (o + i for i in f)
            if len({self.V[a], self.V[b], self.V[c]}) < 3: continue        # collapsed by rounding
            out.append(len(self.F)); self.F.append((a, b, c))
            p = face_paint[k] if face_paint else (paint[k % len(paint)] if isinstance(paint, list) else paint)
            self.FP.append(col(*p)); self.FA.append(int(face_alpha[k] if face_alpha else alpha)); self.FL.append(fl); self.FF.append(bool(flat))
        return out

    @staticmethod
    def _tris(bm, centre=None):
        """bmesh -> (verts in bmesh coordinates, triangles wound outward from `centre`)."""
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
        bm.verts.ensure_lookup_table(); bm.verts.index_update()
        V = [tuple(v.co) for v in bm.verts]; T = []
        c = Vector(centre) if centre is not None else sum((v.co for v in bm.verts), Vector()) / max(1, len(bm.verts))
        for f in bm.faces:
            a, b, d = (v.index for v in f.verts)
            n = (Vector(V[b]) - Vector(V[a])).cross(Vector(V[d]) - Vector(V[a])); m = (Vector(V[a]) + Vector(V[b]) + Vector(V[d])) / 3
            T.append((a, b, d) if n.dot(m - c) >= 0 else (a, d, b))
        return V, T

    # WINDING (found by drawing: the first kit pool was invisible and its lumps were lit from the inside): the client shows a
    # face from the side its cross product (b-a) x (c-a) points to, IN GAME AXES (y down). creature_kit.py reaches the same
    # stored order by building in (x, height, z) and flipping (FLIP = True). Every builder here winds a face so its cross
    # product points OUT of the piece (or UP for a floor piece); export() writes the faces as they are.

    # ------------------------------------------------------------------ builders
    def lump(self, centre, r, vl, fl, paint, alpha=0, level=1, rough=0.22, squash=(1, 1, 1), flat=False, tones=None):
        """A rounded lump. level 1 = icosphere of 20 faces (the game's dust and smoke lumps are 12 to 30 faces each), level 2 = 80.
        rough: each vertex is pushed in or out by up to this share of r (irregular, never spiky). tones: paints picked by
        height (top lighter): a list from top to bottom, 2 or 3 close tones."""
        bm = bmesh.new(); bmesh.ops.create_icosphere(bm, subdivisions=level, radius=1.0)
        seed = self.rng.random() * 100
        for v in bm.verts:
            k = 1 + rough * noise.noise(Vector((v.co.x * 1.3 + seed, v.co.y * 1.3, v.co.z * 1.3)))
            v.co = Vector((v.co.x * k * squash[0], v.co.y * k * squash[1], v.co.z * k * squash[2]))
        V, T = self._tris(bm, (0, 0, 0)); bm.free()
        verts = [(centre[0] + p[0] * r, centre[1] + p[1] * r, centre[2] + p[2] * r) for p in V]
        fp = None
        if tones:
            fp = []
            for a, b, c in T:
                y = (V[a][1] + V[b][1] + V[c][1]) / 3          # game y is down: y < 0 is the top
                fp.append(tones[min(len(tones) - 1, int((y + 1) / 2 * len(tones)))])
        return self._add(verts, T, vl, fl, paint, alpha, flat, face_paint=fp)

    def lump_cloud(self, centre, n, spread, r, vl, fl, paint, alpha=0, sets=1, level=1, squash_y=0.6, r_jitter=0.35, alpha_out=None, tones=None, rough=0.22):
        """n lumps in a ball of radius `spread` squashed in height. Lump i gets labels f'{vl}{i % sets}' / f'{fl}{i % sets}' when
        sets > 1 (so each set can be scaled about the centre at its own time: the cloud rolls open). alpha_out: lumps further
        from the centre take up to this alpha (the outside of the mass is thinner). Returns [(set index, centre, faces)]."""
        out = []
        for i in range(n):
            a = self.rng.random() * 2 * math.pi; rad = spread * math.sqrt(self.rng.random()); h = (self.rng.random() - 0.5) * 2 * spread * squash_y
            c = (centre[0] + rad * math.cos(a), centre[1] - abs(h) if centre[1] == 0 else centre[1] + h, centre[2] + rad * math.sin(a))
            rr = r * (1 + r_jitter * (self.rng.random() * 2 - 1))
            al = alpha if alpha_out is None else int(alpha + (alpha_out - alpha) * min(1, rad / max(1, spread)))
            s = i % sets
            fs = self.lump(c, rr, f'{vl}{s}' if sets > 1 else vl, f'{fl}{s}' if sets > 1 else fl, paint, al, level, rough, tones=tones)
            out.append((s, c, fs))
        return out

    def glow_shell(self, centre, r, vl, fl, paint_core, paint_shell, layers=3, grow=1.35, alpha_core=0, alphas=None, level=1, stretch=(1, 1, 1), core_flat=False, rough=0.0, fl_core=None):
        """Nested shells. The core (radius r) is bright and near opaque; each shell is `grow` times the one inside and more
        transparent: default alphas rise 110, 160, 200 (measured: 64..120 next to the core, 190..230 outermost). Shells are closed
        and smooth; seen from any side the eye adds them up: brightest through the middle, faint at the rim. paint_shell may be a
        list, one per shell (colour runs from the core's pale tone to the edge's saturated one). stretch = per-axis factor.
        Face labels: core f'{fl}0' (or fl_core), shells f'{fl}1', f'{fl}2', .. from the inside out."""
        alphas = alphas or [110, 160, 200, 225][:layers]
        faces = [self.lump(centre, r, vl, fl_core or f'{fl}0', paint_core, alpha_core, level, rough, squash=stretch, flat=core_flat)]
        rr = r
        for k in range(layers):
            rr *= grow
            p = paint_shell[min(k, len(paint_shell) - 1)] if isinstance(paint_shell, list) else paint_shell
            faces.append(self.lump(centre, rr, vl, f'{fl}{k + 1}', p, alphas[k], level, rough, squash=stretch))
        return faces

    def trail(self, z0, z1, r0, r1, vl, fl, paint, alpha0=120, alpha1=220, segments=4, sides=5, centre=(0, 0), wobble=0.0, per_segment=True):
        """A tapering tail along +z (BEHIND a projectile: its front is -z). `segments` rings from z0 (radius r0, alpha0) to z1
        (radius r1, alpha1); each segment a little thinner and fainter; closed to a point at the end. Give successive segments
        labels f'{vl}{k}' so a wave of scale can run down it (the flicker). Returns faces per segment."""
        rings = []
        for k in range(segments + 1):
            t = k / segments; z = z0 + (z1 - z0) * t; r = r0 + (r1 - r0) * t
            wob = wobble * r0 * math.sin(k * 2.1)
            rings.append([(centre[0] + r * math.cos(2 * math.pi * (i + 0.5 * k) / sides) + wob, centre[1] + r * math.sin(2 * math.pi * (i + 0.5 * k) / sides), z) for i in range(sides)])
        out = []
        for k in range(segments):
            a, b = rings[k], rings[k + 1]; verts = a + b; T = []
            for i in range(sides):
                j = (i + 1) % sides
                T += [(i, j, sides + i), (j, sides + j, sides + i)]
            al = int(alpha0 + (alpha1 - alpha0) * (k + 0.5) / segments)
            out.append(self._add(self._fix(verts, T, (centre[0], centre[1], (z0 + z1) / 2), radial_z=True), T, f'{vl}{k}' if per_segment else vl, f'{fl}{k}' if per_segment else fl, paint if not isinstance(paint, list) else paint[min(k, len(paint) - 1)], al))
        # tip
        a = rings[-1]; tip = (centre[0], centre[1], z1 + (z1 - z0) / segments * 0.8)
        T = [(i, (i + 1) % sides, sides) for i in range(sides)]
        self._fix(a + [tip], T, (centre[0], centre[1], (z0 + z1) / 2), radial_z=True)
        out.append(self._add(a + [tip], T, f'{vl}{segments - 1}' if per_segment else vl, f'{fl}{segments - 1}' if per_segment else fl, paint if not isinstance(paint, list) else paint[-1], alpha1))
        return out

    def streaks(self, n, z0, z1, spread, r0, vl, fl, paint, alpha0=40, alpha1=220, segments=3, sides=3, sets=3, jitter=0.35):
        """n thin tapering tongues behind a projectile, scattered  from the axis, each ending in a point: what real magic
        projectiles trail (the Nightmare's and Nex's: 130 faces, twelve alpha steps 16 to 240 along the tongues). Tongue i is on
        labels f'{vl}{i % sets}' / f'{fl}{i % sets}': stretch each set along z on its own beat and the tail flickers."""
        out = []
        for i in range(n):
            a = 2 * math.pi * (i + 0.3 * self.rng.random()) / n; d = spread * (0.6 + 0.4 * self.rng.random())
            zz1 = z0 + (z1 - z0) * (1 - jitter * self.rng.random())
            out.append(self.trail(z0 + (z1 - z0) * 0.1 * self.rng.random(), zz1, r0 * (0.7 + 0.6 * self.rng.random()), r0 * 0.25, f'{vl}{i % sets}', f'{fl}{i % sets}', paint, alpha0, alpha1, segments, sides,
                                  centre=(d * math.cos(a), d * math.sin(a)), per_segment=False))
        return out

    def _fix(self, verts, T, centre, radial_z=False):
        """rewind triangles in place so each faces away from the axis (radial_z) or the centre."""
        for k, (a, b, c) in enumerate(T):
            pa, pb, pc = Vector(verts[a]), Vector(verts[b]), Vector(verts[c]); n = (pb - pa).cross(pc - pa); m = (pa + pb + pc) / 3
            d = m - Vector(centre)
            if radial_z: d.z = 0
            if n.dot(d) < 0: T[k] = (a, c, b)
        return verts

    def dust_burst(self, centre, n, ring_r, r, vl, fl, paint, alpha=0, sets=4, level=1, tones=None, lift=0.5, jitter=0.25):
        """n lumps in a ring of radius ring_r sitting ON the floor (centre[1] is the floor's y, normally -2), half buried.
        Sets go by SECTOR (lump i -> set i * sets // n) so a scale about the centre throws each sector outward on its own beat."""
        out = []
        for i in range(n):
            a = 2 * math.pi * (i + self.rng.random() * 0.6) / n; rad = ring_r * (1 + jitter * (self.rng.random() * 2 - 1))
            rr = r * (0.75 + 0.5 * self.rng.random())
            c = (centre[0] + rad * math.cos(a), centre[1] - rr * lift, centre[2] + rad * math.sin(a))
            s = i * sets // n
            out.append((s, c, self.lump(c, rr, f'{vl}{s}', f'{fl}{s}', paint, alpha, level, 0.25, squash=(1, 0.8, 1), tones=tones)))
        return out

    def disc_falloff(self, centre, r, vl, fl, paint, alphas=(130, 150, 170, 190, 210, 230), sides=16, r_inner=0.0, shape=None, vls=None, fls=None):
        """A flat up-facing disc of concentric rings, alpha stepping outward (the real falling-rock shadow: 6 rings 130 to 230,
        264 faces). r_inner > 0 leaves a hole. shape(angle) scales the radius (a blobby edge). vls / fls: label per ring."""
        n = len(alphas); out = []
        rad = lambda k: r_inner * r + (r - r_inner * r) * k / n
        ring = lambda k: [(centre[0] + rad(k) * (shape(2 * math.pi * i / sides) if shape else 1) * math.cos(2 * math.pi * i / sides), centre[1],
                           centre[2] + rad(k) * (shape(2 * math.pi * i / sides) if shape else 1) * math.sin(2 * math.pi * i / sides)) for i in range(sides)]
        for k in range(n):
            p = paint[min(k, len(paint) - 1)] if isinstance(paint, list) else paint
            v = vls[k] if vls else vl; f = fls[k] if fls else fl
            if k == 0 and r_inner == 0:
                verts = [tuple(centre)] + ring(1); T = [(0, 1 + (i + 1) % sides, 1 + i) for i in range(sides)]
            else:
                verts = ring(k) + ring(k + 1); T = []
                for i in range(sides):
                    j = (i + 1) % sides
                    T += [(i, j, sides + i), (j, sides + j, sides + i)]
            self._up(verts, T)
            out.append(self._add(verts, T, v, f, p, alphas[k]))
        return out

    @staticmethod
    def _up(verts, T):
        """wind so the face normal points UP (game y negative)."""
        for k, (a, b, c) in enumerate(T):
            pa, pb, pc = Vector(verts[a]), Vector(verts[b]), Vector(verts[c])
            if (pb - pa).cross(pc - pa).y > 0: T[k] = (a, c, b)

    def pool(self, centre, r, vl, fl, tones, rim, skirt_alpha=(90, 170), lobes=((3, 0.16), (5, 0.09)), sides=18, dome=7, skirt=0.16,
             ripples=2, ripple_paint=None, bubbles=5, bubble_paint=None, drips=4, seed_phase=0.0):
        """A liquid pool about r across (r = 62 is one tile).
          body     3 tone rings, dark centre -> mid -> bright rim (tones = [centre, mid, edge]); opaque; a low dome (`dome` units high
                   at the centre... real pools are 3 to 33 high) so smooth shading rounds it
          rim      a narrow band of `rim` paint at the edge of the body (the meniscus: real pools carry their brightest tone there)
          skirt    the SOFT EDGE: two translucent bands beyond the rim, alpha skirt_alpha (inner, outer), lying on the floor
          outline  radius x (1 + sum a sin(k angle + phase)): a blobby, never circular edge
          ripples  rings lying on the surface on labels f'{vl}_rip{i}' / f'{fl}_rip{i}': scale them from 30% to 110% while they fade
          bubbles  small domes on f'{vl}_bub{i % 3}' / f'{fl}_bub{i % 3}': scale up from 0, then fade (pop)
          drips    satellite blobs beyond the edge on the body's labels (the splash that made it)
        Labels: body and skirt on (vl, fl); skirt faces on f'{fl}_skirt' so the edge can breathe separately."""
        ph = [self.rng.random() * 6.28 + seed_phase for _ in lobes]
        shape = lambda a: 1 + sum(amp * math.sin(k * a + p) for (k, amp), p in zip(lobes, ph))
        cx, cy, cz = centre
        radii = [0.0, 0.34, 0.66, 0.9, 1.0, 1.0 + skirt * 0.5, 1.0 + skirt]
        heights = [dome, dome * 0.92, dome * 0.62, dome * 0.22, 2, 1, 1]
        paints = [tones[0], tones[1], tones[2], rim, rim, rim]
        alphas = [0, 0, 0, 0, skirt_alpha[0], skirt_alpha[1]]
        ring = lambda k: [(cx + r * radii[k] * shape(2 * math.pi * i / sides) * math.cos(2 * math.pi * i / sides), cy - heights[k],
                           cz + r * radii[k] * shape(2 * math.pi * i / sides) * math.sin(2 * math.pi * i / sides)) for i in range(sides)]
        out = dict(body=[], skirt=[], ripples=[], bubbles=[])
        verts = [(cx, cy - dome, cz)] + ring(1); T = [(0, 1 + (i + 1) % sides, 1 + i) for i in range(sides)]; self._up(verts, T)
        out['body'] += self._add(verts, T, vl, fl, paints[0], 0)
        for k in range(1, 6):
            verts = ring(k) + ring(k + 1); T = []
            for i in range(sides):
                j = (i + 1) % sides
                T += [(i, j, sides + i), (j, sides + j, sides + i)]
            self._up(verts, T)
            fs = self._add(verts, T, vl, fl if k < 4 else f'{fl}_skirt', paints[k], alphas[k])
            out['body' if k < 4 else 'skirt'] += fs
        for i in range(drips):
            a = self.rng.random() * 6.28; d = r * (1.12 + 0.22 * self.rng.random()) * shape(a); rr = r * (0.07 + 0.06 * self.rng.random())
            out['body'] += self.lump((cx + d * math.cos(a), cy - 1, cz + d * math.sin(a)), rr, vl, fl, tones[2], 0, 1, 0.15, squash=(1, 0.25, 1))
        for i in range(ripples):
            # a thin ring lying just above the surface, at 60% radius at rest (the sequence scales it about the pool's centre)
            r0, r1 = r * 0.52, r * 0.60; n = 12
            verts = [(cx + q * shape(2 * math.pi * k / n) * math.cos(2 * math.pi * k / n), cy - dome - 1, cz + q * shape(2 * math.pi * k / n) * math.sin(2 * math.pi * k / n)) for q in (r0, r1) for k in range(n)]
            T = []
            for k in range(n):
                j = (k + 1) % n; T += [(k, j, n + k), (j, n + j, n + k)]
            self._up(verts, T)
            out['ripples'].append(self._add(verts, T, f'{vl}_rip{i}', f'{fl}_rip{i}', ripple_paint or rim, 110))
        for i in range(bubbles):
            a = self.rng.random() * 6.28; d = r * 0.62 * math.sqrt(self.rng.random()); rr = r * (0.07 + 0.05 * self.rng.random())
            c = (cx + d * math.cos(a), cy - dome * (0.95 - 0.5 * d / r), cz + d * math.sin(a))
            out['bubbles'].append(self.lump(c, rr, f'{vl}_bub{i % 3}', f'{fl}_bub{i % 3}', bubble_paint or rim, 40, 1, 0.05, squash=(1, 0.55, 1)))
        return out

    def anchor(self, point, vl):
        """An undrawn pivot: one tiny triangle at alpha 255 (never drawn) carrying a vertex label, for a joint to turn or scale
        other labels about. Its face label is 'anchor' (no frame fades it in)."""
        x, y, z = point
        return self._add([(x - 1, y, z), (x + 1, y, z), (x, y, z + 1)], [(0, 1, 2)], vl, 'anchor', (0, 0, 0), 255)

    # ------------------------------------------------------------------ hard parts (a soft effect keeps a FEW crisp solids:
    # the thing that was thrown, the metal of the creature; flat shading is for metal, crystal and shards)
    def torus(self, centre, axis, R, r, vl, fl, paint, alpha=0, major=10, minor=3, flat=True):
        """A ring of radius R, tube radius r, about `axis` (a game-axes vector). paint may be a list cycled round the tube."""
        ax = Vector(axis).normalized(); u = ax.orthogonal().normalized(); w = ax.cross(u)
        c = Vector(centre); verts = []; T = []; cl = []
        for i in range(major):
            a = 2 * math.pi * i / major; d = u * math.cos(a) + w * math.sin(a)
            for j in range(minor):
                b = 2 * math.pi * (j + 0.5) / minor
                verts.append(tuple(c + d * (R + r * math.cos(b)) + ax * (r * math.sin(b))))
        fp = []
        for i in range(major):
            for j in range(minor):
                a0 = i * minor + j; a1 = i * minor + (j + 1) % minor; b0 = ((i + 1) % major) * minor + j; b1 = ((i + 1) % major) * minor + (j + 1) % minor
                mid = 2 * math.pi * (i + 0.5) / major; cen = c + (u * math.cos(mid) + w * math.sin(mid)) * R
                for t in ((a0, b0, a1), (b0, b1, a1)):
                    pa, pb, pc = (Vector(verts[k]) for k in t); n = (pb - pa).cross(pc - pa)
                    T.append(t if n.dot((pa + pb + pc) / 3 - cen) >= 0 else (t[0], t[2], t[1]))
                    fp.append(paint[j % len(paint)] if isinstance(paint, list) else paint)
        return self._add(verts, T, vl, fl, paint, alpha, flat, face_paint=fp)

    def shard(self, centre, direction, length, width, vl, fl, paint, alpha=0, sides=3, girdle=0.3, flat=True):
        """A bipyramid (a crystal splinter) pointing along `direction`: tip at +length (1 - girdle), tail at -length girdle."""
        ax = Vector(direction).normalized(); u = ax.orthogonal().normalized(); w = ax.cross(u); c = Vector(centre)
        ring = [tuple(c + (u * math.cos(2 * math.pi * i / sides) + w * math.sin(2 * math.pi * i / sides)) * width) for i in range(sides)]
        verts = ring + [tuple(c + ax * length * (1 - girdle)), tuple(c - ax * length * girdle)]; T = []
        for i in range(sides):
            j = (i + 1) % sides
            for t in ((i, j, sides), (j, i, sides + 1)):
                pa, pb, pc = (Vector(verts[k]) for k in t); n = (pb - pa).cross(pc - pa)
                T.append(t if n.dot((pa + pb + pc) / 3 - c) >= 0 else (t[0], t[2], t[1]))
        return self._add(verts, T, vl, fl, paint, alpha, flat)

    def star4(self, centre, size, vl, fl, paint, alpha=60, waist=0.22, points=4):
        """A flat star lying on the floor (up-facing): `points` long points of `size`, waist radius size x waist."""
        cx, cy, cz = centre; verts = [(cx, cy, cz)]; n = points * 2
        for i in range(n):
            r = size if i % 2 == 0 else size * waist; a = 2 * math.pi * i / n
            verts.append((cx + r * math.cos(a), cy, cz + r * math.sin(a)))
        T = [(0, 1 + i, 1 + (i + 1) % n) for i in range(n)]; self._up(verts, T)
        return self._add(verts, T, vl, fl, paint, alpha, True)

    # ------------------------------------------------------------------ export
    def stats(self):
        return dict(vertices=len(self.V), faces=len(self.F), translucent=sum(1 for a in self.FA if 0 < a < 255), smooth=sum(1 for f in self.FF if not f),
                    vlabels=sorted(set(self.VL)), flabels=sorted(set(self.FL)), alphas=sorted(set(self.FA)))

    def export(self, path):
        """Writes the mesh as JSON for fxkit.py: faces as built (cross product pointing out of the piece in game axes)."""
        for a in self.FA: assert a != 254, 'alpha 254 is a special path in the client; use 253'
        d = dict(vertices=self.V, faces=[(a, b, c) for a, b, c in self.F], colors=self.FP, alpha=self.FA, vlabels=self.VL, flabels=self.FL, flat=self.FF)
        pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
        pathlib.Path(path).write_text(json.dumps(d))
        s = self.stats(); print(f'softkit: {path}: {s["vertices"]} vertices {s["faces"]} faces, translucent {s["translucent"]}, smooth {s["smooth"]}; alpha values {s["alphas"]}')
        return d
