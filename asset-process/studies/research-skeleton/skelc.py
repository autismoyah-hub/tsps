"""skelc.py: a skeleton (frame base) compiler. A small description of JOINTS AS A TREE becomes a valid frame base,
the label numbers for the mesh, and named channels for the animation tool.

usage: skelc.py compile DESC.json OUTDIR      writes OUTDIR/base_B.dat (archive 1 group B file 0), OUTDIR/rig.json
                                              (joint name -> group numbers, label name -> number) and prints the groups
       skelc.py describe BASE OUT.json [--parent-first] [--npc ID]
                                              a REAL base of the cache written in this format (joints named jN after
                                              their origin group). Default: same group order as the real base.
                                              --parent-first: joints re-ordered so every parent precedes its children.
       skelc.py check DESC.json               only the checks

DESC.json
{ "name": "armillary", "base_id": 2700,
  "order": "child-first",                "child-first" | "parent-first" | "as-written" (default): how the joints are
                                         written out. child-first = every joint after its descendants: its angles are
                                         then LOCAL to its parent (Vorkath, the wolf). parent-first: angles are in
                                         MODEL axes and each pivot must ride on its parents (the player).
  "labels": {"core": 1},                 optional: pin a label name to a number; other names get free numbers 1.. in
                                         order of first appearance (0 is left free: unlabelled vertices are label 0)
  "joints": [                            IN THE ORDER THE GROUPS ARE WRITTEN (see rules below)
    {"name": "root", "parent": null,
     "pivot": ["floor"],                 labels whose centroid (on the current, already deformed mesh) is the pivot.
                                         A pivot label is NOT moved by this joint unless it is also in "own".
     "own": ["skirt"],                   labels this joint is the last to move (vertex labels)
     "channels": ["move", "turn", "scale"]     one group each, written after the joint's origin group in this order.
                                         move = translate (type 1), turn = rotate (type 2), scale (type 3).
                                         {"kind": "alpha", "faces": ["glow"]} = type 5 on FACE labels.
                                         {"kind": "turn", "name": "turn2", "add": [..], "drop": [..]}: a channel whose
                                         label list differs from the joint's cumulative list (real bases do this).
    }, ... ] }
What is compiled: for joint J, origin group = J.pivot; each move / turn / scale group lists J.own plus everything every
descendant of J owns (the CUMULATIVE list: that is how a chain is expressed without parent pointers).

Rules checked (errors stop, warnings print):
  E  more than 255 groups (the group count of a base and of a frame is one byte), a label above 255
  E  a label owned by two joints; a parent that does not exist
  W  ORDER. Parent-first: a joint's pivot labels may be moved only by itself or by joints written BEFORE it that are
     its ancestors (the pivot then travels with the parent: forward kinematics on current coordinates).
     Child-first: only by ancestors written AFTER it (the child turns about its rest pivot, then the parent carries
     both). Both are used by real bases; within one chain they must not be mixed, and a pivot label moved by a joint
     that is not an ancestor (or by a descendant) is flagged: the pivot would wander.
  W  a pivot with no labels (the pivot is then only the frame's offset, in model space: fine for a whole-body
     group, wrong for a limb); an own label that is also a pivot of an unrelated joint
"""
import sys, json, pathlib, collections
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'research-animation'))
import animlib as A

TYPE = {'move': 1, 'turn': 2, 'scale': 3, 'alpha': 5}
KIND = {0: 'origin', 1: 'move', 2: 'turn', 3: 'scale', 5: 'alpha'}


class Problem(Exception):
    pass


