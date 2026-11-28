"""fxlib.py: shared readers for the effects research (spot animations, their models and sequences, gameval names).

Private to this machine; reads the revision-239 cache, never writes it.
  spot definitions   archive 2, group 13, file = id          (client: src/spot_animation.rs)
  names              archive 24 (gamevals), group 8 = SPOTANIM, 7 = SEQ, 6 = LOC, 1 = NPC; file = id, content = the name
  models             archive 7                                (tools/osrscache.py decode_model, the client's port)
  sequences, frames  research-animation/animlib.py            (proven byte-exact there)
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
sys.path.insert(0, str(ROOT / 'builds' / 'research-animation'))
import osrscache as oc
import animlib as al

CACHE = oc.CACHE
PY = _AP.fix('<PYTHON>')
EXE = _AP.bindir()
RENDER = ROOT / 'render'


def decode_spot(data):
    """Port of spot_animation.rs Definition::decode (local = true). 'ops' keeps the stored order."""
    r = al.Rd(data)
    d = dict(model=0, sequence=-1, scale=[128, 128], rotation=0, ambient=0, contrast=0, flag=True, recolor=[], retexture=[], ops=[])
    while True:
        op = r.u8()
        if op == 0: break
        d['ops'].append(op)
        if op == 1: d['model'] = r.u16()
        elif op == 3: d['model'] = r.u32()
        elif op == 2: d['sequence'] = r.u16()
        elif op == 4: d['scale'][0] = r.u16()
        elif op == 5: d['scale'][1] = r.u16()
        elif op == 6: d['rotation'] = r.u16()
        elif op == 7: d['ambient'] = r.u8()
        elif op == 8: d['contrast'] = r.u8()
        elif op == 10: d['flag'] = False
        elif op in (40, 41):
            pairs = [(r.u16(), r.u16()) for _ in range(r.u8())]
            d['recolor' if op == 40 else 'retexture'] = pairs
        else: raise ValueError(f'spot opcode {op}')
    assert r.left() == 0, 'spot trailing bytes'
    return d


def encode_spot(d):
    """Inverse of decode_spot. With no 'ops' the opcodes are written in ascending order for non-default fields
    (model always; a model id above 65535 uses opcode 3)."""
    w = al.Wr()
    ops = d.get('ops')
    if ops is None:
        ops = [1 if d['model'] <= 65535 else 3]
        if d.get('sequence', -1) != -1: ops.append(2)
        if d.get('scale', [128, 128])[0] != 128: ops.append(4)
        if d.get('scale', [128, 128])[1] != 128: ops.append(5)
        if d.get('rotation', 0): ops.append(6)
        if d.get('ambient', 0): ops.append(7)
        if d.get('contrast', 0): ops.append(8)
        if not d.get('flag', True): ops.append(10)
        if d.get('recolor'): ops.append(40)
        if d.get('retexture'): ops.append(41)
    for op in ops:
        w.u8(op)
        if op == 1: w.u16(d['model'])
        elif op == 3: w.u32(d['model'])
        elif op == 2: w.u16(d['sequence'])
        elif op == 4: w.u16(d['scale'][0])
        elif op == 5: w.u16(d['scale'][1])
        elif op == 6: w.u16(d['rotation'])
        elif op == 7: w.u8(d['ambient'])
        elif op == 8: w.u8(d['contrast'])
        elif op in (40, 41):
            pairs = d['recolor' if op == 40 else 'retexture']
            w.u8(len(pairs))
            for a, b in pairs: w.u16(a); w.u16(b)
    w.u8(0)
    return w.bytes()


_spots = None


def spots(c=None):
    """{id: raw bytes} of every spot animation definition."""
    global _spots
    if c is not None: return c.files(2, 13)
    if _spots is None: _spots = al.cache().files(2, 13)
    return _spots


_names = {}


def names(kind):
    """{id: name} from the gameval archive. kind: 8 spot animations, 7 sequences, 6 objects, 1 NPCs."""
    if kind not in _names:
        out = {}
        for fid, data in al.cache().files(24, kind).items():
            out[fid] = bytes(data).split(b'\0')[0].decode('latin1')
        _names[kind] = out
    return _names[kind]


def spot_model(d):
    """The decoded model of a spot definition with its recolours applied (dict as tools/osrscache.decode_model)."""
    m = oc.model(d['model'])
    if d['recolor']:
        rc = dict(d['recolor'])
        m['colors'] = [rc.get(c, c) for c in m['colors']]
    return m


def hsl(c): return (c >> 10) & 63, (c >> 7) & 7, c & 127


def jdump(path, obj): pathlib.Path(path).write_text(json.dumps(obj, indent=1), encoding='utf-8')
