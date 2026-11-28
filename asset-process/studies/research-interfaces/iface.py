"""iface.py: draw an interface group (or a JSON component list) the way the game client draws its static state.

Port of the tooling client's software path (client/rust-client/src: widget_layout.rs, interface_render.rs,
raster.rs, sprite_pixels.rs, font.rs, widget_line.rs). No client scripts are run: every component is drawn in
the state its definition gives it (the state the client has between opening the group and running its onLoad
scripts). Model components (kind 6) and item icons are NOT drawn; they are listed.

Usage:
  iface.py GROUP OUT.png [--cache 239|242] [--canvas 765x503] [--bg 29251f] [--report out.json]
  iface.py --json spec.json OUT.png [...]   (a component list in the decoded form; see real/*.json)
"""
import sys, json, argparse
from pathlib import Path
import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
import ifcache as C

CANVAS = (765, 503)
# False: the game client's software blend (alpha out of 256, as the tooling client and the Java client do).
# True: TSPS's WebGL blend for translucent fills and sprites (alpha out of 255, rounded per channel).
TSPS_BLEND = False


# ---------------------------------------------------------------- raster (raster.rs)

def _scale_sep(c, a):
    return (((c & 0xff00ff) * a >> 8) & 0xff00ff) + (((c & 0xff00) * a >> 8) & 0xff00)