def order_check(base):
    """The one rule that makes a joint's pivot right, whatever the order of the groups: when a joint's origin group
    is reached, each of its pivot labels must have been carried by exactly the groups, written earlier, that also
    carry the joint's own vertices. A rotate or scale group written earlier that moves the pivot label but not the
    joint (the pivot WANDERS), or moves the joint but not the pivot label (the pivot STAYS BEHIND), is reported;
    translate groups are reported only when they split the two.
    -> [(origin group, 'wanders' | 'stays', text)]"""
    T, Ls = base['types'], [set(l) for l in base['labels']]
    out = []
    i = 0
    n = len(T)
    while i < n:
        if T[i] != 0: i += 1; continue
        j = i + 1
        moved = set()
        while j < n and T[j] != 0:
            if T[j] in (1, 2, 3): moved |= Ls[j]
            j += 1
        piv = Ls[i]
        if piv and moved:
            body = moved - piv or moved
            for g in range(i):
                if T[g] not in (1, 2, 3): continue
                carries_joint = body <= Ls[g]; touches_joint = bool(body & Ls[g])
                carries_piv = piv <= Ls[g]; touches_piv = bool(piv & Ls[g])
                if touches_piv and not touches_joint:
                    out.append((i, 'wanders', f'pivot label(s) {sorted(piv & Ls[g])[:4]} are moved by the earlier group g{g} ({KIND[T[g]]}), which does not move this joint: the pivot wanders'))
                elif carries_joint and not carries_piv:
                    out.append((i, 'stays', f'the earlier group g{g} ({KIND[T[g]]}) carries this joint but not its pivot label(s) {sorted(piv - Ls[g])[:4]}: the pivot stays behind'))
                elif touches_piv and touches_joint and not (carries_joint and carries_piv):
                    out.append((i, 'partly', f'the earlier group g{g} ({KIND[T[g]]}) moves part of this joint or part of its pivot labels'))
        i = j
    return out


