"""dsize.py ROOT [DEPTH] [OUT]: sizes of the folders under ROOT, down to DEPTH levels, largest first.
Uses os.scandir (on Windows the directory listing carries the file sizes, so this is fast). Read only."""
import os
import sys

root = sys.argv[1]
depth = int(sys.argv[2]) if len(sys.argv) > 2 else 1
out = sys.argv[3] if len(sys.argv) > 3 else None
rows = []


def walk(path, level):
    total, count = 0, 0
    try:
        with os.scandir(path) as it:
            for e in it:
                try:
                    if e.is_symlink():
                        continue
                    if e.is_dir(follow_symlinks=False):
                        t, c = walk(e.path, level + 1)
                        total += t; count += c
                    else:
                        total += e.stat(follow_symlinks=False).st_size; count += 1
                except OSError:
                    pass
    except OSError:
        pass
    if level <= depth:
        rows.append((total, count, path))
    return total, count


walk(root, 0)
rows.sort(reverse=True)
lines = [f'{t / 2**30:9.2f} GB {c:9d} files  {p}' for t, c, p in rows]
text = '\n'.join(lines)
if out:
    with open(out, 'w', encoding='utf-8') as f:
        f.write(text + '\n')
print(text)
