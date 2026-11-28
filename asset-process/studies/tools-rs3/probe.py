import pathlib,sys,re,collections
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import *
c=Cache()
for a in [16,18,19,21,26,47,52,53,54,55,56,57,58,59,60,61,62,65,66,67]:
 ix=c.index(a)
 print(a,len(ix),sum(map(len,ix.values())),min(ix),max(ix))
 if a in (26,):
    files=c.files(a,0)
    print(collections.Counter((d[:3].hex(),len(d)) for d in files.values()).most_common(25))
    for i,d in list(files.items())[:25]:print(i,d.hex())
 if a in (16,18,19,21):
    for f,d in list(c.files(a,0).items())[:5]: print(f,d[:128].hex())
    if a in (19,18):
        rx=re.compile(b'\x02(?:Raksha|Arch-Glacor|Kerapac|Croesus|Rasial|Solak|Seiryu|Ambassador|Eldritch crossbow|Fractured Staff of Armadyl|Masterwork platebody)\x00',re.I)
        for g in ix:
            for f,d in c.files(a,g).items():
                if rx.search(d):print((g<<(7 if a==18 else 8))+f,rx.search(d)[0],d[:88].hex())

