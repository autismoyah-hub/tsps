import sys, pathlib, collections, hashlib, json, struct
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from hdcache import *
for rev in [561,601,614]:
    c=Cache(rev)
    for a in [9,26,18,19,20,21,2]:
        ids=c.index(a)
        for g in list(ids)[:1]:
            files=c.files(a,g)
            print(rev,a,g,[(i,d[:85].hex()) for i,d in list(files.items())[:2]])
    for i in json.loads((ROOT/'inventory.json').read_text())[str(rev)]['examples'].values():
        d=c.group(7,i)
        if not d.endswith(b'\xff\xff'):continue
        nv,nf,nt,flags,p,a,fs,ft,vs,x,y,z,idx,coord=struct.unpack('>HH7B5H',d[-23:-2])
        ts=d[:nt]
        end=nt+nv+nf*((flags&1)+1+(p==255)+fs+a+2*ft+2)+nv*vs+x+y+z+idx+coord+6*ts.count(0)
        comp=nt-ts.count(0)
        ver=d[-24] if flags&8 else 12
        end+=comp*(6+{12:6,14:7,15:9}[ver]+3)+ts.count(2)*2
        print(i,flags,ver,sorted(set(ts)),comp,len(d)-23-end,d[end:-23][:64].hex())
c=Cache(239)
def sig(d):return hashlib.blake2s(d,digest_size=16).digest()
by={sig(c.group(7,i)):i for i in c.index(7)}
h=Cache(561)
matches=[]
for i in h.index(7):
    d=h.group(7,i)
    if sig(d) in by:matches.append((i,by[sig(d)]))
(ROOT/'byte-matches.json').write_text(json.dumps(matches))
print('matches',len(matches),matches[:60])

