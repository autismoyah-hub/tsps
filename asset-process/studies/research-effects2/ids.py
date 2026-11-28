"""ids.py ID [ID..]: census line (or 'NOT animated' and the plain definition) for object ids."""
import sys, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import objcensus as oc, objlib
from fxlib import names
rows = {r['id']: r for r in oc.load()}
for a in sys.argv[1:]:
    i = int(a)
    if i in rows: print(oc.line(rows[i])[:330])
    else:
        d = objlib.obj(i); print(i, names(6).get(i, ''), '| NOT animated; models', d['models'], 'shapes', d['shapes'], 'size', d['size'], 'transforms', bool(d['transforms']))
