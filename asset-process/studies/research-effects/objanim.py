"""objanim.py: scenery objects that animate (object definition opcode 24 names a sequence), the other half of what a
fight shows that is not a spot animation: persistent hazards and arena pieces. usage: objanim.py [REGEX]
-> data/objanim.txt (counts; with REGEX, the matching names). Walks definitions with tools/objlight.py's opcode table."""
import sys, re, pathlib, collections
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from fxlib import *
import objlight


def animation(data):
    """opcode 24's value, by the same walk as objlight.decode (which skips it)."""
    R = objlight.R; r = R(data); anim = None; n_models = 0
    orig_u16 = r.u16
    while True:
        op = r.u8()
        if op == 0: break
        if op == 24: anim = r.u16(); continue
        # delegate one opcode to objlight's table by decoding a two-byte slice is not possible; repeat its table
        if op in (6, 7):
            n = r.u8()
            for _ in range(n):
                r.u32(); n_models += 1
                if op == 6: r.u8()
        elif op in (1, 5):
            n = r.u8()
            for _ in range(n):
                r.u16(); n_models += 1
                if op == 1: r.u8()
        elif op in (2, 3): r.string()
        elif op in (14, 15, 19, 28, 69, 75, 91, 95, 96, 29, 39, 81): r.u8()
        elif 30 <= op <= 34: r.string()
        elif op == 100: r.u8(); r.u8(); r.string()
        elif op in (101, 102):
            r.u8(); r.u16(); r.u16()
            if op == 102: r.u16()
            r.u32(); r.u32(); r.string()
        elif op in (40, 41):
            for _ in range(r.u8()): r.u16(); r.u16()
        elif op in (61, 68, 82, 200) or 65 <= op <= 67 or 70 <= op <= 72: r.u16()
        elif op in (77, 92):
            r.u16(); r.u16()
            if op == 92: r.u16()
            for _ in range(r.u8() + 1): r.u16()
        elif op == 78: r.u16(); r.u8(); r.u8()
        elif op == 79:
            r.u16(); r.u16(); r.u8(); r.u8()
            for _ in range(r.u8()): r.u16()
        elif op == 93: r.u8(); r.u16(); r.u8(); r.u16()
        elif op == 249:
            for _ in range(r.u8()):
                k = r.u8(); r.u24()
                if k == 1: r.string()
                elif k == 2: r.p += 8
                else: r.u32()
    return anim, n_models


defs = al.cache().files(2, 6); nm = names(6); sn = names(7)
rows = []; bad = 0
for oid, data in defs.items():
    try: a, n = animation(data)
    except Exception: bad += 1; continue
    if a is not None and a != 65535: rows.append((oid, nm.get(oid, ''), a, sn.get(a, '')))
out = [f'object definitions {len(defs)}; not walked {bad}; with a sequence (opcode 24): {len(rows)} ({100 * len(rows) / len(defs):.1f}%); distinct sequences {len(set(r[2] for r in rows))}']
tok = collections.Counter(t for r in rows for t in re.split(r'[_0-9]+', r[1]) if len(t) > 3)
out.append('commonest name words: ' + ' '.join(f'{t}:{n}' for t, n in tok.most_common(60)))
pat = sys.argv[1] if len(sys.argv) > 1 else r'nex|nightmare|verzik|tob_|olm|raids|toa_|warden|doom|yama|hydra|vorkath|zulrah|colosseum|duke|whisperer|leviathan|vardorvis'
hit = [r for r in rows if re.search(pat, r[1])]
out.append(f'matching /{pat}/: {len(hit)}')
out += [f'  {o} {n}: sequence {a} {s}' for o, n, a, s in hit[:120]]
(HERE / 'data' / 'objanim.txt').write_text('\n'.join(out) + '\n', encoding='utf-8')
print('\n'.join(out[:3])); print('\n'.join(out[3:60]))
