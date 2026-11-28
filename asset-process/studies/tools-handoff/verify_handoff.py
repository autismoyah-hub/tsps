"""Read back the requested label reproductions, item dump, picture and JSON verdicts."""
from pathlib import Path
import hashlib
import json
import sys

sys.dont_write_bytecode = True
folder = Path(__file__).resolve().parent
sys.path.insert(0, str(folder))
from handoff import decode, osrscache as o
import scene
from PIL import Image

for name in ('legs','cuirass'):
    old = (folder/f'{name}-reference.bin').read_bytes()
    new = (folder/f'{name}-reproduced.bin').read_bytes()
    assert old == new, f'{name} differs'
    print(f'{name}: {len(new)} bytes identical, SHA-256 {hashlib.sha256(new).hexdigest()}')

dump = json.loads((folder/'1127.json').read_text())
base = json.loads((folder/'1127-baseline.json').read_text())
assert all(dump[k] == value for k, value in base.items())
item = json.loads(next((scene.ARCH/'items').glob('*/1127.json')).read_text())
parts = [o.model(i) for i in item['wear_models'][0] if i not in (-1,None)]
for key, source in [('vertex_labels','vertex_skins'), ('face_labels','face_skins'), ('priorities','priorities')]:
    assert dump[key] == [v for part in parts for v in part[source]]
    print(f'item 1127: {len(dump[key])} {key} match decoded cache parts in order')
print('All original worn dump fields match.')
box = decode(folder/'box-index.bin')
assert box['vertex_skins'][0] == 44 and box['vertex_skins'][3] == 43
assert not box['face_labels_present']
print('Inclusive y-up box labelled vertex 3; index override won at vertex 0; face stream absent.')
with Image.open(folder/'warden.png') as picture:
    assert picture.size == (600,410)
    picture.verify()
print('warden.png: verified 600 x 410 PNG')

levels = ['PASS','BORDERLINE','FAIL']
for name in ['claude','astra','opus','desk','map','helm','helm-stone','legs','cuirass','chain',
             'chain-astra','fang','fang-astra','helm-alone']:
    result = json.loads((folder/f'{name}.json').read_text())
    scored = [c['verdict'] for c in result['checks'] if c['verdict'] != 'INFO']
    assert result['verdict'] == max(scored, key=levels.index)
    print(f'{name}.json: {len(result["checks"])} checks, worst = {result["verdict"]}')
