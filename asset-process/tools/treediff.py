"""treediff.py A B: files under A that are missing from B or differ in size (relative paths). Read only."""
import os
import sys

a, b = sys.argv[1], sys.argv[2]
missing, differ, total = [], [], 0
for dp, dn, fn in os.walk(a):
    for f in fn:
        pa = os.path.join(dp, f)
        rel = os.path.relpath(pa, a)
        pb = os.path.join(b, rel)
        total += 1
        try:
            sa = os.stat(pa).st_size
        except OSError:
            continue
        try:
            sb = os.stat(pb).st_size
        except OSError:
            missing.append((sa, rel)); continue
        if sa != sb:
            differ.append((sa, sb, rel))
print(f'{total} files under A; missing in B: {len(missing)} ({sum(s for s, _ in missing) / 2**20:.1f} MB); size differs: {len(differ)}')
tops = {}
for s, rel in missing:
    k = rel.split(os.sep)[0] + os.sep + (rel.split(os.sep)[1] if os.sep in rel else '')
    tops[k] = tops.get(k, 0) + s
for k, s in sorted(tops.items(), key=lambda x: -x[1])[:20]:
    print(f'  missing {s / 2**20:9.1f} MB  {k}')
for sa, sb, rel in differ[:10]:
    print(f'  differs {sa} vs {sb}  {rel}')
