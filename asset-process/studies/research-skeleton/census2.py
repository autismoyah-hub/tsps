"""census2.py: conventions over every frame base that serves an NPC: written order (child-first / parent-first), the
order check of skelc.py, pivot offsets, group counts. Prints the numbers quoted in SKELETONS.md."""
import sys, json, pathlib, collections
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent / 'research-animation'))
import animlib as A, skelread, skelc
C = HERE.parent / 'research-npc' / 'census'
seqs = {}
for l in (C / 'sequences.jsonl').read_text().splitlines():
    if l.strip():
        r = json.loads(l); seqs[r['id']] = r
bases = sorted({seqs[n['anims'][0]]['skeleton'] for n in (json.loads(l) for l in (C / 'npcs.jsonl').read_text().splitlines() if l.strip())
                if n['anims'][0] in seqs and seqs[n['anims'][0]]['kind'] == 'classic' and seqs[n['anims'][0]]['skeleton'] >= 0})
style = collections.Counter(); clean = 0; kinds = collections.Counter(); root_last = root_first = 0
groups = []; maxlab = []
for b in bases:
    base = A.frame_base(b); bl = skelread.blocks_of(base); par = skelread.tree_of(bl)
    pairs = [(i, p) for i, p in enumerate(par) if p is not None]
    cf = sum(1 for i, p in pairs if i < p); pf = len(pairs) - cf
    if pairs: style['child-first (every joint before its parent)' if pf == 0 else 'parent-first (every joint after its parent)' if cf == 0 else
                    'mostly child-first' if cf >= 3 * pf else 'mostly parent-first' if pf >= 3 * cf else 'mixed'] += 1
    else: style['no chains'] += 1
    issues = skelc.order_check(base); clean += not issues
    for _, k, _ in issues: kinds[k] += 1
    alll = {l for ls, t in zip(base['labels'], base['types']) if t != 5 for l in ls}
    whole = [i for i, B in enumerate(bl) if B['moved'] and len(B['moved']) >= 0.95 * len(alll)]
    if whole:
        root_last += whole[-1] >= len(bl) - 2; root_first += whole[0] <= 1
    groups.append(len(base['types'])); maxlab.append(max(alll) if alll else 0)
print(f'{len(bases)} frame bases serve NPCs with a classic stand sequence')
print('written order of the joint blocks:', dict(style))
print(f'a whole-body block (moves 95% or more of the labels) is among the last two blocks in {root_last} bases, among the first two in {root_first}')
print(f'order check (skelc.order_check) finds nothing in {clean} of {len(bases)} bases; findings by kind {dict(kinds)}')
q = lambda xs, p: sorted(xs)[min(len(xs) - 1, int(p * len(xs)))]
print(f'groups per base 10/50/90/max: {q(groups, .1)}/{q(groups, .5)}/{q(groups, .9)}/{max(groups)}; bases with exactly 255 groups: {groups.count(255)}; highest label number 50/90/max: {q(maxlab, .5)}/{q(maxlab, .9)}/{max(maxlab)}')
# constant pivot offsets and channels actually used, on the five study bases
for b in (326, 815, 1491, 1080, 1782, 0):
    base = A.frame_base(b); n, use, move, vals = skelread.usage(b, base)
    T = base['types']
    off = [g for g in range(len(T)) if T[g] == 0 and vals[g] and vals[g].most_common(1)[0][0] != (0, 0, 0)]
    never = [g for g in range(len(T)) if use[g] == 0]
    mv = collections.Counter(skelread.KIND.get(T[g], T[g]) for g in range(len(T)) if move[g])
    print(f'base {b}: {n} sequences; {len(T)} groups; origin groups that carry a non-zero offset in most frames: {len(off)}; groups no sequence mentions: {len(never)}; groups some sequence moves, by type: {dict(mv)}')
