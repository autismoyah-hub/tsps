"""Exercise object dump and editable material slots. Render three intentional variants."""
import json,sys,subprocess,tempfile,runpy
from pathlib import Path
import bpy
here=Path(__file__).resolve().parent
tools=here.parents[1]/'tools'
sys.path.insert(0,str(tools))
import blender_kit
model=next(ob for ob in bpy.context.scene.objects if ob.type=='MESH')
for slot in model.material_slots:
    name=slot.material.name
    import re
    if sys.argv[-1]=='alpha':
        name=re.sub(r'_a\d+','',name)
    else:
        name=re.sub(r'_t\d+','',name)
    slot.material=bpy.data.materials.get(name) or bpy.data.materials.new(name)
sys.argv=sys.argv[:-1]
runpy.run_path(str(tools/'blender_export.py'),run_name='__main__')
