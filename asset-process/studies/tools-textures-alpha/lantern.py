"""Crystal lantern, dimensions in game units. Run in Blender."""
import math, sys
from pathlib import Path
import bpy
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]/'tools'))
from blender_kit import reset, add_mesh, recalc_outward, save
reset()
verts=[];faces=[];mats=[]
# Raised octagonal foot, bronze lip, inset crystal chamber and stepped cap.
profile=[(0,21),(4,25),(9,25),(14,17),(19,19),(52,19),(55,23),(60,23),(68,12),(72,6)]
for y,r in profile:
    for i in range(8):
        angle=(i+.5)*math.pi/4
        verts.append((round(r*math.cos(angle)),y,round(r*math.sin(angle))))
for level in range(len(profile)-1):
    for i in range(8):
        a=level*8+i;b=level*8+(i+1)%8
        faces.append((a,b,b+8,a+8))
        mats.append((29+i%3,5,66+5*(i%2),True,None,None,None,100) if level==4
                    else (6,4,34+6*(level%3),False,None,None,0,48 if level==3 else 0))
for top,base in [(False,0),(True,len(verts)-8)]:
    center=len(verts);verts.append((0,profile[-1 if top else 0][0],0))
    for i in range(8):
        faces.append((base+i,center,base+(i+1)%8));mats.append((6,4,44))
ob=add_mesh('Prismatic lantern',verts,faces,mats)
recalc_outward(ob)
# Four narrow uprights frame the panes. A faceted amber light hangs inside.
for k in range(0,8,2):
    angle=(k+.5)*math.pi/4
    x,z=20*math.cos(angle),20*math.sin(angle)
    post=[(round(x+u),y,round(z+v)) for y in (17,54)
          for u,v in ((-2,-2),(2,-2),(2,2),(-2,2))]
    quads=[(0,1,2,3),(4,5,6,7)]+[(i,(i+1)%4,(i+1)%4+4,i+4) for i in range(4)]
    recalc_outward(add_mesh(f'Upright {k}',post,quads,[(6,4,42,False,None,None,0,0)]*6))
core=[(0,22,0),(0,47,0),(7,32,0),(0,32,7),(-7,32,0),(0,32,-7)]
facets=[(tip,2+i,2+(i+1)%4) for tip in (0,1) for i in range(4)]
recalc_outward(add_mesh('Amber light',core,facets,[(8,7,88+i%4*5,True) for i in range(8)]))
save(str(HERE/'lantern.blend'))
