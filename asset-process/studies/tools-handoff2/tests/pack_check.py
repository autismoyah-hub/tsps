"""Check the delivered snapshot against its sources, including the picture bytes."""
import re
import sys
from pathlib import Path
from urllib.parse import unquote
sys.dont_write_bytecode = True
base = Path(__file__).resolve().parents[1]
sys.path.insert(0,str(base))
import review_pack as pack
out = base/'review-pack-test'
index = pack.read(out/'INDEX.md')
targets = re.findall(r'\]\(([^)]+)\)', index)
for target in targets:
    assert (out/unquote(target)).is_file(), target
pieces = list((out/'pieces').glob('*.md'))
photos = 0
for page in pieces:
    src = pack.read(page).split('## Picture source\n\n',1)[1].partition('\n')[0]
    if src != 'Missing':
        assert Path(src).read_bytes() == (out/'pictures'/(page.stem+'.png')).read_bytes()
        photos += 1
for p in (out/'owner').iterdir():
    assert p.read_bytes() == (pack.VERDICTS / p.name).read_bytes()
merged = pack.read(out/'PROPOSALS-ALL.md')
for path in (out/'producers').glob('*/*/PROPOSALS.md'):
    for raw in pack.read(path).splitlines():
        s = re.sub(r'^\d+\. |^- ','',raw.strip()).strip('| ').strip()
        if s and not raw.startswith(('#','| ---','| Source')): assert s in merged,s
assert 'VERDICT BORDERLINE' in pack.read(out/'pieces/trial-astra-table.md')
assert 'parked/trial-astra-table' in index and 'sitting-2026-11-26-g.json' in index
print(f'Verified {len(targets)} index links, {len(pieces)} piece records, {photos} byte-identical pictures, owner file bytes and all copied proposal entries.')
