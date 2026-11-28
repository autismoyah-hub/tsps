"""Blender worker. Embed the exact cache payload alongside editable material and UV attributes."""
import bpy,sys,json,colorsys,hashlib,base64
from pathlib import Path
p,out=map(Path,sys.argv[sys.argv.index('--')+1:])
m=json.loads((p/'model.json').read_text())
bpy.ops.wm.read_factory_settings(use_empty=True)
name=f"r{m['revision']} model {m['id']}"
me=bpy.data.meshes.new(name);ob=bpy.data.objects.new(name,me);bpy.context.collection.objects.link(ob)
unit=512 if m['hd'].get('version') in (14,15) else 128
me.from_pydata([(x/unit,z/unit,-y/unit) for x,y,z in m['vertices']],[],[tuple(reversed(f)) for f in m['faces']]);me.update()
def attr(key,domain,values):
 if values is not None:me.attributes.new('cache_'+key,'INT',domain).data.foreach_set('value',values)
for key in ('colors','priorities','alpha','face_textures','texture_coords','render_types','face_skins'):attr(key,'FACE',m[key])
attr('vertex_skins','POINT',m['vertex_skins'])
uv=me.uv_layers.new(name='HD mapping')
materials={};imgs={}
for i,f in enumerate(me.polygons):
 h,t,a,r=[m[k][i] for k in ['colors','face_textures','alpha','render_types']]
 key=(h,t,a,r)
 if key not in materials:
    mat=bpy.data.materials.new(f'HSL {h} | texture {t} | alpha {a}');mat.use_nodes=True
    rgb=colorsys.hls_to_rgb(((h>>10)+.5)/64,(h&127)/128,(((h>>7)&7)+.5)/8)
    opacity=(255-a)/255
    mat.diffuse_color=(*rgb,opacity);bs=mat.node_tree.nodes.get('Principled BSDF')
    bs.inputs['Base Color'].default_value=(*rgb,1);bs.inputs['Alpha'].default_value=opacity
    bs.inputs['Roughness'].default_value=1
    if t>=0:
        if t not in imgs:
            img=bpy.data.images.load(str(p/f'{t}.png'),check_existing=True);img.name=f'HD texture {t}'
            img.pack();imgs[t]=img
        node=mat.node_tree.nodes.new('ShaderNodeTexImage');node.image=imgs[t];node.interpolation='Closest'
        mat.node_tree.links.new(node.outputs['Color'],bs.inputs['Base Color'])
        mix=mat.node_tree.nodes.new('ShaderNodeMath');mix.operation='MULTIPLY'
        mix.inputs[1].default_value=opacity
        mat.node_tree.links.new(node.outputs['Alpha'],mix.inputs[0]);mat.node_tree.links.new(mix.outputs[0],bs.inputs['Alpha'])
    materials[key]=len(me.materials);me.materials.append(mat)
 f.material_index=materials[key];f.use_smooth=r!=1
 for loop,(u,v) in zip(f.loop_indices,reversed(m['uv'][i])):uv.data[loop].uv=(u,1-v)
doc=bpy.data.texts.new('HD source and fields.json');doc.write(json.dumps(m,separators=(',',':')))
doc.use_fake_user=True;me['source_sha256']=hashlib.sha256(base64.b64decode(m['source'])).hexdigest()
me['texture_preview_issues']=json.dumps(m['texture_issues'])
ob.select_set(True);bpy.context.view_layer.objects.active=ob
for area in bpy.context.screen.areas:
 if area.type=='VIEW_3D':
    area.spaces.active.region_3d.view_distance=max(2,max(ob.dimensions)*2)
    area.spaces.active.shading.color_type='TEXTURE'
bpy.context.preferences.filepaths.save_version=0
bpy.ops.wm.save_as_mainfile(filepath=str(out))
bpy.ops.wm.open_mainfile(filepath=str(out))
mesh=bpy.data.meshes[name]
assert len(mesh.polygons)==len(m['faces']) and len(mesh.vertices)==len(m['vertices'])
assert all(img.packed_file for img in bpy.data.images if img.name.startswith('HD texture'))
assert list(x.value for x in mesh.attributes['cache_priorities'].data)==m['priorities']
assert list(x.value for x in mesh.attributes['cache_alpha'].data)==m['alpha']
print('Verified reopened mesh',name,len(mesh.polygons),'faces',len(imgs),'packed textures')

