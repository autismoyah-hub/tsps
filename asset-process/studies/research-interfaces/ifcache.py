"""Read-only cache access for interfaces, sprites and fonts (revision 239 local cache, or TSPS's 242 cache).

Ported from the tooling client (client/rust-client/src: cache.rs, widget.rs, sprite.rs, font.rs).
Private: everything decoded from the cache stays under <ROOT>.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import bz2, zlib
from pathlib import Path

CACHE_239 = Path(_AP.fix(r'<OSRS_CACHE_239>'))
CACHE_242 = Path(_AP.fix(r'<OSRS_CACHE_242>'))


def name_hash(name):
    h = 0
    for b in name.encode('cp1252'):
        if b > 127:
            b -= 256
        h = (h * 31 + b) & 0xffffffff
    return h


class Reader:
    def __init__(self, data, p=0):
        self.d, self.p = data, p

    def u8(self):
        v = self.d[self.p]; self.p += 1; return v

    def s8(self):
        v = self.u8(); return v - 256 if v > 127 else v

    def u16(self):
        v = int.from_bytes(self.d[self.p:self.p + 2], 'big'); self.p += 2; return v

    def s16(self):
        v = self.u16(); return v - 65536 if v > 32767 else v

    def u24(self):
        v = int.from_bytes(self.d[self.p:self.p + 3], 'big'); self.p += 3; return v

    def u32(self):
        v = int.from_bytes(self.d[self.p:self.p + 4], 'big'); self.p += 4; return v

    def s32(self):
        v = self.u32(); return v - (1 << 32) if v >= 1 << 31 else v

    def large_smart(self):
        if self.d[self.p] & 0x80:
            return self.u32() & 0x7fffffff
        return self.u16()

    def string(self):
        e = self.d.index(0, self.p)
        s = self.d[self.p:e].decode('cp1252', errors='replace')
        self.p = e + 1
        return s

    def bytes(self, n):
        v = self.d[self.p:self.p + n]; self.p += n; return v

    def remaining(self):
        return len(self.d) - self.p


class Cache:
    def __init__(self, root=CACHE_239):
        self.root = Path(root)
        self.dat = (self.root / 'main_file_cache.dat2').read_bytes()
        self._idx = {}
        self._ref = {}
        self._groups = {}

    def raw(self, archive, gid):
        if archive not in self._idx:
            self._idx[archive] = (self.root / f'main_file_cache.idx{archive}').read_bytes()
        idx = self._idx[archive]
        e = idx[gid * 6:gid * 6 + 6]
        if len(e) != 6:
            raise KeyError((archive, gid))
        length = int.from_bytes(e[:3], 'big')
        sector = int.from_bytes(e[3:], 'big')
        if not length or not sector:
            raise KeyError((archive, gid))
        out = bytearray()
        chunk = 0
        h = 10 if gid > 65535 else 8
        while len(out) < length:
            b = self.dat[sector * 520:sector * 520 + 520]
            assert int.from_bytes(b[:h - 6], 'big') == gid
            assert int.from_bytes(b[h - 6:h - 4], 'big') == chunk
            assert b[h - 1] == archive
            out += b[h:h + min(length - len(out), 520 - h)]
            sector = int.from_bytes(b[h - 4:h - 1], 'big')
            chunk += 1
        return bytes(out)

    @staticmethod
    def unpack(raw):
        kind = raw[0]
        n = int.from_bytes(raw[1:5], 'big')
        if kind == 0:
            return raw[5:5 + n]
        payload = raw[9:9 + n]
        return bz2.decompress(b'BZh1' + payload) if kind == 1 else zlib.decompress(payload, 31)

    def reference(self, archive):
        """Archive reference table: {group id: {'name': hash or None, 'files': [file ids]}} (cache.rs ArchiveIndex)."""
        if archive in self._ref:
            return self._ref[archive]
        r = Reader(self.unpack(self.raw(255, archive)))
        fmt = r.u8()
        if fmt >= 6:
            r.u32()
        flags = r.u8()
        count = r.large_smart if fmt == 7 else r.u16
        n = count()
        ids, gid = [], 0
        for _ in range(n):
            gid += count(); ids.append(gid)
        names = [r.u32() for _ in ids] if flags & 1 else [None] * n
        for _ in ids:
            r.u32()
        if flags & 4:
            for _ in ids:
                r.u32(); r.u32()
        for _ in ids:
            r.u32()
        fcounts = [count() for _ in ids]
        files = []
        for c in fcounts:
            fid, fl = 0, []
            for _ in range(c):
                fid += count(); fl.append(fid)
            files.append(fl)
        out = {g: {'name': nm, 'files': fl} for g, nm, fl in zip(ids, names, files)}
        self._ref[archive] = out
        return out

    def group(self, archive, gid):
        """{file id: bytes} for one group (cache.rs split_files)."""
        key = (archive, gid)
        if key in self._groups:
            return self._groups[key]
        fids = self.reference(archive)[gid]['files']
        data = self.unpack(self.raw(archive, gid))
        if len(fids) == 1:
            out = {fids[0]: data}
        else:
            chunks = data[-1]
            table = len(data) - 1 - chunks * len(fids) * 4
            r = Reader(data, table)
            sizes = [[0] * len(fids) for _ in range(chunks)]
            for c in range(chunks):
                acc = 0
                for f in range(len(fids)):
                    acc += r.s32(); sizes[c][f] = acc
            parts = [bytearray() for _ in fids]
            p = 0
            for c in range(chunks):
                for f in range(len(fids)):
                    parts[f] += data[p:p + sizes[c][f]]; p += sizes[c][f]
            out = {fid: bytes(b) for fid, b in zip(fids, parts)}
        if len(self._groups) > 4096:
            self._groups.clear()
        self._groups[key] = out
        return out

    def file(self, archive, gid, fid=0):
        return self.group(archive, gid)[fid]

    def find_name(self, archive, name):
        h = name_hash(name)
        for g, info in self.reference(archive).items():
            if info['name'] == h:
                return g
        return None


# ---------------------------------------------------------------- widgets (rust-client src/widget.rs)

LISTENER_NAMES = ['onLoad', 'onMouseOver', 'onMouseLeave', 'onTargetLeave', 'onTargetEnter', 'onVarTransmit',
                  'onInvTransmit', 'onStatTransmit', 'onTimer', 'onOp', 'onMouseRepeat', 'onClick', 'onClickRepeat',
                  'onRelease', 'onHold', 'onDrag', 'onDragComplete', 'onScrollWheel']
KIND_NAMES = {0: 'layer', 3: 'rectangle', 4: 'text', 5: 'graphic', 6: 'model', 8: 'tooltip(if1)', 9: 'line',
              11: 'layer(11)', 12: 'input', 1: 'if1-unused', 2: 'inventory(if1)', 7: 'textinventory(if1)'}


def _nullable(r):
    v = r.u16()
    return -1 if v == 65535 else v


def decode_widget(wid, data):
    r = Reader(data)
    if3 = data[0] == 255
    if if3:
        r.u8()
    w = dict(id=wid, group=wid >> 16, file=wid & 0xffff, if3=if3, kind=r.u8(), button=0, content=0,
             raw_position=[0, 0], raw_size=[0, 0], size_mode=[0, 0], position_mode=[0, 0], parent=None,
             hidden=False, mouse_over=-1, scroll_size=[0, 0], no_click_through=False, colors=[0, 0, 0, 0],
             filled=False, transparency=0, fill_mode=0, line_width=1, line_reversed=False, sprites=[-1, -1],
             sprite_angle=0, sprite_tiling=False, outline=0, sprite_shadow=0, flip_v=False, flip_h=False,
             models=[-1, -1], sequences=[-1, -1], model_offset=[0, 0], model_angles=[0, 0, 0], model_zoom=100,
             model_ortho=False, model_extra=0, model_scale=[0, 0], font=-1, text='', alternate_text='',
             text_alignment=[0, 0], line_height=0, text_shadow=False, flags=0, data_text='', actions=[],
             drag_zone=0, drag_threshold=0, scrollbar=False, spell_action='', spell_name='', button_text='Ok',
             listeners={}, triggers={}, cs1_comparisons=[], cs1=[])
    if not if3:
        w['button'] = r.u8()
    w['content'] = r.u16()
    w['raw_position'] = [r.s16(), r.s16()]
    w['raw_size'] = [r.u16(), r.s16() if (if3 and w['kind'] == 9) else r.u16()]
    if if3:
        w['size_mode'] = [r.s8(), r.s8()]
        w['position_mode'] = [r.s8(), r.s8()]
    else:
        w['transparency'] = r.u8()
    p = _nullable(r)
    w['parent'] = None if p == -1 else (wid & 0xffff0000) | p
    if if3:
        w['hidden'] = r.u8() == 1
        k = w['kind']
        if k == 0:
            w['scroll_size'] = [r.u16(), r.u16()]
            w['no_click_through'] = r.u8() == 1
        elif k == 5:
            w['sprites'][0] = r.s32()
            w['sprite_angle'] = r.u16()
            w['sprite_tiling'] = r.u8() == 1
            w['transparency'] = r.u8()
            w['outline'] = r.u8()
            w['sprite_shadow'] = r.u32()
            w['flip_v'] = r.u8() == 1
            w['flip_h'] = r.u8() == 1
        elif k == 6:
            w['models'][0] = r.s32()
            w['model_offset'] = [r.s16(), r.s16()]
            w['model_angles'] = [r.u16(), r.u16(), r.u16()]
            w['model_zoom'] = r.u16()
            w['sequences'][0] = _nullable(r)
            w['model_ortho'] = r.u8() == 1
            w['model_extra'] = r.u16()
            if w['size_mode'][0] != 0:
                w['model_scale'][0] = r.u16()
            if w['size_mode'][1] != 0:
                w['model_scale'][1] = r.u16()
        elif k == 4:
            w['font'] = _nullable(r)
            w['text'] = r.string()
            w['line_height'] = r.u8()
            w['text_alignment'] = [r.u8(), r.u8()]
            w['text_shadow'] = r.u8() == 1
            w['colors'][0] = r.u32()
        elif k == 3:
            w['colors'][0] = r.u32()
            w['filled'] = r.u8() == 1
            w['transparency'] = r.u8()
        elif k == 9:
            w['line_width'] = r.u8()
            w['colors'][0] = r.u32()
            w['line_reversed'] = r.u8() == 1
        w['flags'] = r.u24()
        w['data_text'] = r.string()
        w['actions'] = [r.string() for _ in range(r.u8())]
        w['drag_zone'] = r.u8()
        w['drag_threshold'] = r.u8()
        w['scrollbar'] = r.u8() == 1
        w['spell_action'] = r.string()
        for name in LISTENER_NAMES:
            n = r.u8()
            if n:
                args = []
                for _ in range(n):
                    t = r.u8()
                    args.append(r.s32() if t == 0 else r.string() if t == 1 else None)
                w['listeners'][name] = args
        for name in ('varTransmit', 'invTransmit', 'statTransmit'):
            n = r.u8()
            if n:
                w['triggers'][name] = [r.s32() for _ in range(n)]
    else:
        w['mouse_over'] = _nullable(r)
        for _ in range(r.u8()):
            w['cs1_comparisons'].append([r.u8(), r.u16()])
        for _ in range(r.u8()):
            w['cs1'].append([_nullable(r) for _ in range(r.u16())])
        k = w['kind']
        if k == 0:
            w['scroll_size'][1] = r.u16()
            w['hidden'] = r.u8() == 1
        if k == 1:
            r.u16(); r.u8()
        if k == 3:
            w['filled'] = r.u8() == 1
        if k in (4, 1):
            w['text_alignment'] = [r.u8(), r.u8()]
            w['line_height'] = r.u8()
            w['font'] = _nullable(r)
            w['text_shadow'] = r.u8() == 1
        if k == 4:
            w['text'] = r.string()
            w['alternate_text'] = r.string()
        if k in (1, 3, 4):
            w['colors'][0] = r.u32()
        if k in (3, 4):
            w['colors'][1] = r.u32(); w['colors'][2] = r.u32(); w['colors'][3] = r.u32()
        if k == 5:
            w['sprites'] = [r.s32(), r.s32()]
        if k == 6:
            w['models'] = [r.s32(), r.s32()]
            w['sequences'] = [_nullable(r), _nullable(r)]
            w['model_zoom'] = r.u16()
            w['model_angles'][0] = r.u16(); w['model_angles'][1] = r.u16()
        if k == 8:
            w['text'] = r.string()
        if w['button'] == 2:
            w['spell_action'] = r.string(); w['spell_name'] = r.string()
            w['flags'] |= (r.u16() & 63) << 11
        if w['button'] in (1, 4, 5, 6):
            t = r.string()
            w['button_text'] = t or {1: 'Ok', 4: 'Select', 5: 'Select'}.get(w['button'], 'Continue')
        if w['button'] in (1, 4, 5):
            w['flags'] |= 4194304
        if w['button'] == 6:
            w['flags'] |= 1
    w['trailing'] = r.remaining()
    return w


def load_interface(cache, gid):
    files = cache.group(3, gid)
    return [decode_widget((gid << 16) | f, files[f]) for f in sorted(files)]


# ---------------------------------------------------------------- sprites (sprite.rs)

class SpriteSheet:
    def __init__(self, data):
        count = int.from_bytes(data[-2:], 'big')
        meta = len(data) - 7 - count * 8
        r = Reader(data, meta)
        self.width, self.height = r.u16(), r.u16()
        plen = r.u8() + 1
        xs = [r.u16() for _ in range(count)]
        ys = [r.u16() for _ in range(count)]
        ws = [r.u16() for _ in range(count)]
        hs = [r.u16() for _ in range(count)]
        ppos = meta - (plen - 1) * 3
        p = Reader(data, ppos)
        self.palette = [0] + [max(p.u24(), 1) for _ in range(plen - 1)]
        px = Reader(data[:ppos])
        self.frames = []
        for i in range(count):
            w, h = ws[i], hs[i]
            flags = px.u8()
            raw = px.bytes(w * h)
            if flags & 1:
                idx = bytearray(w * h)
                for x in range(w):
                    for y in range(h):
                        idx[x + y * w] = raw[y + x * h]
                raw = bytes(idx)
            if flags & 2:
                px.bytes(w * h)  # alpha plane: the 239 client skips it
            self.frames.append(dict(x=xs[i], y=ys[i], w=w, h=h, idx=bytes(raw), alpha_plane=bool(flags & 2)))
        assert px.remaining() == 0


_sheet_cache = {}


def sprite_sheet(cache, gid):
    key = (id(cache), gid)
    if key not in _sheet_cache:
        _sheet_cache[key] = SpriteSheet(cache.file(8, gid, 0))
    return _sheet_cache[key]


# ---------------------------------------------------------------- fonts (font.rs)

class Metrics:
    def __init__(self, data):
        r = Reader(data)
        self.advances = list(r.bytes(256))
        self.kerning = None
        if len(data) == 257:
            self.ascent = r.u8()
            return
        lengths = list(r.bytes(256))
        offsets = list(r.bytes(256))
        profiles = []
        for _ in range(2):
            plane = []
            for n in lengths:
                v, prof = 0, []
                for _ in range(n):
                    v = (v + r.u8()) & 0xff
                    prof.append(v - 256 if v > 127 else v)
                plane.append(prof)
            profiles.append(plane)
        k = [0] * 65536
        for a in range(256):
            if a in (32, 160):
                continue
            for b in range(256):
                if b in (32, 160):
                    continue
                s = max(offsets[a], offsets[b])
                e = min(offsets[a] + lengths[a], offsets[b] + lengths[b])
                ov = min(self.advances[a], self.advances[b])
                for row in range(s, e):
                    ov = min(ov, profiles[1][a][row - offsets[a]] + profiles[0][b][row - offsets[b]])
                v = (-ov) & 0xff
                k[a * 256 + b] = v - 256 if v > 127 else v
        self.kerning = k
        self.ascent = offsets[32] + lengths[32]

    def pair(self, a, b):
        if a is None or self.kerning is None:
            return 0
        return self.kerning[a * 256 + b]


class Font:
    def __init__(self, cache, fid):
        self.id = fid
        self.metrics = Metrics(cache.file(13, fid, 0))
        self.glyphs = SpriteSheet(cache.file(8, fid, 0))
        assert len(self.glyphs.frames) == 256
        tops = [g['y'] for g in self.glyphs.frames if g['h']]
        self.max_ascent = self.metrics.ascent - min(tops)
        self.max_descent = max(g['y'] + g['h'] for g in self.glyphs.frames) - self.metrics.ascent


_font_cache = {}


def font(cache, fid):
    key = (id(cache), fid)
    if key not in _font_cache:
        _font_cache[key] = Font(cache, fid)
    return _font_cache[key]


def font_names(cache):
    """Font group id -> name, for the font names known to the client."""
    names = ['p11_full', 'p12_full', 'b12_full', 'q8_full', 'verdana_11pt_regular', 'verdana_11pt_bold',
             'verdana_13pt_regular', 'verdana_13pt_bold', 'verdana_15pt_regular', 'verdana_15pt_bold',
             'quill8', 'quill_caps_large', 'lunar_alphabet', 'lunar_alphabet_lrg', 'b12_bold_full',
             'p11', 'p12', 'b12', 'q8', 'quill_oblique_large']
    out = {}
    for n in names:
        g = cache.find_name(13, n)
        if g is not None:
            out[g] = n
    return out