class Raster:
    def __init__(self, w, h, bg=0xff29251f):
        self.w, self.h = w, h
        self.px = np.full((h, w), bg, dtype=np.int64)
        self.clip = [0, 0, w, h]

    def set_clip(self, l, t, r, b):
        l = min(max(l, 0), self.w); t = min(max(t, 0), self.h)
        self.clip = [l, t, max(min(max(r, 0), self.w), l), max(min(max(b, 0), self.h), t)]

    def _gl(self, c, t, alpha):
        """TSPS's WebGL blend: src*a + dst*(1-a) with a = (255 - transparency) / 255, rounded to 8 bits."""
        a = (alpha - 1) / 255.0
        c = np.asarray(c, dtype=np.int64); out = 0xff000000
        res = np.zeros(np.broadcast(c, t).shape, dtype=np.int64) | out
        for sh in (16, 8, 0):
            v = np.floor(((c >> sh) & 255) * a + ((t >> sh) & 255) * (1 - a) + 0.5).astype(np.int64)
            res |= v << sh
        return res

    def _blend_region(self, ys, xs, color, alpha):
        """blend() over pixels at index arrays (ys, xs); color may be an array of the same shape or a scalar."""
        a = min(alpha, 256)
        if TSPS_BLEND and 0 < a < 256:
            self.px[ys, xs] = self._gl(color, self.px[ys, xs], a)
            return
        if a == 256:
            self.px[ys, xs] = np.asarray(color, dtype=np.int64) | 0xff000000
        elif a:
            b = 256 - a
            t = self.px[ys, xs]
            c = np.asarray(color, dtype=np.int64)
            rb = ((c & 0xff00ff) * a + (t & 0xff00ff) * b) & 0xff00ff00
            g = ((c & 0xff00) * a + (t & 0xff00) * b) & 0xff0000
            self.px[ys, xs] = 0xff000000 | ((rb | g) >> 8)

    def fill_rect(self, x, y, w, h, color, alpha=256):
        l, t = max(x, self.clip[0]), max(y, self.clip[1])
        r, b = min(x + max(w, 0), self.clip[2]), min(y + max(h, 0), self.clip[3])
        if l >= r or t >= b:
            return
        ys, xs = np.mgrid[t:b, l:r]
        self._blend_region(ys, xs, color, alpha)

    def widget_span(self, rect, color, alpha, separate):
        x, y, w, h = rect
        if alpha == 256:
            return self.fill_rect(x, y, w, h, color, alpha)
        l, t = max(x, self.clip[0]), max(y, self.clip[1])
        r, b = min(x + max(w, 0), self.clip[2]), min(y + max(h, 0), self.clip[3])
        if l >= r or t >= b:
            return
        a, bb = alpha, 256 - alpha
        tgt = self.px[t:b, l:r]
        c = np.int64(color)
        if TSPS_BLEND:
            if alpha > 0:
                self.px[t:b, l:r] = self._gl(c, tgt, alpha)
            return
        if separate:
            self.px[t:b, l:r] = _scale_sep(c, a) + _scale_sep(tgt, bb)
        else:
            rb = ((c & 0xff00ff) * a + (tgt & 0xff00ff) * bb) & 0xff00ff00
            g = ((c & 0xff00) * a + (tgt & 0xff00) * bb) & 0xff0000
            self.px[t:b, l:r] = (rb | g) >> 8

    def widget_rect(self, rect, color, alpha, filled):
        x, y, w, h = rect
        if filled:
            self.widget_span(rect, color, alpha, True)
        else:
            self.widget_span([x, y, w, 1], color, alpha, False)
            self.widget_span([x, y + h - 1, w, 1], color, alpha, False)
            if h >= 3:
                self.widget_span([x, y + 1, 1, h - 2], color, alpha, False)
                self.widget_span([x + w - 1, y + 1, 1, h - 2], color, alpha, False)

    def fill_gradient(self, rect, colors, alpha):
        x, y, width, height = rect
        if width <= 0 or height <= 0:
            return
        l, t = max(x, self.clip[0]), max(y, self.clip[1])
        r, b = min(x + width, self.clip[2]), min(y + height, self.clip[3])
        if l >= r or t >= b:
            return
        step = 65536 // height
        c0, c1 = colors
        for row in range(t, b):
            phase = (row - y) * step
            a, bb = (65536 - phase) >> 8, phase >> 8
            rb = (((c0 & 0xff00ff) * a + (c1 & 0xff00ff) * bb) & 0xffffffff) & 0xff00ff00
            g = (((c0 & 0xff00) * a + (c1 & 0xff00) * bb) & 0xffffffff) & 0xff0000
            color = ((rb + g) & 0xffffffff) >> 8
            if alpha is None:
                self.px[row, l:r] = 0xff000000 | color
            else:
                op = ((alpha[0] * a + alpha[1] * bb) & 0xff00) >> 8
                if op == 0:
                    continue
                self.px[row, l:r] = _scale_sep(np.int64(color), op) + _scale_sep(self.px[row, l:r], 255 - op)

    def draw_mask(self, mask, w, h, x, y, color, alpha):
        l, t = max(x, self.clip[0]), max(y, self.clip[1])
        r, b = min(x + w, self.clip[2]), min(y + h, self.clip[3])
        if l >= r or t >= b:
            return
        m = mask[t - y:b - y, l - x:r - x] != 0
        if not m.any():
            return
        a = min(alpha, 256)

        def sc(c, wt):
            return ((((c & 0xff00ff) * wt) & 0xff00ff00) + (((c & 0xff00) * wt) & 0xff0000)) >> 8
        region = self.px[t:b, l:r]
        new = 0xff000000 | (sc(np.int64(color), a) + sc(region, 256 - a))
        region[m] = new[m] if np.ndim(new) else new

    def draw_indexed(self, sheet, frame, x, y):
        f = sheet.frames[frame]
        x += f['x']; y += f['y']
        idx = np.frombuffer(f['idx'], dtype=np.uint8).reshape(f['h'], f['w'])
        l, t = max(x, self.clip[0]), max(y, self.clip[1])
        r, b = min(x + f['w'], self.clip[2]), min(y + f['h'], self.clip[3])
        if l >= r or t >= b:
            return
        sub = idx[t - y:b - y, l - x:r - x]
        pal = np.array(sheet.palette, dtype=np.int64)
        m = sub != 0
        region = self.px[t:b, l:r]
        region[m] = pal[sub[m]] | 0xff000000

    def rgb(self):
        p = self.px
        return np.stack([(p >> 16) & 255, (p >> 8) & 255, p & 255], -1).astype(np.uint8)


# ---------------------------------------------------------------- sprites with widget effects (sprite_pixels.rs)

