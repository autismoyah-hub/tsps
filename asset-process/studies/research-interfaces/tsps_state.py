"""tsps_state.py DRIVE_LOG EVAL_NAME GROUP OUT.jsonl [--cache 242]

Build an iface.py state (laid-out tree in draw order) for one group from the TSPS client's own runtime values,
read in the running client by a drive.cjs eval that returned rows
[component, type, absX, absY, width, height, hidden, text, textColour, transparency, spriteId].
Everything else (fonts, alignment, shadow, tiling, flags) comes from the cache definition of that revision.
This lets iface.py draw exactly the state TSPS drew, for components the scripts changed at run time."""
import sys, json, re, argparse
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifcache as C


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('log'); ap.add_argument('name'); ap.add_argument('group', type=int); ap.add_argument('out')
    ap.add_argument('--cache', default='242')
    a = ap.parse_args()
    rows = None
    for line in Path(a.log).read_text().splitlines():
        m = re.search(r'eval %s -> (.*)$' % re.escape(a.name), line)
        if m:
            rows = json.loads(m.group(1))
    cache = C.Cache(C.CACHE_242 if a.cache == '242' else C.CACHE_239)
    defs = {w['file']: w for w in C.load_interface(cache, a.group)}
    rt = {r[0]: r for r in rows}
    out = []
    clips = {}
    for f in sorted(defs):  # cache order is draw order for a static tree
        w = dict(defs[f]); r = rt.get(f)
        if r is None or r[2] is None:
            continue
        _, kind, x, y, wd, ht, hidden, text, colour, transp, sprite = r
        par = w['parent']
        pclip = clips.get(par, [0, 0, 1 << 20, 1 << 20]) if par is not None else [0, 0, 1 << 20, 1 << 20]
        clip = [max(pclip[0], x), max(pclip[1], y), min(pclip[2], x + wd), min(pclip[3], y + ht)]
        clips[w['id']] = clip
        phidden = any(o['id'] == par and o['hidden'] for o in out)
        w.update(abs=[x, y], size=[wd, ht], clip=clip, hidden=bool(hidden) or phidden,
                 parent=-1 if par is None else par)
        if text is not None:
            w['text'] = text
        if colour is not None:
            w['colors'] = [colour] + w['colors'][1:]
        if transp is not None:
            w['transparency'] = transp
        if sprite is not None:
            w['sprites'] = [sprite, w['sprites'][1]]
        w['logical'] = w['id']; w['child_index'] = -1; w['item'] = -1; w['count'] = 0; w['condition'] = False
        w.pop('listeners', None); w.pop('triggers', None); w.pop('cs1', None)
        out.append(w)
    Path(a.out).write_text('\n'.join(json.dumps(w) for w in out) + '\n')
    print(len(out), 'components')


if __name__ == '__main__':
    main()
