"""Count differing pixels between column 1 and each other column over many views and frames.

usage: sweep.py OUTDIR "ENTRY,ENTRY,..." --rows R1,R2.. [--yaws 0,128,...] [--pitches 0,220] [--zoom N] [--cell WxH]
Column 1 is the reference. Prints, per other column, the total and the worst cell, and writes every sheet to OUTDIR.
"""
import sys, subprocess, pathlib, re
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
out = pathlib.Path(a[1]); entries = a[2]; n = len(entries.split(','))
yaws = opt('--yaws', ','.join(str(y) for y in range(0, 2048, 128))).split(',')
pitches = opt('--pitches', '0,220').split(',')
tot = {k: 0 for k in range(2, n + 1)}; worst = {k: (0, '') for k in range(2, n + 1)}; cells = 0
py = sys.executable
for p in pitches:
    for y in yaws:
        f = out / f'p{p}-y{y}.png'
        r = subprocess.run([py, '-I', str(pathlib.Path(__file__).parent / 'npc.py'), str(f), entries, '--rows', opt('--rows', 'bind'), '--yaw', y, '--pitch', p,
                            '--zoom', opt('--zoom', '330'), '--cell', opt('--cell', '420x560'), '--diff', ','.join(f'1:{k}' for k in range(2, n + 1)), '--quiet'],
                           capture_output=True, text=True)
        if r.returncode: raise SystemExit(r.stdout + r.stderr)
        for m in re.finditer(r'diff col 1 vs col (\d+) row (\S+): (\d+) of', r.stdout):
            k, row, d = int(m[1]), m[2], int(m[3]); tot[k] += d
            if d > worst[k][0]: worst[k] = (d, f'pitch {p} yaw {y} row {row}')
            if k == 2: cells += 1
for k in tot: print(f'col {k} against col 1: {tot[k]} differing pixels over {cells} cells (mean {tot[k] / max(1, cells):.0f}); worst {worst[k][0]} at {worst[k][1]}')
