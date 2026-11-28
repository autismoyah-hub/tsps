import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from inventory import *
p=HERE
g=json.loads((p/'inventory.json').read_text())
print([v[0] for v in g.values() if Path(v[0]).read_bytes()[-2:]==bytes([255,254])][:10])
c=json.loads((p/'candidates.json').read_text())
for kind,tex,alpha in [('shape-0',1,1),('shape-0',1,0),('shape-0',0,1),('slot-1',1,1),('slot-1',0,1),('shape-0',0,0)]:
    print([r for r in c if r[0]==kind and bool(r[4])==tex and bool(r[5])==alpha][:15])
