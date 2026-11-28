from pathlib import Path
import json
here=Path(__file__).resolve().parent
bands=json.loads((here/'bands.json').read_bytes())
slots=[r for k,r in bands.items() if k.startswith('slot-')]
for r in bands.values():
    for k in ('total','textured','alpha','both'):r.setdefault(k,0)
bands['worn']={key:sum(r[key] for r in slots) for key in ('total','textured','alpha','both')}
for key in ('ids','values'):
    bands['worn'][key]=sorted(set().union(*(r[key] for r in slots)))
for r in bands.values():
    for key in ('textured','alpha','both'):
        r[key+'_percent']=round(r[key]*100/r['total'],2) if r['total'] else None
(here.parents[1]/'tools/material_bands.json').write_text(json.dumps(bands,indent=1))
