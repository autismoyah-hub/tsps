"""Write our rooms' compose tables in the armoury's placements.csv form, by running their own compose code's room
builders (no picture is made; nothing in their folders is written).

usage: export_ours.py OUTDIR
  OUTDIR/scribes-p1.csv   scribes' hall, first-pass plan (the arrangement shown when it was graded)
  OUTDIR/scribes-p2.csv   scribes' hall, second-pass plan
  OUTDIR/observatory.csv  furnished observatory (compose.py chamber(''))
  OUTDIR/armoury-orig.csv the armoury as the orchestrator composed it (copied from night/armoury/boards/final-selected)
"""
import sys, csv, shutil, importlib.util, pathlib
HERE = pathlib.Path(__file__).resolve().parent
TF = HERE.parents[1]
sys.dont_write_bytecode = True


def rows_of(room):
    by_id = {mid: key[0] for key, (mid, *_) in room._models.items()}
    out = []
    for mid, _, _, x, z, rot, shape in room.extra:
        out.append([by_id[mid], x, z, rot, shape, 1, 1])
    for rid, x, z, rot in room.real:
        out.append([('figure:' if rid < 0 else 'real:') + str(abs(rid) if rid < 0 else rid), x, z, rot, 10, 1, 1])
    return out


def write(path, rows, sizes):
    with open(path, 'w', newline='') as f:
        w = csv.writer(f); w.writerow(['source', 'tile_x', 'tile_z', 'rotation', 'shape', 'width', 'depth'])
        for r in rows:
            for pat, (sx, sz) in sizes.items():
                if pat in r[0].replace('\\', '/'): r[5], r[6] = sx, sz
            w.writerow(r)


def load(name, path, argv=None):
    old = sys.argv
    if argv: sys.argv = argv
    sys.path.insert(0, str(path.parent))
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)
    sys.argv = old; sys.path.pop(0)
    for k in [k for k in sys.modules if k in ('roomlib', 'kitlib')]: del sys.modules[k]
    return mod


def main():
    out = pathlib.Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
    SIZES = {'desk.bin': (2, 1), 'scribe-bookcase': (2, 1), 'obs2-table-trestle': (2, 1)}
    for plan in ('p1', 'p2'):
        sc = load('scribes_' + plan, TF / 'builds/room-scribes-hall/compose.py', ['compose.py', 'export', 'plan=' + plan])
        from_room = sc.Room(ambient=40, contrast=25)
        sc.DECO.clear(); sc.floor(from_room); sc.shell(from_room, ''); sc.furniture(from_room)
        write(out / f'scribes-{plan}.csv', rows_of(from_room), SIZES)
    ob = load('obs_furnished', TF / 'builds/room-observatory-furnished/compose.py')
    room = ob.chamber('')
    write(out / 'observatory.csv', rows_of(room), SIZES)
    src = TF / 'night/armoury/boards/final-selected/placements.csv'
    rows = list(csv.reader(open(src, newline='')))
    with open(out / 'armoury-orig.csv', 'w', newline='') as f:
        w = csv.writer(f); w.writerow(rows[0][:7])
        for r in rows[1:]: w.writerow(r[:7])
    print('written', sorted(p.name for p in out.glob('*.csv')))


if __name__ == '__main__':
    main()
