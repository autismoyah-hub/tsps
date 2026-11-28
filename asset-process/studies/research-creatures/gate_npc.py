"""Acceptance gate for a creature rebuild: PASS, BORDERLINE or FAIL.

usage: gate_npc.py rebuild-npc REAL_NPC_ID OURS.json [--json FILE] [--out DIR] [--per N] [--attack SEQ] [--death SEQ]

Prints one line per check: name | value | threshold | verdict, then VERDICT. Exit code 0 PASS, 1 BORDERLINE, 2 FAIL.
ALL THRESHOLDS ARE PROVISIONAL (2026-11-27). They are anchored at one end only: the ceiling (a real creature
re-encoded through our encoder scores overlap 1.000, 0 differing pixels, colour distance 0.00 on three creatures) and
checked for direction on deliberately degraded copies (see CREATURES.md, "The rebuild gate"). The pass marks for
size, faces, colours, colour distance and outline are the furniture and worn-equipment gate's (tools/gate.py), which
were set from rebuilds the owner could not tell from the real thing; the motion marks have no such calibration yet.
They must be reset after the first three creature rebuilds have been through a blind line-up.
"""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import compare_npc
LEVELS = ['PASS', 'BORDERLINE', 'FAIL']
class Checks:
    def __init__(self): self.rows = []
    def record(self, name, value, threshold, verdict):
        self.rows.append(dict(name=name, value=value, threshold=threshold, verdict=verdict)); print(f'{name} | {json.dumps(value)} | {threshold} | {verdict}', flush=True)
    def info(self, name, value): self.record(name, value, 'informational', 'INFO')
    def most(self, name, value, good, edge): self.record(name, round(value, 3), f'provisional: PASS <= {good}, BORDERLINE <= {edge}', LEVELS[0 if value <= good else 1 if value <= edge else 2])
    def least(self, name, value, good, edge): self.record(name, round(value, 3), f'provisional: PASS >= {good}, BORDERLINE >= {edge}', LEVELS[0 if value >= good else 1 if value >= edge else 2])
    def exact(self, name, value, want): self.record(name, value, f'must equal {want}', LEVELS[0 if value == want else 2])

def gate(real_id, ours, per=6, overrides=None, out=None):
    R = compare_npc.compare(real_id, ours, per, overrides, out); c = Checks()
    # --- the definition is part of the creature: lighting values and scale are given on the sheet
    c.exact('definition scale, ambient, contrast', R['definition']['ours'], R['definition']['real'])
    # --- size: worst face of the bounding box in body space, against the creature's largest dimension
    big = max(R['size_real'])
    c.most('bounding box, worst face gap in units', R['box_gap'], max(2, round(.015 * big)), max(4, round(.03 * big)))
    r, u = R['real']['faces'], R['ours']['faces']
    c.most(f'face count error % (ours {u}, real {r})', 100 * abs(u - r) / r, 10, 15)
    r, u = R['real']['colours'], R['ours']['colours']; cap = max(r + 1, -(-r * 3 // 2))
    c.record('distinct colours', u, f'provisional: PASS <= {cap} (real + 1 or half as many again), BORDERLINE <= {2 * r}', LEVELS[0 if u <= cap else 1 if u <= 2 * r else 2])
    c.most('colour distance', R['colour_distance'], 2.5, 3.5)
    for view, v in R['outline'].items(): c.least(f'rest outline, {view}', v, .93, .90)
    # --- built as the game builds: welded, no flat-shaded body unless the real one has it
    c.most('vertices on a shared position', R['ours']['split'], max(R['real']['split'], 0), max(R['real']['split'], 0) + 4)
    rf, uf = R['real']['flat'] / max(R['real']['faces'], 1), R['ours']['flat'] / max(R['ours']['faces'], 1)
    c.most('flat-shaded face share above the real one', max(0.0, uf - rf), .03, .08)
    # --- the skeleton's contract
    c.record('labels the real creature does not use', R['labels_not_in_real'], 'none (PASS); any is BORDERLINE: it may still be a label of the skeleton', LEVELS[0 if not R['labels_not_in_real'] else 1])
    c.info('real labels ours lacks', R['labels_missing'])
    if R['pivot_gap_median'] is not None:
        c.most('label centroid gap, median (units)', R['pivot_gap_median'], 3, 6)
        c.info('label centroid gap, 90th centile and worst', [round(R['pivot_gap_p90'], 1), list(R['pivot_gap_worst'])])
    # --- motion: the same sequences, front and rear quarter views
    bind = R['motion']['bind']['overlap_mean']
    for name, a in R['motion'].items():
        if name == 'bind':
            c.least('standing picture: outline overlap', a['overlap_mean'], .90, .85); c.most('standing picture: tone difference (grey levels of 255)', a['tone_mean'], 8, 12); continue
        c.least(f'{name}: outline overlap, mean of {a["frames"]} cells', a['overlap_mean'], .88, .82)
        c.least(f'{name}: outline overlap, worst cell ({a["worst_cell"]})', a['overlap_worst'], .78, .68)
        c.most(f'{name}: overlap lost against standing', max(0.0, bind - a['overlap_mean']), .05, .10)
        c.most(f'{name}: tone difference, mean', a['tone_mean'], 8, 12)
        c.info(f'{name}: share of pixels differing', round(a['differ_mean'], 3))
    missing = [k for k in ('stand', 'walk', 'attack', 'death') if k not in R['motion']]
    c.record('sequences tested', sorted(R['motion']), 'stand, walk, attack and death all present', LEVELS[0 if not missing else 1])
    for n in R.get('harness_notes', []): c.record('harness note', n, 'no sequence may fail to apply', 'FAIL' if 'NOT APPLIED' in n or 'NO `' in n else 'INFO')
    for k, (a, b) in R['style'].items(): c.info(f'style, real | ours: {k}', [round(a, 3), round(b, 3)])
    code = max(LEVELS.index(r['verdict']) for r in c.rows if r['verdict'] != 'INFO')
    print('VERDICT', LEVELS[code]); return code, c.rows, R

if __name__ == '__main__':
    a = sys.argv
    if len(a) < 4 or a[1] != 'rebuild-npc': raise SystemExit(__doc__)
    def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
    ov = {k: int(opt('--' + k)) for k in ('attack', 'death', 'stand', 'walk') if opt('--' + k)}
    try: code, rows, R = gate(a[2], a[3], int(opt('--per', 6)), ov, opt('--out'))
    except (Exception, SystemExit) as e:
        print(f'measurement error | "{type(e).__name__}: {e}" | all measurements complete | FAIL'); print('VERDICT FAIL'); code, rows = 2, [dict(name='measurement error', value=str(e), verdict='FAIL')]
    if opt('--json'): pathlib.Path(opt('--json')).write_text(json.dumps(dict(checks=rows, verdict=LEVELS[code]), indent=1, default=str))
    sys.exit(code)
