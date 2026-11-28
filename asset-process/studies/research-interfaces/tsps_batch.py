"""tsps_batch.py CAPTURE_DIR: compare every TSPS capture in a drive.cjs session (shots g<group>.png with eval
pos-<group> lines) against iface.py's drawing of state/<group>.jsonl. Writes CAPTURE_DIR/compare.json."""
import sys, json, re, subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
PY = sys.executable


def main(cap):
    cap = Path(cap)
    pos = {}
    for line in (cap / 'drive.log').read_text().splitlines():
        m = re.search(r'eval pos-(\d+) -> (.*)$', line)
        if m:
            pos[int(m.group(1))] = json.loads(m.group(2))
    res = []
    (cap / 'cmp').mkdir(exist_ok=True)
    for g, roots in sorted(pos.items()):
        cand = [r for r in roots if r[1] is not None and r[3] and r[4]]
        if not cand:
            res.append(dict(group=g, error='no placed root')); continue
        i, x, y, w, h, scale = cand[0]
        st = HERE / "state_modal" / f"{g}.jsonl"
        p = subprocess.run([PY, '-I', str(HERE / 'tsps_compare.py'), str(cap / f'g{g}.png'), str(x), str(y), str(st),
                            f'{g}.{i}', str(cap / 'cmp' / f'{g}')], capture_output=True, text=True)
        try:
            r = json.loads(p.stdout.strip().splitlines()[-1])
        except Exception:
            r = dict(error=(p.stderr or p.stdout)[-300:])
        r.update(group=g, tsps_pos=[x, y, w, h], scale=scale)
        res.append(r)
        print(json.dumps(r), flush=True)
    (cap / 'compare.json').write_text(json.dumps(res, indent=0))


if __name__ == '__main__':
    main(sys.argv[1])