def compile_desc(desc, quiet=False):
    """-> (base dict for animlib.encode_base, rig dict, [warnings])"""
    joints = desc['joints']
    names = [j['name'] for j in joints]
    if len(set(names)) != len(names): raise Problem('two joints share a name')
    how = desc.get('order', 'as-written')
    if how != 'as-written':
        # "child-first": every joint after all its descendants (angles are then LOCAL to the parent, as on Vorkath and
        # the wolf); "parent-first": every joint before its descendants (angles are in MODEL axes, as on the player)
        ks = collections.defaultdict(list)
        for j in joints: ks[j.get('parent')].append(j)
        ordered = []
        def walk(j):
            if how == 'parent-first': ordered.append(j)
            for k in ks[j['name']]: walk(k)
            if how == 'child-first': ordered.append(j)
        for j in ks[None]: walk(j)
        if len(ordered) != len(joints): raise Problem('a joint is not reachable from a root (parent missing or a cycle)')
        joints = ordered
    byname = {j['name']: j for j in joints}
    pos = {j['name']: i for i, j in enumerate(joints)}
    for j in joints:
        if j.get('parent') is not None and j['parent'] not in byname: raise Problem(f'{j["name"]}: parent {j["parent"]} does not exist')
    # ---- label numbers
    pinned = dict(desc.get('labels', {}))
    used = {v for v in pinned.values()}
    def all_labels():
        for j in joints:
            for l in j.get('pivot', []): yield l, 'v'
            for l in j.get('own', []): yield l, 'v'
            for c in j.get('channels', []):
                if isinstance(c, dict):
                    for l in c.get('add', []) + c.get('drop', []): yield l, 'v'
                    for l in c.get('faces', []): yield l, 'f'
    for l, _ in all_labels():
        if isinstance(l, int): used.add(l)
    number = {}; face_number = {}
    nxt = [1, 1]
    def num(l, kind):
        if isinstance(l, int): return l
        table = number if kind == 'v' else face_number
        if l in pinned: table[l] = pinned[l]
        if l not in table:
            k = 0 if kind == 'v' else 1
            while nxt[k] in used and kind == 'v': nxt[k] += 1
            table[l] = nxt[k]; nxt[k] += 1
        return table[l]
    for l, kind in all_labels(): num(l, kind)
    if any(v > 255 or v < 0 for v in list(number.values()) + list(face_number.values()) + list(used)): raise Problem('a label number is outside 0..255')
    # ---- tree
    kids = collections.defaultdict(list)
    for j in joints: kids[j.get('parent')].append(j['name'])
    own = {j['name']: [num(l, 'v') for l in j.get('own', [])] for j in joints}
    owner = {}
    for n, ls in own.items():
        for l in ls:
            if l in owner: raise Problem(f'label {l} is owned by both {owner[l]} and {n}')
            owner[l] = n
    cum = {}
    def cumulative(n):
        if n not in cum:
            out = list(own[n])
            for k in kids[n]: out += cumulative(k)
            cum[n] = out
        return cum[n]
    def ancestors(n):
        out = []
        p = byname[n].get('parent')
        while p is not None:
            out.append(p); p = byname[p].get('parent')
        return out
    # ---- groups
    types, labels = [], []
    rig = {}
    warnings = []
    moved_by = collections.defaultdict(list)       # label -> [(joint, group index)] for move/turn/scale groups
    for j in joints:
        n = j['name']
        piv = [num(l, 'v') for l in j.get('pivot', [])]
        entry = dict(parent=j.get('parent'), origin=len(types), pivot_labels=piv, own_labels=own[n], channels={})
        types.append(0); labels.append(piv)
        for c in j.get('channels', ['turn']):
            spec = c if isinstance(c, dict) else {'kind': c}
            kind = spec['kind']
            if kind == 'alpha':
                ls = [num(l, 'f') for l in spec.get('faces', [])]
            else:
                ls = list(cumulative(n))
                ls = [l for l in ls if l not in {num(x, 'v') for x in spec.get('drop', [])}]
                ls += [num(x, 'v') for x in spec.get('add', []) if num(x, 'v') not in ls]
                for l in ls: moved_by[l].append((n, len(types)))
            cname = spec.get('name', kind)
            if cname in entry['channels']: raise Problem(f'{n}: two channels named {cname}; give one a "name"')
            entry['channels'][cname] = dict(group=len(types), kind=kind, labels=len(ls))
            types.append(TYPE[kind]); labels.append(ls)
        rig[n] = entry
    if len(types) > 255: raise Problem(f'{len(types)} groups: a frame base holds at most 255')
    # ---- order checks
    for j in joints:
        n = j['name']
        if not rig[n]['pivot_labels'] and byname[n].get('parent') is not None and any(c['kind'] != 'alpha' for c in rig[n]['channels'].values()):
            warnings.append(f'{n}: no pivot labels on a joint that is not the root: its pivot is the frame offset alone, in model space')
    gname = {}
    for n, e in rig.items():
        gname[e['origin']] = n
        for c in e['channels'].values(): gname[c['group']] = n
    for g, kind, what in order_check(dict(types=types, labels=labels)):
        warnings.append(f'{gname[g]}: {what}')
    order = collections.Counter('parent-first' if pos[j['parent']] < pos[j['name']] else 'child-first' for j in joints if j.get('parent') is not None)
    base = dict(types=types, labels=labels, tail=bytes(2))
    meta = dict(name=desc.get('name'), base_id=desc.get('base_id'), groups=len(types), joints=rig,
                labels={k: v for k, v in sorted(number.items(), key=lambda kv: kv[1])}, face_labels=face_number,
                order=dict(order), kinds=dict(collections.Counter(KIND[t] for t in types)))
    return base, meta, warnings


def write(desc, outdir):
    base, meta, warnings = compile_desc(desc)
    out = pathlib.Path(outdir); out.mkdir(parents=True, exist_ok=True)
    data = A.encode_base(base)
    back = A.decode_base(data)
    assert back['types'] == base['types'] and back['labels'] == base['labels'], 'base self-check'
    (out / f'base_{desc["base_id"]}.dat').write_bytes(data)
    (out / 'rig.json').write_text(json.dumps(meta, indent=1))
    print(f'{desc.get("name")}: base {desc["base_id"]}, {len(base["types"])} groups {meta["kinds"]}, {len(meta["joints"])} joints, '
          f'{len(meta["labels"])} named vertex labels, {len(meta["face_labels"])} face labels, {len(data)} bytes; joint order {meta["order"]}')
    for n, e in meta['joints'].items():
        ch = ', '.join(f'{c}=g{v["group"]}({v["labels"]})' for c, v in e['channels'].items())
        print(f'  {n:14s} parent {str(e["parent"]):12s} origin g{e["origin"]} pivot {e["pivot_labels"]}  {ch}')
    for w in warnings: print('WARNING', w)
    print(f'wrote {out / ("base_%d.dat" % desc["base_id"])} and rig.json')
    return base, meta, warnings


