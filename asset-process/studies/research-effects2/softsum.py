"""softsum.py: the softness study (data/soft.json, made by soft.py measure) reduced to rule numbers -> data/soft-summary.txt"""
import sys, json, pathlib, statistics as st, collections
HERE = pathlib.Path(__file__).resolve().parent
R = json.loads((HERE / 'data' / 'soft.json').read_text())
out = []


def med(v):
    v = [x for x in v if x is not None]
    return round(st.median(v), 2) if v else None


def rng(v):
    v = sorted(x for x in v if x is not None)
    return f'{v[0]} / {med(v)} / {v[-1]} (n={len(v)})' if v else '-'


def say(s): out.append(s); print(s)


F = [r['files'] for r in R]
say(f'{len(R)} real effects and animated objects of the reference fights; by kind {dict(collections.Counter(r["kind"] for r in R))}')
tr = [r for r in R if r['files']['alpha_share'] >= 0.3]; op = [r for r in R if r['files']['alpha_share'] < 0.05]
say(f'translucent over 30% of surface: {len(tr)}; wholly opaque at rest (under 5%): {len(op)}; of those opaque ones, alpha ANIMATED in the sequence: {sum(1 for r in op if r["files"].get("motion") and r["files"]["motion"]["alpha_last"] > r["files"]["motion"]["alpha_min"] + 20)}')
say(f'smooth-shaded share of faces, min / median / max: {rng([f["smooth_share"] for f in F])}; effects that are 100% smooth: {sum(1 for f in F if f["smooth_share"] >= 0.999)} of {len(F)}')
say(f'separate pieces per effect: {rng([f["parts"] for f in F])}; faces per piece (median over effects): {rng([round(f["faces"] / f["parts"]) for f in F])}')
many = [f for f in F if f['parts'] >= 20]
say(f'effects made of 20 or more pieces (lump or particle clouds): {len(many)}; their faces per piece {rng([round(f["faces"] / f["parts"]) for f in many])}; share of pieces under a fifth of the whole {med([f["small_parts"] / f["parts"] for f in many])}')
say(f'closed pieces share (closed shells / pieces), median {med([f["closed_parts"] / f["parts"] for f in F])}; flat cards share {med([f["card_parts"] / f["parts"] for f in F])}')
say(f'nesting depth (shells inside shells): {dict(sorted(collections.Counter(f["nest_depth"] for f in F).items()))}')
nested = [f for f in F if f['nest_depth'] >= 1 and f['alpha_share'] >= 0.1]
say(f'nested AND translucent: {len(nested)}; alpha by level, innermost first:')
for f, r in zip(F, R):
    if f in nested: say(f'    {r["effect"]:10s} {r["name"][:36]:36s} ' + ' > '.join(f'alpha {l["alpha"]} light {l["light"]} sat {l["sat"]}' for l in f['levels']))
ins = [f['levels'][0]['alpha'] for f in nested]; outs = [f['levels'][-1]['alpha'] for f in nested]
say(f'    innermost level alpha {rng(ins)}; outermost {rng(outs)}; outermost is the more transparent in {sum(1 for a, b in zip(ins, outs) if b > a)} of {len(nested)}')
say(f'alpha values per effect (each over 2% of the surface, opaque counted): {rng([len(f["alpha_values"]) for f in F])}; translucent values in use: ' +
    str(sorted(collections.Counter(a for f in F for a, s in f['alpha_values'] if a).items(), key=lambda kv: -kv[1])[:14]))
