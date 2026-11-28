"""Classify build binaries by package registry and validated model layout."""
import collections,sys,json,hashlib
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
sys.path.insert(0,str(ROOT/'tools'))
import handoff
def inventory():
    paths=[]; skipped=collections.Counter(); groups=collections.defaultdict(list); rejected=[]
    registry={}
    for reg in ROOT.joinpath('builds').rglob('registry.txt'):
        for line in reg.read_text(errors='replace').splitlines()[1:]:
            row=line.split()
            if len(row)==4: registry[reg.parent/row[2]]=row[0]
    for path in ROOT.joinpath('builds').rglob('*.bin'):
        if HERE in path.parents:continue
        if 'incremental' in path.parts:skipped['rust incremental']+=1;continue
        if registry.get(path,'model')!='model':skipped['registry non-model']+=1;continue
        try:
            raw=path.read_bytes()
            m=handoff.decode(path)
            if min(m['priorities'])<0 or max(m['priorities'])>11:raise ValueError()
        except (AssertionError,IndexError,ValueError,KeyError) as error:
            rejected.append([str(path),repr(error)])
            skipped['not decodable model']+=1;continue
        if any(m['alpha']) or any(t>=0 for t in m['face_textures']):
            skipped['material-bearing model']+=1;continue
        groups[hashlib.sha256(raw).hexdigest()].append(str(path))
        paths.append(m['format'])
    (HERE/'inventory.json').write_text(json.dumps(groups,indent=1))
    (HERE/'non-models.json').write_text(json.dumps(rejected,indent=1))
    print(collections.Counter(paths),len(groups),skipped)
if __name__=='__main__':inventory()
