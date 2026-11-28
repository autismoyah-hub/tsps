"""cs2.py: minimal client-script reader (header, name, instructions) after rust-client src/script.rs. Read only."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import ifcache as C


def decode(data):
    end = len(data) - 2
    sw = int.from_bytes(data[end:], 'big')
    foot = end - sw - 16
    r = C.Reader(data, foot)
    count = r.u32()
    li, ls, ll, ia, sa, la = [r.u16() for _ in range(6)]
    r = C.Reader(data[:foot])
    name = r.string()
    ins = []
    for _ in range(count):
        op = r.u16()
        if op == 3:
            v = r.string()
        elif op == 61:
            v = int.from_bytes(r.bytes(8), 'big')
        elif op in (21, 38, 39, 62, 63):
            v = r.u8()
        elif op <= 99:
            v = r.s32()
        else:
            v = r.u8()
        ins.append((op, v))
    return dict(name=name or None, int_args=ia, str_args=sa, instructions=ins)


def script(cache, sid):
    return decode(cache.file(12, sid))


if __name__ == '__main__':
    c = C.Cache()
    for s in sys.argv[1:]:
        d = script(c, int(s))
        ints = [v for op, v in d['instructions'] if op == 0]
        strs = [v for op, v in d['instructions'] if op == 3]
        calls = [v for op, v in d['instructions'] if op == 40]
        print(s, d['name'], 'args', d['int_args'], d['str_args'], 'n', len(d['instructions']), 'ints', ints[:40], 'strs', strs[:8], 'calls', calls[:20])