class Pixels:
    def __init__(self, sheet, frame=0):
        f = sheet.frames[frame]
        self.canvas = [sheet.width, sheet.height]
        self.offset = [f['x'], f['y']]
        self.size = [f['w'], f['h']]
        pal = np.array(sheet.palette, dtype=np.int64)
        self.p = pal[np.frombuffer(f['idx'], dtype=np.uint8)].reshape(f['h'], f['w']).copy()

    def flip(self, hz, vt):
        w, h = self.size
        if hz:
            self.p = self.p[:, ::-1].copy()
            self.offset[0] = self.canvas[0] - w - self.offset[0]
        if vt:
            self.p = self.p[::-1, :].copy()
            self.offset[1] = self.canvas[1] - h - self.offset[1]

    def pad(self, n):
        w, h = self.size
        left, top = min(n, self.offset[0]), min(n, self.offset[1])
        right = min(n, self.canvas[0] - w - self.offset[0])
        bottom = min(n, self.canvas[1] - h - self.offset[1])
        q = np.zeros((h + top + bottom, w + left + right), dtype=np.int64)
        q[top:top + h, left:left + w] = self.p
        self.p = q
        self.size = [w + left + right, h + top + bottom]
        self.offset = [self.offset[0] - left, self.offset[1] - top]

    def outline(self, color):
        p = self.p
        nz = p != 0
        n = np.zeros_like(nz)
        n[:, 1:] |= nz[:, :-1]; n[1:, :] |= nz[:-1, :]; n[:, :-1] |= nz[:, 1:]; n[:-1, :] |= nz[1:, :]
        out = p.copy()
        out[(~nz) & n] = color
        self.p = out

    def shadow(self, color):
        h, w = self.p.shape
        for y in range(h - 1, 0, -1):  # in place, bottom-up, right-to-left, as the client does
            row, above = self.p[y], self.p[y - 1]
            for x in range(w - 1, 0, -1):
                if row[x] == 0 and above[x - 1] != 0:
                    row[x] = color

    def effects(self, hz, vt, outline, shadow):
        self.flip(hz, vt)
        if outline > 0:
            self.pad(outline); self.outline(1)
        if outline >= 2:
            self.outline(0xffffff)
        if shadow:
            self.shadow(shadow)

    def draw_scaled(self, ras, rect, alpha):
        x, y, width, height = rect
        if width <= 0 or height <= 0 or 0 in self.canvas:
            return
        steps = [(self.canvas[0] << 16) // width, (self.canvas[1] << 16) // height]
        if 0 in steps:
            raise ValueError('sprite scale precision')
        org, dim = [x, y], [width, height]
        starts = [org[a] + ((self.offset[a] << 16) + steps[a] - 1) // steps[a] for a in (0, 1)]
        ends = [org[a] + (((self.offset[a] + self.size[a]) << 16) + steps[a] - 1) // steps[a]
                if self.size[a] < self.canvas[a] else org[a] + dim[a] for a in (0, 1)]
        l, t = max(starts[0], ras.clip[0]), max(starts[1], ras.clip[1])
        r, b = min(ends[0], ras.clip[2]), min(ends[1], ras.clip[3])
        if l >= r or t >= b:
            return
        dys = np.arange(t, b); dxs = np.arange(l, r)
        sy = ((dys - y) * steps[1] >> 16) - self.offset[1]
        sx = ((dxs - x) * steps[0] >> 16) - self.offset[0]
        vy = (sy >= 0) & (sy < self.size[1]); vx = (sx >= 0) & (sx < self.size[0])
        if not vy.any() or not vx.any():
            return
        dys, sy = dys[vy], sy[vy]; dxs, sx = dxs[vx], sx[vx]
        src = self.p[np.ix_(sy, sx)]
        m = src != 0
        Y, X = np.meshgrid(dys, dxs, indexing='ij')
        ras._blend_region(Y[m], X[m], src[m], alpha)

    def draw(self, ras, x, y, alpha):
        self.draw_scaled(ras, [x, y, self.canvas[0], self.canvas[1]], alpha)

    def draw_rotated(self, ras, center, angle, scale):
        import math
        if scale == 0 or 0 in self.size:
            return
        pivot = [(self.canvas[a] << 3) - (self.offset[a] << 4) for a in (0, 1)]
        center = [v << 4 for v in center]
        rad = (angle & 65535) * 9.587379924285257E-5
        sn, cs = math.sin(rad), math.cos(rad)
        fs, fc = math.floor(sn * scale + 0.5), math.floor(cs * scale + 0.5)
        lo, hi = [2 ** 31, 2 ** 31], [-2 ** 31, -2 ** 31]
        for px in (0, self.size[0] << 4):
            for py in (0, self.size[1] << 4):
                ax, ay = px - pivot[0], py - pivot[1]
                pt = [ax * fc + ay * fs, -ax * fs + ay * fc]
                for a in (0, 1):
                    lo[a] = min(lo[a], pt[a]); hi[a] = max(hi[a], pt[a])
        for a in (0, 1):
            lo[a] = max(((lo[a] >> 12) + center[a]) >> 4, ras.clip[a])
            hi[a] = min((((hi[a] + 4095) >> 12) + center[a] + 15) >> 4, ras.clip[a + 2])
        if lo[0] >= hi[0] or lo[1] >= hi[1]:
            return
        rec = 16777216.0 / scale
        ss, sc = math.floor(sn * rec + 0.5), math.floor(cs * rec + 0.5)
        dx = (lo[0] << 4) + 8 - center[0]
        dy = (lo[1] << 4) + 8 - center[1]
        rx = (pivot[0] << 8) - (dy * ss >> 4)
        ry = (pivot[1] << 8) + (dy * sc >> 4)
        for yy in range(lo[1], hi[1]):
            sx = rx + (dx * sc >> 4); sy = ry + (dx * ss >> 4)
            for xx in range(lo[0], hi[0]):
                if sx >= 0 and sy >= 0 and (sx >> 12) < self.size[0] and (sy >> 12) < self.size[1]:
                    v = int(self.p[sy >> 12, sx >> 12])
                    if v:
                        ras._blend_region(np.array([yy]), np.array([xx]), v, 256)
                sx += sc; sy += ss
            rx -= ss; ry += sc

    def draw_widget(self, ras, rect, angle, tiled, if3, alpha):
        x, y, width, height = rect
        if not if3:
            return self.draw(ras, x, y, 256)
        saved = list(ras.clip)
        if x >= saved[2] or y >= saved[3] or x + width <= saved[0] or y + height <= saved[1]:
            return
        sw, sh = self.canvas
        if sw == 0 or sh == 0:
            return
        if not tiled:
            if angle:
                return self.draw_rotated(ras, [x + width // 2, y + height // 2], angle, int(width * 4096 / sw))
            return self.draw_scaled(ras, rect, alpha)
        ras.set_clip(max(x, saved[0]), max(y, saved[1]), min(x + width, saved[2]), min(y + height, saved[3]))
        for col in range((width + sw - 1) // sw):
            for row in range((height + sh - 1) // sh):
                left, top = x + col * sw, y + row * sh
                if angle:
                    self.draw_rotated(ras, [left + sw // 2, top + sh // 2], angle, 4096)
                else:
                    self.draw(ras, left, top, alpha)
        ras.set_clip(*saved)


# ---------------------------------------------------------------- text (font.rs)

def _cp(c):
    return list(c.encode('cp1252', errors='replace'))


def tokens(text):
    out, tag = [], None
    for c in text:
        if c == '<':
            tag = ''
        elif c == '>' and tag is not None:
            t, tag = tag, None
            out.append(('g', 60) if t == 'lt' else ('g', 62) if t == 'gt' else ('t', t))
        elif tag is not None:
            tag += c
        elif c != '\0':
            for b in _cp(c):
                out.append(('g', 32 if b == 160 else b))
    return out


def icon(tag, icons):
    if tag.startswith('img='):
        try:
            i = int(tag[4:])
        except ValueError:
            return None
        if 0 <= i < len(icons):
            return icons[i]
    return None


def text_width(m, text, icons):
    width, prev = 0, None
    for kind, v in tokens(text):
        if kind == 'g':
            width += m.advances[v] + m.pair(prev, v); prev = v
        else:
            ic = icon(v, icons)
            if ic:
                width += ic.width; prev = None
    return width


def wrap(m, text, widths, icons):
    lines, builder = [], ''
    start, width, prev = 0, 0, 0
    bp, tag = None, None
    for c in text:
        if c == '<':
            tag = ''
            continue
        glyph = None
        if c == '>' and tag is not None:
            t, tag = tag, None
            builder += '<' + t + '>'
            if t == 'br':
                lines.append(builder[start:]); builder = ''; start = 0; width = 0; bp = None; prev = 0
            elif t in ('lt', 'gt'):
                b = 60 if t == 'lt' else 62
                width += m.advances[b] + m.pair(prev, b); prev = b
            else:
                ic = icon(t, icons)
                if ic:
                    width += ic.width; prev = 0
        elif tag is not None:
            tag += c
            continue
        elif c != '\0':
            builder += c
            for b in _cp(c):
                nb = 32 if b == 160 else b
                width += m.advances[nb] + m.pair(prev, b); prev = b
            glyph = c
        if glyph == ' ':
            bp = (len(builder), width, 1)
        if widths is not None:
            if width > widths[min(len(lines), len(widths) - 1)] and bp is not None:
                end, at_w, trim = bp
                lines.append(builder[start:end - trim]); start = end; bp = None; width -= at_w; prev = 0
        if glyph == '-':
            bp = (len(builder), width, 0)
    if len(builder) > start:
        lines.append(builder[start:])
    return lines


class Paint:
    def __init__(self, style):
        self.style = dict(style); self.strike = None; self.underline = None
        self.spacing = 0; self.remainder = 0; self.glyph = 0


def _glyph_masks(fnt):
    if not hasattr(fnt, '_masks'):
        fnt._masks = [np.frombuffer(g['idx'], dtype=np.uint8).reshape(g['h'], g['w']) for g in fnt.glyphs.frames]
    return fnt._masks


def draw_styled(ras, fnt, text, x, baseline, style, icons, paint):
    m = fnt.metrics
    masks = _glyph_masks(fnt)
    start = x
    y = baseline - m.ascent
    color, shadow, alpha = paint.style['color'], paint.style['shadow'], paint.style['alpha']
    strike, underline, prev = paint.strike, paint.underline, None
    for kind, v in tokens(text):
        if kind == 't':
            t = v
            ic = icon(t, icons)
            if ic:
                ras.draw_indexed(ic, 0, x, baseline - ic.height); x += ic.width; prev = None; continue

            def hx(s):
                try:
                    return int(s, 16)
                except ValueError:
                    return None
            if t.startswith('col=') and hx(t[4:]) is not None:
                color = hx(t[4:])
            elif t == '/col':
                color = style['color']
            elif t.startswith('str=') and hx(t[4:]) is not None:
                strike = hx(t[4:])
            elif t == 'str':
                strike = 0x800000
            elif t == '/str':
                strike = None
            elif t.startswith('u=') and hx(t[2:]) is not None:
                underline = hx(t[2:])
            elif t == 'u':
                underline = 0
            elif t == '/u':
                underline = None; paint.spacing = 0; paint.remainder = 0
            elif t.startswith('shad=') and hx(t[5:]) is not None:
                shadow = hx(t[5:])
            elif t == 'shad':
                shadow = 0
            elif t == '/shad':
                shadow = style['shadow']
            elif t == 'br':
                color = style['color']; shadow = style['shadow']; alpha = 256; strike = None; underline = None
        else:
            c = v
            x += m.pair(prev, c)
            g = fnt.glyphs.frames[c]
            paint.glyph += 1
            if c != 32:
                gx, gy = x + g['x'], y + g['y']
                if shadow is not None:
                    ras.draw_mask(masks[c], g['w'], g['h'], gx + 1, gy + 1, shadow, alpha)
                ras.draw_mask(masks[c], g['w'], g['h'], gx, gy, color, alpha)
            elif paint.spacing > 0:
                paint.remainder += paint.spacing; x += paint.remainder >> 8; paint.remainder &= 255
            adv = m.advances[c]
            if strike is not None:
                ras.fill_rect(x, y + int(m.ascent * 0.7), adv, 1, strike, 256)
            if underline is not None:
                ras.fill_rect(x, baseline + 1, adv, 1, underline, 256)
            x += adv
            prev = c
    paint.style = dict(color=color, shadow=shadow, alpha=alpha)
    paint.strike, paint.underline = strike, underline
    return x - start


def draw_paragraph(ras, fnt, text, rect, style, align, line_height, icons):
    x, y, width, height = rect
    style = dict(style)
    if style['alpha'] == 255:
        style['alpha'] = 256
    m = fnt.metrics
    if line_height == 0:
        line_height = m.ascent
    limits = None if (height < fnt.max_ascent + fnt.max_descent + line_height and height < line_height * 2) else [width]
    lines = wrap(m, text, limits, icons)
    count = len(lines)
    vertical = 1 if (align[1] == 3 and count == 1) else align[1]
    extra = height - fnt.max_ascent - fnt.max_descent - (count - 1) * line_height
    if vertical == 0:
        baseline = y + fnt.max_ascent
    elif vertical == 1:
        baseline = y + fnt.max_ascent + int(extra / 2)
    elif vertical == 2:
        baseline = y + height - fnt.max_descent - (count - 1) * line_height
    else:
        space = max(int(extra / (count + 1)), 0)
        line_height += space
        baseline = y + fnt.max_ascent + space
    paint = Paint(style)
    for i, line in enumerate(lines):
        tw = text_width(m, line, icons)
        if align[0] == 1:
            left = x + int((width - tw) / 2)
        elif align[0] == 2:
            left = x + width - tw
        else:
            left = x
        if align[0] > 2 and i + 1 < len(lines):
            spaces = sum(1 for k, v in tokens(line) if k == 'g' and v == 32)
            if spaces > 0:
                paint.spacing = int(((width - tw) << 8) / spaces)
        draw_styled(ras, fnt, line, left, baseline, style, icons, paint)
        paint.spacing = 0
        baseline += line_height
    return len(lines)


# ---------------------------------------------------------------- line (widget_line.rs)

def draw_line(r, a, b, width, color):
    a, b = list(a), list(b)
    dx, dy = b[0] - a[0], b[1] - a[1]
    thick = width > 1
    low = width // 2 if thick else 0
    high = width - low if thick else 0
    if dy == 0:
        left = min(a[0], b[0])
        return r.fill_rect(left, a[1] - low, max(a[0], b[0]) - left + 1, low + high + 1, color, 256)
    if dx == 0:
        top = min(a[1], b[1])
        return r.fill_rect(a[0] - low, top, low + high + 1, max(a[1], b[1]) - top + 1, color, 256)
    if thick and abs(dx) == abs(dy):
        sx = 1 if dx > 0 else -1; sy = 1 if dy > 0 else -1
        clip = r.clip
        mnx, mxx = clip[0] - width - 1, clip[2] + width + 1
        s, e = (mnx - a[0], mxx - a[0]) if sx > 0 else (a[0] - mxx, a[0] - mnx)
        for step in range(max(s, 0), min(e, abs(dx)) + 1):
            x, y = a[0] + step * sx, a[1] + step * sy
            for off in range(-low, high):
                px, py = x - sy * off, y + sx * off
                r.fill_rect(px, py, 1, 1, color, 256)
                if step & 1:
                    r.fill_rect(px + 1, py, 1, 1, color, 256)
                r.fill_rect(px + sx, py, 1, 1, color, 256)
        return
    if thick:
        axis = 1 if abs(dx) < abs(dy) else 0
    else:
        if dx + dy < 0:
            a, b = b, a
        axis = 1 if (b[0] - a[0]) <= (b[1] - a[1]) else 0
    if thick and b[axis] < a[axis]:
        a, b = b, a
    other = 1 - axis
    major = b[axis] - a[axis]
    num = (b[other] - a[other]) << 16
    if thick:
        slope = int((num + major // 2) / major) if num >= 0 else -int((-num + major // 2) / major)
    else:
        import math
        slope = math.floor(num / major + 0.5)
    clip = r.clip
    start, end = max(a[axis], clip[axis]), min(b[axis], clip[axis + 2] - 1)
    minor = (a[other] << 16) + 32768 + slope * (start - a[axis])
    for mj in range(start, end + 1):
        mp = (minor >> 16) - low
        if axis == 0:
            r.fill_rect(mj, mp, 1, low + high + 1, color, 256)
        else:
            r.fill_rect(mp, mj, low + high + 1, 1, color, 256)
        minor += slope


# ---------------------------------------------------------------- layout (widget_layout.rs)

def align(w, parent):
    size = [0, 0]
    for a in (0, 1):
        raw, mode = w['raw_size'][a], w['size_mode'][a]
        size[a] = raw if mode == 0 else parent[a] - raw if mode == 1 else (raw * parent[a]) >> 14 if mode == 2 else 0
    if w['size_mode'][0] == 4:
        size[0] = int(1 * size[1] / 1)
    if w['size_mode'][1] == 4:
        size[1] = int(1 * size[0] / 1)
    pos = [0, 0]
    for a in (0, 1):
        raw, mode = w['raw_position'][a], w['position_mode'][a]
        space = parent[a] - size[a]
        rel = (raw * parent[a]) >> 14
        half = int(space / 2)
        pos[a] = (raw if mode == 0 else raw + half if mode == 1 else space - raw if mode == 2 else
                  rel if mode == 3 else rel + half if mode == 4 else space - rel)
    return pos, size


def layout(widgets, canvas=CANVAS):
    by_parent = {}
    for w in widgets:
        by_parent.setdefault(w['parent'], []).append(w)
    out, order = {}, []

    def visit(parent, psize, offset, clip, hidden, depth):
        for w in by_parent.get(parent, []):
            pos, size = align(w, psize)
            ab = [offset[0] + pos[0], offset[1] + pos[1]]
            own = list(clip)
            for a in (0, 1):
                end = ab[a] + size[a]
                lo, hi = (min(ab[a], end), max(ab[a], end) + 1) if w['kind'] == 9 else (ab[a], end)
                own[a] = max(own[a], lo); own[a + 2] = min(own[a + 2], hi)
            h = hidden or w['hidden']
            out[w['id']] = dict(parent=parent, pos=pos, size=size, abs=ab, clip=own, hidden_raw=h)
            order.append(w['id'])
            if w['kind'] in (0, 11):
                ext = [w['scroll_size'][a] or size[a] for a in (0, 1)]
                visit(w['id'], ext, ab, own, h, depth + 1)
    visit(None, list(canvas), [0, 0], [0, 0, canvas[0], canvas[1]], False, 0)
    byid = {w['id']: w for w in widgets}
    for i in order:
        w, p = byid[i], out[i]
        ph = p['parent'] is not None and out[p['parent']]['hidden']
        p['hidden'] = ph or (w['hidden'] and (w['if3'] or w['kind'] == 0))
    return out, order


# ---------------------------------------------------------------- draw

def text_icons(cache):
    data = cache.file(17, 3, 0)
    r = C.Reader(data)
    ids = [None] * 11
    while True:
        op = r.u8()
        if op == 0:
            break
        if op == 1:
            r.u24()
        elif op == 2:
            for i in range(11):
                head = r.u8()
                if head & 128:
                    ids[i] = ((head & 127) << 24) | r.u24()
                else:
                    v = (head << 8) | r.u8()
                    ids[i] = None if v == 32767 else v
        elif op == 3:
            r.bytes(45)
        elif op == 5:
            r.u32(); r.u32()
    if ids[10] is None:
        return []
    sheet = C.sprite_sheet(cache, ids[10])
    out = []
    for f in sheet.frames:
        s = C.SpriteSheet.__new__(C.SpriteSheet)
        s.width, s.height, s.palette, s.frames = sheet.width, sheet.height, sheet.palette, [f]
        out.append(s)
    return out


def substitute(text):
    for i in range(5):
        text = text.replace(f'%{i + 1}', '-2')
    return text


def render(cache, widgets, canvas=CANVAS, bg=0xff29251f, raster=None):
    ras = raster or Raster(canvas[0], canvas[1], bg)
    lay, order = layout(widgets, canvas)
    byid = {w['id']: w for w in widgets}
    return draw(cache, order, lay, byid, ras, canvas)


def load_state(path):
    """A laid-out tree in draw order (state/*.jsonl from the tooling client's iface_dump after its scripts ran)."""
    rows = [json.loads(l) for l in Path(path).read_text().splitlines() if l.strip()]
    order, lay, byid = [], {}, {}
    for r in rows:
        i = r['id']
        order.append(i)
        lay[i] = dict(parent=None if r['parent'] < 0 else r['parent'], abs=r['abs'], size=r['size'], clip=r['clip'],
                      hidden=r['hidden'])
        w = dict(r)
        w.setdefault('file', i & 0xffff)
        w['line_reversed'] = r['line_reversed']
        byid[i] = w
    return order, lay, byid


def render_state(cache, path, canvas=CANVAS, bg=0xff29251f):
    order, lay, byid = load_state(path)
    return draw(cache, order, lay, byid, Raster(canvas[0], canvas[1], bg), canvas)


def draw(cache, order, lay, byid, ras, canvas):
    icons = text_icons(cache)
    report = dict(drawn=[], hidden=[], not_drawn=[], scripted=[])
    cclip = [0, 0, canvas[0], canvas[1]]
    for i in order:
        w, p = byid[i], lay[i]
        if w.get('listeners') or w.get('triggers') or w.get('cs1'):
            report['scripted'].append(dict(id=i, file=w['file'], kind=w['kind'],
                                           listeners=sorted(w['listeners']), triggers=sorted(w['triggers']),
                                           cs1=bool(w['cs1'])))
        if w.get('item', -1) != -1 and w['kind'] in (4, 5, 6):
            report['not_drawn'].append([i, f'item {w["item"]} x{w.get("count")} (icon/name)'])
            continue
        if p['hidden']:
            report['hidden'].append(i); continue
        k = w['kind']
        if (w['if3'] or k not in (4, 5, 8)) and (p['clip'][0] >= p['clip'][2] or p['clip'][1] >= p['clip'][3]):
            continue
        x, y = p['abs']; width, height = p['size']
        ras.set_clip(*p['clip'])
        alpha = 256 - w['transparency']
        cond = bool(w.get('condition', False))
        color = w['colors'][1 if cond else 0]
        pclip = lay[p['parent']]['clip'] if p['parent'] is not None else cclip

        def parent_clip():
            ras.set_clip(max(pclip[0], cclip[0]), max(pclip[1], cclip[1]), min(pclip[2], cclip[2]), min(pclip[3], cclip[3]))
        if w['content'] in (1336, 1337, 1338, 1339, 1400, 1401, 1402):
            report['not_drawn'].append([i, f'special content {w["content"]}'])
            if w['content'] in (1336, 1337, 1400, 1401):
                continue
        if k in (0, 11):
            continue
        if k == 3:
            if w['filled'] and w['fill_mode'] in (1, 2):
                ga = [255 - w['transparency'], (255 - w.get('transparency_bottom', 0)) & 255] if w['fill_mode'] == 2 else None
                ras.fill_gradient([x, y, width, height], [w['colors'][0], w['colors'][1]], ga)
            else:
                ras.widget_rect([x, y, width, height], color, alpha, w['filled'])
            report['drawn'].append(i)
        elif k == 4:
            parent_clip()
            if w['font'] < 0:
                continue
            fnt = C.font(cache, w['font'])
            text = w['alternate_text'] if (cond and w.get('alternate_text')) else w['text']
            text = text if w['if3'] else substitute(text)
            draw_paragraph(ras, fnt, text, [x, y, width, height],
                           dict(color=color, shadow=0 if w['text_shadow'] else None, alpha=255 - w['transparency']),
                           w['text_alignment'], w['line_height'], icons)
            report['drawn'].append(i)
        elif k == 5:
            sid = w['sprites'][0] if (w['if3'] or cond) else w['sprites'][1]
            if sid < 0:
                continue
            try:
                px = Pixels(C.sprite_sheet(cache, sid))
            except KeyError:
                report['not_drawn'].append([i, f'sprite {sid} missing']); continue
            px.effects(w['flip_h'], w['flip_v'], w['outline'], w['sprite_shadow'])
            parent_clip()
            px.draw_widget(ras, [x, y, width, height], w['sprite_angle'], w['sprite_tiling'], w['if3'], alpha)
            report['drawn'].append(i)
        elif k == 6:
            report['not_drawn'].append([i, 'model'])
        elif k == 8:
            report['not_drawn'].append([i, 'if1 tooltip (hover only)'])
        elif k == 9:
            parent_clip()
            a, b = ([x, y + height], [x + width, y]) if w['line_reversed'] else ([x, y], [x + width, y + height])
            draw_line(ras, a, b, w['line_width'], w['colors'][0])
            report['drawn'].append(i)
        else:
            report['not_drawn'].append([i, f'kind {k}'])
    return ras, report


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('pos', nargs='+', help='[GROUP] OUT.png')
    ap.add_argument('--json', help='component list (decoded form) instead of a cache group')
    ap.add_argument('--state', help='laid-out tree in draw order (iface_dump .jsonl) instead of a cache group')
    ap.add_argument('--cache', default='239')
    ap.add_argument('--canvas', default='765x503')
    ap.add_argument('--bg', default='29251f')
    ap.add_argument('--report')
    a = ap.parse_args()
    a.out = a.pos[-1]
    cache = C.Cache(C.CACHE_242 if a.cache == '242' else C.CACHE_239 if a.cache == '239' else a.cache)
    canvas = tuple(int(v) for v in a.canvas.split('x'))
    bg = 0xff000000 | int(a.bg, 16)
    if a.state:
        ras, rep = render_state(cache, a.state, canvas, bg)
    else:
        if a.json:
            spec = json.loads(Path(a.json).read_text())
            widgets = spec['components'] if isinstance(spec, dict) else spec
        else:
            widgets = C.load_interface(cache, int(a.pos[0]))
        ras, rep = render(cache, widgets, canvas, bg)
    Image.fromarray(ras.rgb()).save(a.out)
    print(f"drawn {len(rep['drawn'])}, hidden {len(rep['hidden'])}, not drawn {len(rep['not_drawn'])}, "
          f"scripted {len(rep['scripted'])}")
    if a.report:
        Path(a.report).write_text(json.dumps(rep, indent=1))


if __name__ == '__main__':
    main()
