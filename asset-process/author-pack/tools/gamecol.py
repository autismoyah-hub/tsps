"""The game's colour table: 16-bit colour (hue 0-63, saturation 0-7, lightness 0-127) -> screen RGB.

A direct port of `palette(brightness)` in rust-client/src/floor.rs (osrs-workspace f745d18), with the
brightness exponent 0.8 that the line-up renderer uses (examples/authored_scene_snapshot.rs:
`floor::palette(0.8)`). Not a fit: same arithmetic, same truncations.

    table()            -> NumPy array [65536, 3] of RGB, index = (h << 10) | (s << 7) | l
    rgb(h, s, l)       -> (r, g, b)
    nearest(rgb_pixel) -> (h, s, l, distance); see px2col.py for the caveats
"""
import numpy as np

BRIGHTNESS = 0.8
_table = None


def pack(h, s, l):
    return (h << 10) | (s << 7) | l


def split(c):
    return (c >> 10) & 63, (c >> 7) & 7, c & 127


def table():
    global _table
    if _table is not None:
        return _table
    gamma = [int((n / 256.0) ** BRIGHTNESS * 256.0) for n in range(256)]
    out = np.zeros((65536, 3), dtype=np.int32)
    for i in range(512):
        hue = (i >> 3) / 64.0 + 0.0078125
        sat = (i & 7) / 8.0 + 0.0625
        for j in range(128):
            light = j / 128.0
            high = light * (1.0 + sat) if light < 0.5 else light + sat - light * sat
            low = 2.0 * light - high

            def channel(h):
                if h > 1.0:
                    h -= 1.0
                if h < 0.0:
                    h += 1.0
                if 6.0 * h < 1.0:
                    v = low + (high - low) * 6.0 * h
                elif 2.0 * h < 1.0:
                    v = high
                elif 3.0 * h < 2.0:
                    v = low + (high - low) * (0.6666666666666666 - h) * 6.0
                else:
                    v = low
                return gamma[int(v * 256.0)]
            r, g, b = channel(hue + 0.3333333333333333), channel(hue), channel(hue - 0.3333333333333333)
            if (r, g, b) == (0, 0, 0):
                b = 1                      # the source keeps 0 for "transparent": .max(1)
            out[i * 128 + j] = (r, g, b)
    _table = out
    return out


def rgb(h, s, l):
    return tuple(int(v) for v in table()[pack(h, s, l)])


def key(a):
    """RGB array [..., 3] -> one integer per pixel."""
    a = np.asarray(a, dtype=np.int32)
    return a[..., 0] << 16 | a[..., 1] << 8 | a[..., 2]


def ground_mask(im):
    """True where a capture shows bare ground. The ground is the picture's commonest colour; every pixel on that
    colour's hue/sat line of the table (any lightness: the ground darkens round a solid object) counts as ground."""
    k = key(im)
    vals, counts = np.unique(k, return_counts=True)
    t = key(table())
    hit = np.nonzero(t == vals[counts.argmax()])[0]
    if len(hit) == 0:
        raise SystemExit('the commonest colour of this picture is not a game colour: is it a capture eKvR.png?')
    # several hue/sat lines can share that RGB (7/0 and 8/0 do at lightness 26): take all of them
    lines = np.concatenate([t[b:b + 128] for b in sorted({int(h) & 65408 for h in hit})])
    return np.isin(k, lines)


def nearest(p):
    """Nearest table entry to an RGB pixel. Among equally near entries the lowest saturation, then hue, wins."""
    d = ((table() - np.asarray(p, dtype=np.int32)[:3]) ** 2).sum(axis=1)
    c = int(d.argmin())
    return split(c) + (float(d[c]) ** 0.5,)
