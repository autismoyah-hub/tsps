"""Write a classic model and describe transfer constraints for that particular model."""
import sys,pathlib
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from hdview import *
def convert(rev,i):
 c=Cache(rev);src=c.group(7,i);raw,hd=translate(src);m=decode(src);loss=[]
 if any(m['texture_types']):loss.append('Projected texture bases require baked UVs or a client projection path.')
 if any(hd.get('attachments',{}).values()):loss.append('Particle and billboard attachments omitted.')
 if hd['version'] in (14,15):
    if any(v%4 for p in m['vertices'] for v in p):loss.append('Subunit vertex coordinates rounded down to whole game units.')
    _,parts=asset(c,'model',i);raw=packed(parts)
 tex=sorted(set(m['face_textures'])-{-1})
 return raw,dict(revision=rev,id=i,format=hd['version'],losses=loss,texture_ids_requiring_target_materials=tex)
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('id',type=int);p.add_argument('out',type=Path)
 p.add_argument('--cache',type=int,choices=[561,601,614],default=614);p.add_argument('--allow-loss',action='store_true');a=p.parse_args()
 raw,details=convert(a.cache,a.id);print(json.dumps(details,indent=2))
 if details['losses'] and not a.allow_loss:raise SystemExit('Conversion needs --allow-loss for the listed fields.')
 a.out.write_bytes(raw)

