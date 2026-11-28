"""Score our rooms on every measure beside the real ranges for their kind; mark what falls outside the real p10..p90.

usage: score.py OUT.md      (reads ours/*.csv through roomcheck.load; kinds: armoury, laboratory, library)
"""
import sys, json, pathlib
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import roommeasure as M
import roomcheck as C

ROOMS = [('armoury as composed (Usable)', 'ours/armoury-orig.csv', 'armoury'),
         ('observatory (Strong)', 'ours/observatory.csv', 'laboratory'),
         ("scribes' hall, plan p1 (Strong)", 'ours/scribes-p1.csv', 'library'),
         ("scribes' hall, plan p2", 'ours/scribes-p2.csv', 'library')]
extra = sys.argv[2:]
for i in range(0, len(extra), 3): ROOMS.append((extra[i], extra[i + 1], extra[i + 2]))
KEYS = ['n_furn', 'furn_per10', 'per10', 'occ_share', 'wall_share', 'wall_lined', 'bare_run', 'n_decor', 'decor_per10w',
        'decor_edge_share', 'floor_cover', 'grouped_share', 'n_groups', 'mean_group', 'max_repeat', 'rows', 'variety', 'variety_all',
        'sym_axis', 'sym_door', 'sym_id', 'faces_door', 'centre_occ', 'centre_floor', 'corner_use', 'reach1', 'reach2', 'free_share',
        'n_lights', 'n_clutter']
ranges = json.loads((HERE / 'ranges.json').read_text())
study = json.loads((HERE / 'study.json').read_text())


def pct_rank(v, vals):
    vals = [x for x in vals if x is not None]
    if v is None or not vals: return None
    return (sum(1 for x in vals if x < v) + 0.5 * sum(1 for x in vals if x == v)) / len(vals)


lines = ['# Our rooms on every measure, beside the real rooms of their kind', '',
         'Real range shown as p10 .. p25 / median / p75 .. p90 of the study rooms of the kind (all 96 when the kind has fewer than 8).',
         '`pr` is the share of those real rooms that score below this room (percentile rank). **LOW** / **HIGH**: outside p10..p90.', '']
summary = []
for title, path, kind in ROOMS:
    room = C.load(HERE / path)
    m = M.measures(room)
    grp = kind if ranges[kind]['n'] >= 8 else 'all'
    vals = [r['m'] for r in study if (r['kind'] == kind or grp == 'all')]
    lines += [f'## {title}: kind {kind}, compared with "{grp}" (n={ranges[grp]["n"]}), floor {m["area"]} tiles', '',
              '| measure | this room | real p10 .. p25 / med / p75 .. p90 | pr | |', '|---|---|---|---|---|']
    out = []
    for k in KEYS:
        v = m.get(k); s = ranges[grp].get(k)
        if s is None: continue
        pr = pct_rank(v, [x.get(k) for x in vals])
        flag = ''
        if v is not None and v < s['p10'] - 1e-9: flag = '**LOW**'
        if v is not None and v > s['p90'] + 1e-9: flag = '**HIGH**'
        if flag: out.append(f'{k} {flag.strip("*").lower()}')
        f = lambda x: f'{x:.2f}' if abs(x) < 10 else f'{x:.0f}'
        lines.append(f"| {k} | {'-' if v is None else f(v)} | {f(s['p10'])} .. {f(s['p25'])} / {f(s['med'])} / {f(s['p75'])} .. {f(s['p90'])} | "
                     f"{'-' if pr is None else f'{pr:.2f}'} | {flag} |")
    lines.append('')
    lines.append('Outside p10..p90: ' + (', '.join(out) if out else 'none'))
    lines.append('')
    summary.append((title, len(out), out))
lines += ['## Count of measures outside the real p10..p90', ''] + [f'- {t}: {n} ({", ".join(o)})' for t, n, o in summary]
pathlib.Path(sys.argv[1]).write_text('\n'.join(lines) + '\n', encoding='utf-8')
for t, n, o in summary: print(t, n, o)
