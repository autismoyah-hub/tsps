"""Teacher-only gate: score a rebuild in our revision-242 format against the build-951 original converted.

usage: rs3gate.py rebuild KIND ID OURS.bin [--out DIR]     exit 0 PASS, 1 BORDERLINE, 2 FAIL
Same checks and thresholds as tools-hd-gate/hdgate.py (geometry, outline, material area and placement,
appearance against the conversion ceiling). Material lines are labelled with build-951 material ids.
"""
import sys, argparse, subprocess
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import rs3asset
import hdgate, hdsheet
rs3asset.install(hdgate, hdsheet)
Reference = hdgate.Reference

if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('command', choices=['rebuild'])
    p.add_argument('kind', choices=rs3asset.KINDS)
    p.add_argument('id', type=int)
    p.add_argument('ours', type=Path)
    p.add_argument('--out', type=Path)
    a = p.parse_args()
    ref = Reference(rs3asset.REV, a.kind, a.id)
    try:
        q = ref.score(a.ours, a.out)
    except (ValueError, KeyError, IndexError, subprocess.CalledProcessError) as e:
        print('candidate decode or render |', e, '| FAIL\nVERDICT FAIL')
        raise SystemExit(2)
    # Region labels name build-951 material ids; the hdgate code labels them 'HD'.
    print('\n'.join(q.lines()).replace('HD ', 'RS3 '))
    raise SystemExit(q.severity)