def describe(bid, parent_first=False, child_first=False):
    """A real base in the description format. Exact: compiling it gives the same groups (types and label sets) in the
    same order, unless parent_first, where the joint blocks are re-ordered parents before children."""
    import skelread
    base = A.frame_base(bid)
    blocks = skelread.blocks_of(base); parent = skelread.tree_of(blocks); own = skelread.owner_of(blocks)
    kids = collections.defaultdict(list)
    for b, p in enumerate(parent): kids[p].append(b)
    name = lambda b: f'j{blocks[b]["origin"]}' if blocks[b]['origin'] is not None else f'j_{b}'
    cum = {}
    def cumulative(b):
        if b not in cum:
            s = {l for l, o in own.items() if o == b}
            for k in kids[b]:
                if blocks[k]['moved']: s |= cumulative(k)
            cum[b] = s
        return cum[b]
    order = list(range(len(blocks)))
    if parent_first:
        order = []
        def walk(b):
            order.append(b)
            for k in kids[b]: walk(k)
        for b in kids[None]: walk(b)
    if child_first:
        order = []
        def walk2(b):
            for k in kids[b]: walk2(k)
            order.append(b)
        for b in kids[None]: walk2(b)
    joints = []; patched = 0; dup = 0
    for b in order:
        B = blocks[b]
        if B['origin'] is None: raise Problem('base starts with a non-origin group: not expressible')
        j = dict(name=name(b), parent=name(parent[b]) if parent[b] is not None else None, pivot=list(base['labels'][B['origin']]),
                 own=sorted(l for l, o in own.items() if o == b), channels=[])
        seen = collections.Counter()
        for g in B['groups']:
            t = base['types'][g]; ls = base['labels'][g]
            if len(set(ls)) != len(ls): dup += 1
            kind = KIND[t]
            seen[kind] += 1
            c = dict(kind=kind)
            if seen[kind] > 1: c['name'] = f'{kind}{seen[kind]}'
            if t == 5: c['faces'] = list(ls)
            else:
                add = sorted(set(ls) - cumulative(b)); drop = sorted(cumulative(b) - set(ls))
                if add: c['add'] = add
                if drop: c['drop'] = drop
                patched += bool(add or drop)
            j['channels'].append(c if len(c) > 1 else kind)
        joints.append(j)
    desc = dict(name=f'real base {bid}', base_id=bid, joints=joints)
    return desc, base, blocks, order, dict(channels_patched=patched, groups_with_duplicate_labels=dup)


if __name__ == '__main__':
    cmd = sys.argv[1]
    try:
        if cmd == 'compile':
            write(json.loads(pathlib.Path(sys.argv[2]).read_text()), sys.argv[3])
        elif cmd == 'check':
            base, meta, warnings = compile_desc(json.loads(pathlib.Path(sys.argv[2]).read_text()))
            print(f'{len(base["types"])} groups, {len(meta["joints"])} joints, order {meta["order"]}; {len(warnings)} warnings')
            for w in warnings: print('WARNING', w)
        elif cmd == 'describe':
            desc, base, blocks, order, info = describe(int(sys.argv[2]), '--parent-first' in sys.argv)
            pathlib.Path(sys.argv[3]).write_text(json.dumps(desc, indent=1))
            print(f'base {sys.argv[2]}: {len(desc["joints"])} joints written to {sys.argv[3]}; {info}')
    except Problem as e:
        raise SystemExit(f'ERROR {e}')
