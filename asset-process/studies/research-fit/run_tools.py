"""run_tools.py: step 5. Run fit.py on the five references and our seven helms, and definition.py on the five reference
sets and our two sets (with and without helm). Writes tool-table.txt and tool-runs/*.txt (each tool's full print)."""
import sys, pathlib, subprocess, json
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import lib as L
HERE = pathlib.Path(__file__).resolve().parent
RUNS = HERE / 'tool-runs'; RUNS.mkdir(exist_ok=True)
lines = []


def run(tool, args, tag):
    r = subprocess.run([L.PY, '-I', str(HERE / tool), *args, '--json', str(RUNS / f'{tag}.json')], capture_output=True, text=True)
    (RUNS / f'{tag}.txt').write_text(r.stdout + r.stderr)
    return r.returncode, r.stdout, json.load(open(RUNS / f'{tag}.json'))


import fit as FT
keys = [k for k, c, w in FT.LINES]
core = {k for k, c, w in FT.LINES if c}
R = FT.ranges()
lines.append('FIT (fit.py). * = fit line. Values outside the five references are marked !')
lines.append(f"{'helm':12}{'result':>7}{'*fails':>7}{'size':>5} " + ''.join(f"{('*' if k in core else '') + k[:9]:>11}" for k in keys))
lines.append(f"{'the five lo':12}{'':>19} " + ''.join(f'{R[k][0]:>11g}' for k in keys))
lines.append(f"{'the five hi':12}{'':>19} " + ''.join(f'{R[k][1]:>11g}' for k in keys))
todo = [(r[0], [f'item:{r[1]}']) for r in L.REFS] + [(r[0], [r[1], r[2], '--body', r[4]]) for r in L.OURS]
for key, args in todo:
    code, out, d = run('fit.py', args, f'fit-{key}')
    cf = sum(1 for k in keys if k in core and d.get(k) is not None and not (R[k][0] <= d[k] <= R[k][1]))
    sf = sum(1 for k in keys if k not in core and d.get(k) is not None and not (R[k][0] <= d[k] <= R[k][1]))
    def cell(k):
        v = d.get(k)
        if v is None: return f"{'-':>11}"
        return f"{(str(v) + ('!' if not (R[k][0] <= v <= R[k][1]) else '')):>11}"
    lines.append(f"{key:12}{('PASS' if code == 0 else 'FAIL'):>7}{cf:>7}{sf:>5} " + ''.join(cell(k) for k in keys))
    print(lines[-1], flush=True)

import definition as DF
dk = [k for k, s, w in DF.LINES]
lines.append('')
lines.append('DEFINITION (definition.py): median of 34 pictures at zoom 300. ! = below the least divided reference of its value class')
lines.append(f"{'set':20}{'class':>6}{'result':>7}{'fails':>6} " + ''.join(f'{k:>11}' for k in dk))
sets = [(r[0], L.ref_outfit(r)) for r in L.REFS] + [('earlier', L.HELM_V2A + '+' + L.SET_V2), ('redesigned', L.HELM_V3A + '+' + L.SET_V3)]
sets += [(r[0] + '-nohelm', L.ref_outfit(r, helm=False)) for r in L.REFS] + [('earlier-nohelm', L.SET_V2), ('redesigned-nohelm', L.SET_V3)]
for key, entry in sets:
    code, out, s = run('definition.py', [entry], f'def-{key}')
    marks = {}
    for ln in out.splitlines():
        p = ln.split()
        if p and p[0] in dk: marks[p[0]] = 'FAIL' in ln
    cls = 'dark' if s['p50'] < 35 else 'light'
    lines.append(f"{key:20}{cls:>6}{('PASS' if code == 0 else 'FAIL'):>7}{sum(marks.values()):>6} " + ''.join(f"{(str(s[k]) + ('!' if marks.get(k) else '')):>11}" for k in dk))
    print(lines[-1], flush=True)
(HERE / 'tool-table.txt').write_text('\n'.join(lines) + '\n')
print('\n'.join(lines))
