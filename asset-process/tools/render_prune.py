"""render_prune.py [--delete]: list (or delete) intermediate render folders under <ROOT>/render.
Keeps: names containing 'final'; fight films (fight-*, fx-*-fight*); research-arenas; boss-pitch-*; anything
modified in the last 6 hours (work in flight). Everything else is regenerable from the build scripts.
Without --delete it only writes the plan to render-prune-plan.txt beside this tool's parent folder."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import os
import shutil
import sys
import time

ROOT = _AP.fix('<ROOT>/render')
PLAN = _AP.fix('<ROOT>/render-prune-plan.txt')
now = time.time()


def keep(name, mtime):
    n = name.lower()
    return ('final' in n or n.startswith('fight-') or ('fight' in n and n.startswith('fx-'))
            or n.startswith('research-arenas') or n.startswith('boss-pitch-') or now - mtime < 6 * 3600)


rows = []
for e in os.scandir(ROOT):
    if not e.is_dir(follow_symlinks=False):
        continue
    size, m = 0, 0.0
    for dp, dn, fn in os.walk(e.path):
        for f in fn:
            try:
                st = os.stat(os.path.join(dp, f)); size += st.st_size; m = max(m, st.st_mtime)
            except OSError:
                pass
    rows.append((e.name, size, m, keep(e.name, m)))
kept = [r for r in rows if r[3]]; gone = [r for r in rows if not r[3]]
with open(PLAN, 'w', encoding='utf-8') as f:
    f.write(f'KEEP {len(kept)} folders, {sum(r[1] for r in kept) / 2**30:.2f} GB\n')
    for r in sorted(kept): f.write(f'  keep {r[1] / 2**20:9.1f} MB  {r[0]}\n')
    f.write(f'DELETE {len(gone)} folders, {sum(r[1] for r in gone) / 2**30:.2f} GB\n')
    for r in sorted(gone): f.write(f'  del  {r[1] / 2**20:9.1f} MB  {r[0]}\n')
print(f'keep {len(kept)} folders {sum(r[1] for r in kept) / 2**30:.2f} GB; delete {len(gone)} folders {sum(r[1] for r in gone) / 2**30:.2f} GB; plan in {PLAN}')
if '--delete' in sys.argv:
    for name, _, _, _ in gone:
        shutil.rmtree(os.path.join(ROOT, name), ignore_errors=True)
    left = [n for n, _, _, _ in gone if os.path.exists(os.path.join(ROOT, n))]
    print('deleted;', len(left), 'folders could not be fully removed:', left[:10])
