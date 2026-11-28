"""Print the raw ambient and contrast bytes (the values lineup.py / gate.py take) of real object definitions.

usage: objlight.py ID [ID ...]
Decoder follows client/rust-client/src/object.rs (rev 239 cache). Read-only; private output.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys
sys.dont_write_bytecode = True
sys.path.insert(0, _AP.fix('<ROOT>/tools'))
import scene


class R:
    def __init__(self, b):
        self.b, self.p = b, 0

    def u8(self):
        v = self.b[self.p]; self.p += 1; return v

    def u16(self):
        v = int.from_bytes(self.b[self.p:self.p + 2], 'big'); self.p += 2; return v

    def u24(self):
        v = int.from_bytes(self.b[self.p:self.p + 3], 'big'); self.p += 3; return v

    def u32(self):
        v = int.from_bytes(self.b[self.p:self.p + 4], 'big'); self.p += 4; return v

    def string(self):
        e = self.b.index(0, self.p); s = self.b[self.p:e]; self.p = e + 1; return s.decode('latin-1')


def decode(data, local=False):
    r = R(data); d = dict(ambient=0, contrast=0, name=None, models=[], recolor=[], scale={})
    while True:
        op = r.u8()
        if op == 0:
            break
        elif local and op in (1, 5):
            n = r.u8()
            for _ in range(n):
                d['models'].append(r.u16())
                if op == 1: r.u8()
        elif op in (6, 7):
            n = r.u8()
            for _ in range(n):
                d['models'].append(r.u32())
                if op == 6: r.u8()
        elif op == 2 or (local and op == 3):
            value = r.string()
            if op == 2: d['name'] = value
        elif op in (14, 15, 19, 28, 69, 75, 91, 95, 96): r.u8()
        elif op == 29: d['ambient'] = r.u8() - (256 if data[r.p - 1] > 127 else 0)
        elif op == 39: d['contrast'] = r.u8() - (256 if data[r.p - 1] > 127 else 0)
        elif 30 <= op <= 34: r.string()
        elif op == 100: r.u8(); r.u8(); r.string()
        elif op in (101, 102):
            r.u8(); r.u16(); r.u16()
            if op == 102: r.u16()
            r.u32(); r.u32(); r.string()
        elif op in (40, 41):
            n = r.u8(); pairs = [(r.u16(), r.u16()) for _ in range(n)]
            if op == 40: d['recolor'] = pairs
        elif op in (24, 61, 68, 82) or (local and op == 200): r.u16()
        elif 65 <= op <= 67: d['scale'][op - 65] = r.u16()
        elif 70 <= op <= 72: r.u16()
        elif op in (77, 92):
            r.u16(); r.u16()
            if op == 92: r.u16()
            n = r.u8()
            for _ in range(n + 1): r.u16()
        elif op == 78: r.u16(); r.u8(); r.u8()
        elif op == 79:
            r.u16(); r.u16(); r.u8(); r.u8(); n = r.u8()
            for _ in range(n): r.u16()
        elif op == 81: r.u8()
        elif op == 93: r.u8(); r.u16(); r.u8(); r.u16()
        elif op == 249:
            for _ in range(r.u8()):
                k = r.u8(); r.u24()
                if k == 1: r.string()
                elif k == 2: r.p += 8
                else: r.u32()
        # all other opcodes carry no data
    return d


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('ids', type=int, nargs='+')
    for i in parser.parse_args().ids:
        d = decode(scene.object_def(i))
        print(i, repr(d['name']), 'ambient', d['ambient'], 'contrast', d['contrast'], 'models', d['models'], 'recolours', len(d['recolor']))
