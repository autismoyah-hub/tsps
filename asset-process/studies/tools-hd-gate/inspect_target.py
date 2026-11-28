import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from hdasset import *
for kind,i in [('object',22772),('item-ground',7535),('item-worn',7535),('item-female',7535),('spot',2768)]:
    d,p=target(614,kind,i)
    m=combined(p)
    print(kind,d,len(m['vertices']),len(m['faces']),set(m['alpha']),set(m['face_textures']),flush=True)
    if kind=='spot':print(animation(614,d['sequence'],m)[0])

