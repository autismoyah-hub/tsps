"""Indexed-sprite decoding for the revision-239 controls."""
from hdcache import Cursor
import numpy as np
from PIL import Image
N=128
def sprite(data):
 n=int.from_bytes(data[-2:],'big')
 r=Cursor(data);r.pos=len(data)-7-n*8
 w,h,pal=r.u16(),r.u16(),r.u8()
 offsets=[[r.u16() for _ in range(n)] for _ in range(4)]
 x,y,sw,sh=[a[0] for a in offsets]
 if max(w,h,sw,sh)>4096:raise ValueError('sprite dimensions')
 pos=len(data)-7-n*8-pal*3
 palette=np.zeros((pal+1,3),dtype='uint8')
 palette[1:]=np.frombuffer(data[pos:pos+pal*3],np.uint8).reshape(-1,3)
 flag=data[0]
 def plane(start):
    a=np.frombuffer(data,np.uint8,sw*sh,start)
    return a.reshape(sw,sh).T if flag&1 else a.reshape(sh,sw)
 ix=plane(1);arr=np.zeros((h,w,4),np.uint8)
 arr[y:y+sh,x:x+sw,:3]=palette[ix]
 arr[y:y+sh,x:x+sw,3]=plane(1+sw*sh) if flag&2 else (ix!=0)*255
 return np.asarray(Image.fromarray(arr).resize((N,N),Image.Resampling.NEAREST))
