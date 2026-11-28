import sys, collections
sys.path.insert(0, str(__import__('pathlib').Path(__file__).parent))
import animlib as A
c = A.Cache()
seqs = c.files(2, 12)
for sid in [1771, 4847, 8025, 8026, 8525, 9300, 11553]:
    d = seqs[sid]; s = A.decode_sequence(d); e = A.encode_sequence(s)
    print(sid, len(d), len(e), s['ops'], d[:40].hex(), '|', e[:40].hex())
# keyframe sequences: what does opcode 1 hold
k = [s for s in map(A.decode_sequence, seqs.values()) if s['cached_animation'] is not None]
print('keyframe seqs with classic frames too:', sum(1 for s in k if s['frames']), 'of', len(k))
ids = sorted(sid for sid, d in seqs.items() if A.decode_sequence(d)['cached_animation'] is not None)
print('keyframe sequence ids: first', ids[:10], 'last', ids[-5:], 'below 10000:', sum(1 for i in ids if i < 10000))
# bases with bones
nb = 0; odd = collections.Counter()
for bid in c.reference(1)['groups']:
    b = A.decode_base(c.file(1, bid, 0))
    if int.from_bytes(b['tail'][:2], 'big') > 0: nb += 1
    for t in b['types']:
        if t not in (0, 1, 2, 3, 5): odd[(bid, t)] += 1
print('bases with bones > 0:', nb, 'odd types', dict(odd))
print('player base 0 tail', c.file(1, 0, 0)[-8:].hex(), 'bones', int.from_bytes(A.decode_base(c.file(1, 0, 0))['tail'][:2], 'big'))
# failing frames
bad = collections.Counter()
for gid in c.reference(0)['groups']:
    files = c.files(0, gid)
    first = next(iter(files.values())); bid = int.from_bytes(first[:2], 'big'); base = A.frame_base(bid, c)
    for fid, data in files.items():
        try: A.decode_frame(data, base)
        except Exception as e: bad[(gid, bid, repr(e))] += 1
print(dict(bad))
used = collections.Counter()
for sid, d in seqs.items():
    s = A.decode_sequence(d)
    for f in s['frames']:
        if any(f >> 16 == g for g, _, _ in bad): used[sid] += 1
print('sequences using failing groups:', dict(used))
print('archive 22 groups:', len(c.reference(22)['groups']), 'files', sum(len(g['files']) for g in c.reference(22)['groups'].values()))