allv = sorted(a for f in tr for a, s in [(r_['files'], 0) for r_ in []] ) if False else None
vals = [a for r in tr for a, s in r['files']['alpha_values'] if a]
say(f'translucent alpha values in the translucent effects, 10 / 50 / 90 centile: {sorted(vals)[len(vals) // 10]} / {med(vals)} / {sorted(vals)[9 * len(vals) // 10]}')
say(f'alpha against distance from the centre (rank correlation), translucent effects: {rng([r["files"]["alpha_vs_radius"] for r in tr])}; positive (thinner outside) in {sum(1 for r in tr if (r["files"]["alpha_vs_radius"] or 0) > 0.1)} of {len(tr)}, negative in {sum(1 for r in tr if (r["files"]["alpha_vs_radius"] or 0) < -0.1)}')
say(f'inner (r < 0.4) alpha {rng([r["files"]["inner_alpha"] for r in tr])}; outer (r > 0.7) alpha {rng([r["files"]["outer_alpha"] for r in tr])}')
say(f'lightness inner {rng([f["inner_light"] for f in F])}; outer {rng([f["outer_light"] for f in F])}; lighter inside in {sum(1 for f in F if f["inner_light"] is not None and f["outer_light"] is not None and f["inner_light"] > f["outer_light"] + 3)}, lighter outside in {sum(1 for f in F if f["inner_light"] is not None and f["outer_light"] is not None and f["outer_light"] > f["inner_light"] + 3)} of {len(F)}')
say(f'colours per effect {rng([f["colours"] for f in F])}; median face edge over largest dimension {rng([f["edge_over_size"] for f in F])}')
mo = [r for r in R if r['files'].get('motion')]
grow = [r for r in mo if r['files']['motion']['size_max'] >= 1.5 * max(1, r['files']['motion']['size_first'])]
say(f'with a classic sequence: {len(mo)}; grow 1.5x or more: {len(grow)}; of those, growth factor {rng([round(r["files"]["motion"]["size_max"] / max(1, r["files"]["motion"]["size_first"]), 1) for r in grow])}, size peaks at {rng([r["files"]["motion"]["peak_at"] for r in grow])} of the life')
say(f'    growers: mean alpha first / at the size peak / last: {med([r["files"]["motion"]["alpha_first"] for r in grow])} / {med([r["files"]["motion"]["alpha_at_peak"] for r in grow])} / {med([r["files"]["motion"]["alpha_last"] for r in grow])}; '
    f'size and alpha rise together (corr > 0.3) in {sum(1 for r in grow if (r["files"]["motion"]["size_alpha_corr"] or 0) > 0.3)}, opposed (< -0.3) in {sum(1 for r in grow if (r["files"]["motion"]["size_alpha_corr"] or 0) < -0.3)}')
say(f'    ends more transparent than it starts: {sum(1 for r in mo if r["files"]["motion"]["alpha_last"] > r["files"]["motion"]["alpha_first"] + 10)} of {len(mo)}; starts transparent and comes in: {sum(1 for r in mo if r["files"]["motion"]["alpha_first"] > r["files"]["motion"]["alpha_min"] + 30)}')
pj = [r for r in R if r['kind'] == 'projectile']
tl = [r for r in pj if r['files'].get('trail')]
say(f'projectiles {len(pj)}: length along flight {rng([r["files"]["z_back"] - r["files"]["z_front"] for r in pj])}; behind the origin {rng([r["files"]["z_back"] for r in pj])}, ahead {rng([-r["files"]["z_front"] for r in pj])}')
say(f'    with faces well behind the origin (a tail): {len(tl)}; tail faces {rng([r["files"]["trail"]["faces"] for r in tl])}; tail alpha near / far {med([r["files"]["trail"]["alpha_near"] for r in tl])} / {med([r["files"]["trail"]["alpha_far"] for r in tl])}; '
    f'thinner toward the end (width against z < -0.2) in {sum(1 for r in tl if (r["files"]["trail"]["width_vs_z"] or 0) < -0.2)}; fainter toward the end in {sum(1 for r in tl if (r["files"]["trail"]["alpha_vs_z"] or 0) > 0.2)}')
sc = [r for r in R if r.get('screen', {}).get('edge_ratio') is not None]
say(f'ON SCREEN ({len(sc)} with 80 or more pixels): edge ratio (change 1 px inside the outline / change in the core) {rng([r["screen"]["edge_ratio"] for r in sc])}')
say(f'    translucent effects: {rng([r["screen"]["edge_ratio"] for r in sc if r["files"]["alpha_share"] >= 0.3])}; opaque: {rng([r["screen"]["edge_ratio"] for r in sc if r["files"]["alpha_share"] < 0.05])}')
say(f'    change levels across the mass {rng([r["screen"]["levels"] for r in sc])}; core change {rng([r["screen"]["core"] for r in sc])}')
for k in ('projectile', 'impact', 'aura', 'hazard', 'telegraph', 'hit', 'object'):
    rs = [r for r in sc if r['kind'] == k]
    say(f'    {k:10s} n {len(rs):2d}: edge ratio {med([r["screen"]["edge_ratio"] for r in rs])}, core change {med([r["screen"]["core"] for r in rs])}, edge change {med([r["screen"]["edge"] for r in rs])}, levels {med([r["screen"]["levels"] for r in rs])}, '
        f'smooth {med([r["files"]["smooth_share"] for r in rs])}, translucent share {med([r["files"]["alpha_share"] for r in rs])}, pieces {med([r["files"]["parts"] for r in rs])}, nest {med([r["files"]["nest_depth"] for r in rs])}')
(HERE / 'data' / 'soft-summary.txt').write_text('\n'.join(out) + '\n', encoding='utf-8')
