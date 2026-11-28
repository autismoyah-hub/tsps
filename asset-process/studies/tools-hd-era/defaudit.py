import sys,pathlib,collections,json
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from definitions import *
for rev in (561,601,614):
 c=Cache(rev); errors={}
 for kind in BANK:
    hist=collections.Counter(); samples={}
    for i,data in records(c,kind):
        try:parse(kind,data,rev)
        except (ValueError,IndexError) as e:
            key=str(e).split(' at ')[0].split(' after ')[0]
            hist[key]+=1;samples.setdefault(key,[i,data.hex()[:220]])
    errors[kind]={k:[n,samples[k]] for k,n in hist.items()}
    print(rev,kind,hist.most_common(12))
 (ROOT/f'def-errors-{rev}.json').write_text(json.dumps(errors,indent=1))

