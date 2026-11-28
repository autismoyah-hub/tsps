"""Compare both parsers and the independently archived sheets."""
import importlib.util
import sys
from pathlib import Path
sys.dont_write_bytecode = True
base = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(base))
import compare
spec = importlib.util.spec_from_file_location('previous', base.parents[1] / 'tools/compare.py')
old = importlib.util.module_from_spec(spec)
spec.loader.exec_module(old)
worked = 0
for oid in [39924, 25702, *sorted(compare.scene.config_files(6, 'objects'))]:
    raw = compare.scene.object_def(oid)
    before = old.recolours(raw)
    after = compare.recolours(raw)
    if oid not in (39924, 25702) and (not before or before != after or
            not (compare.scene.ARCH / 'objects/shape-10' / f'{oid}.json').exists()):
        continue
    sheet = compare.scene.object_json(oid)
    assert {k:v for k,v in after.items() if isinstance(k, int)} == dict(sheet['recolors'])
    assert [after.get(('scale', k), 128) for k in range(3)] == sheet['scale']
    if oid == 39924:
        assert not before and len(after) == 3
    print(oid, before, '->', after)
    worked += before == after
    if worked == 5: break
assert worked == 5
