"""Blender backend for a decoded JSON mesh. Use Blender -b --python-exit-code 1 -P this.py -- mesh.json out.blend [views-directory]."""
import sys,json,math
from pathlib import Path
import bpy
from mathutils import Vector

def build(path):
    m=json.loads(path.read_text(encoding='utf8'))
    bpy.ops.wm.read_factory_settings(use_empty=True)
    mesh=bpy.data.meshes.new(m['name'])
    obj=bpy.data.objects.new(m['name'],mesh)
    bpy.context.collection.objects.link(obj)
    # RS3 +Y up. Reflection to Blender (X,Z,Y) requires reversing winding.
    mesh.from_pydata([(x/4,z/4,y/4) for x,y,z in m['vertices']],[],[f[::-1] for f in m['faces']])
    mesh.update()
    color=mesh.color_attributes.new(name='Vertex RGBA',type='FLOAT_COLOR',domain='POINT')
    if len(m['rgba'])!=len(mesh.vertices):raise ValueError('vertex colours')
    color.data.foreach_set('color',[v for row in m['rgba'] for v in row])
    uv=mesh.uv_layers.new(name='Mesh UV')
    alpha=mesh.attributes.new('Transparency','INT','FACE')
    alpha.data.foreach_set('value',m['alpha'])
    face_material=mesh.attributes.new('Source material','INT','FACE')
    face_material.data.foreach_set('value',m['material'])
    slots={}
    for f in mesh.polygons:
        i=f.index
        mid=m['material'][i]
        key=(mid,m['alpha'][i])
        if key not in slots:
            mat=bpy.data.materials.new(f'Material {mid}, alpha {key[1]}')
            mat.use_nodes=True
            nodes=mat.node_tree.nodes
            link=mat.node_tree.links.new
            shader=nodes.get('Principled BSDF')
            colors=nodes.new('ShaderNodeVertexColor')
            colors.layer_name='Vertex RGBA'
            rgb=colors.outputs['Color']
            opacity=colors.outputs['Alpha']
            if mid>=0:
                # Only local PNGs beside the interchange file are accepted.
                file=(path.parent/f'{mid}.png').resolve()
                if file.parent!=path.parent or not file.read_bytes().startswith(bytes.fromhex('89504e470d0a1a0a')):raise ValueError('expected PNG')
                img=bpy.data.images.load(str(file),check_existing=True)
                img.pack()
                texture=nodes.new('ShaderNodeTexImage')
                texture.image=img
                mix=nodes.new('ShaderNodeMixRGB')
                mix.blend_type='MULTIPLY'
                mix.inputs[0].default_value=1
                link(rgb,mix.inputs[1])
                link(texture.outputs['Color'],mix.inputs[2])
                rgb=mix.outputs[0]
                mult=nodes.new('ShaderNodeMath')
                mult.operation='MULTIPLY'
                link(opacity,mult.inputs[0])
                link(texture.outputs['Alpha'],mult.inputs[1])
                opacity=mult.outputs[0]
            mult=nodes.new('ShaderNodeMath')
            mult.operation='MULTIPLY'
            mult.inputs[1].default_value=1-key[1]/255
            link(opacity,mult.inputs[0])
            link(mult.outputs[0],shader.inputs['Alpha'])
            link(rgb,shader.inputs['Base Color'])
            slots[key]=len(mesh.materials)
            mesh.materials.append(mat)
        f.material_index=slots[key]
        for loop,coord in zip(f.loop_indices,reversed(m['uv'][i]),strict=True):
            uv.data[loop].uv=coord
    for i,weights in enumerate(m['weights']):
        for bone,weight in weights:
            group=obj.vertex_groups.get(str(bone)) or obj.vertex_groups.new(name=str(bone))
            group.add([i],weight/255,'REPLACE')
    bpy.context.view_layer.objects.active=obj
    obj.select_set(True)
    return obj,m

def render(obj,folder):
    sc=bpy.context.scene
    sc.render.engine='CYCLES'
    sc.cycles.device='CPU'
    sc.cycles.samples=4
    sc.render.threads_mode='FIXED'
    sc.render.threads=2
    sc.render.resolution_x=sc.render.resolution_y=220
    sc.render.resolution_percentage=100
    sc.render.film_transparent=True
    sc.world=bpy.data.worlds.new('Studio')
    sc.world.use_nodes=True
    sc.world.node_tree.nodes['Background'].inputs[0].default_value=(.85,.85,.85,1)
    bounds=[Vector(v) for v in obj.bound_box]
    center=sum(bounds,Vector())/8
    radius=max(.01,max((p-center).length for p in bounds))
    bpy.ops.object.camera_add()
    cam=bpy.context.object
    sc.camera=cam
    cam.data.type='ORTHO'
    cam.data.ortho_scale=2.12*radius
    cam.data.clip_start=radius*.001
    cam.data.clip_end=radius*10
    for i,d in enumerate([(0,-1,0),(1,0,0),(0,1,0),(-1,0,0),(0,0,1),(1,-1,.7)]):
        cam.location=center+Vector(d).normalized()*radius*4
        cam.rotation_euler=(center-cam.location).to_track_quat('-Z','Y').to_euler()
        sc.render.filepath=str(folder/f'{i}.png')
        bpy.ops.render.render(write_still=True)
if __name__=='__main__':
    args=sys.argv[sys.argv.index('--')+1:]
    obj,source=build(Path(args[0]).resolve())
    name=obj.name
    bpy.context.preferences.filepaths.save_version=0
    bpy.ops.wm.save_as_mainfile(filepath=str(Path(args[1]).resolve()))
    bpy.ops.wm.open_mainfile(filepath=str(Path(args[1]).resolve()))
    obj=bpy.data.objects[name]
    assert [f.value for f in obj.data.attributes['Transparency'].data]==source['alpha']
    assert all(im.packed_file for im in bpy.data.images if im.source=='FILE' and im.filepath)
    for i,ws in enumerate(source['weights']):
        assert len(obj.data.vertices[i].groups)==len(ws)
        for bone,w in ws:assert abs(obj.vertex_groups[str(bone)].weight(i)-w/255)<1e-6
    if len(args)>2:render(obj,Path(args[2]))

