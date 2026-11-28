import sys,pathlib,collections,hashlib,json
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from models import *
# Filter by colours and face count before performing exact decoded comparisons.
def signature(d):
    ext=d[-2:] in (b'\xff\xff',b'\xff\xfd')
    skin=d[-2:] in (b'\xff\xfe',b'\xff\xfd')
    f=d[-(26 if ext and skin else 23 if ext or skin else 18):]
    nv,nf,nt,flags,p,a,fs=struct.unpack('>HH5B',f[:9])
    if ext:
        ft,vs=f[9:11]
        lens=11+skin
    else:
        ft,vs=0,f[9]
        lens=10+skin
    idx=int.from_bytes(f[lens+6:lens+8],'big')
    coord=int.from_bytes(f[lens+8:lens+10],'big') if ext else 0
    vlen=int.from_bytes(f[lens+8+2*ext:lens+10+2*ext],'big') if skin else nv*vs
    start=nt*ext+nv+nf*((flags&1)+1+(p==255)+fs+a+2*ft)+vlen+idx+coord
    return nv,nf,hashlib.blake2s(d[start:start+2*nf],digest_size=12).digest()
def main():
    o=Cache(239); lookup=collections.defaultdict(list)
    for i in o.index(7):
        d=o.group(7,i)
        try:lookup[signature(d)].append(i)
        except (IndexError,struct.error):pass
    results=[]
    errors={}
    for rev in [561,601,614]:
        h=Cache(rev)
        unique=set()
        for i in h.index(7):
            d=h.group(7,i)
            try:
                m=decode(d)
                if len(unique)>=12 or len(m['faces'])<8 or any(t>=0 for t in m['face_textures']):continue
                sig=signature(d)
                if sig in unique:continue
                for j in lookup.get(sig,[]):
                    n=osrs.decode_model(o.group(7,j))
                    if all(m[k]==n[k] for k in ['vertices','faces','colors','render_types','priorities','alpha','face_textures','depth_bias']):
                        results.append([rev,i,j,len(m['vertices']),len(m['faces'])]);unique.add(sig)
                        (ROOT/'matches.json').write_text(json.dumps(results))
                        break
            except Exception as e:
                errors.setdefault(str(rev),{}).setdefault(str(e),[]).append(i)
        print(rev,len(unique),[(k,len(v)) for k,v in errors.get(str(rev),{}).items()],flush=True)
        (ROOT/'model-errors.json').write_text(json.dumps(errors))
if __name__=='__main__':main()

