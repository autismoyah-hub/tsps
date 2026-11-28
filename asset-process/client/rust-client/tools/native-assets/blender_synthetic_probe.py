"""Original connected synthetic mesh and one classic ordered-property action.

Run only with an explicit Blender BUILD GO. Importing this module does not import
bpy. No cache/model/old action inputs, namespace assignment or native launch.
"""
import argparse
import json
import math
from pathlib import Path
import sys
sys.dont_write_bytecode = True  # Embedded Blender must not write helper pyc beside source.
sys.path.insert(0,str(Path(__file__).resolve().parent))
import synthetic_expectations as oracle


def authored_input():
    # Original authoring declarations, independent of the oracle's literal poses.
    model=dict(vertices=[[0,0,0],[128,0,0],[112,0,64],[0,0,64],
                         [0,-128,0],[96,-128,0],[80,-128,48],[0,-128,64]],
               faces=[[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],
                      [1,2,6],[1,6,5],[2,3,7],[2,7,6],[3,0,4],[3,4,7]],
               colors=[43051,43051,8084,8084,43051,8084,43051,8084,43051,8084,43051,8084],
               vertex_skins=[0,0,0,0,1,1,1,1],priority=0)
    rows=[]
    for length,angle,shift in [(50,0,[0,0,0]),(75,64,[0,0,0]),(100,64,[8,0,-16])]:
        rows.append(dict(length=length,types=[0,2,1],labels=[[],[1],[1]],
                         transforms=[[0,[16,-64,8]],[1,[0,0,angle]],[2,shift]]))
    return model,dict(frames=rows,policy=dict(loop_frames=-1,max_loops=1))

def native_preview(model, row):
    """Narrow diagnostic preview; independent literal oracle checks its results.

    Only this probe's absolute pivot, Z rotation and final translation are used.
    This is not a decoder and does not establish native integer parity generally.
    """
    pivot,rotation,shift=[x[1] for x in row['transforms']]
    angle=(rotation[2]&255)*8
    sine=int(math.sin(angle*0.0030679615)*65536)
    cosine=int(math.cos(angle*0.0030679615)*65536)
    out=[]
    for vertex,label in zip(model['vertices'],model['vertex_skins']):
        if label!=1:
            out.append(list(vertex));continue
        x,y,z=[vertex[i]-pivot[i] for i in range(3)]
        q=[(sine*y+cosine*x)>>16,(cosine*y-sine*x)>>16,z]
        out.append([q[i]+pivot[i]+shift[i] for i in range(3)])
    return out

def observation(model, action):
    return dict(schema=1,units_per_tile=128,axes='blender=(x,z,-y)/128',
                model=model,action=action,
                preview=[native_preview(model,row) for row in action['frames']])

def _integer(value):
    if not math.isfinite(value) or abs(value-round(value))>1e-5:
        raise ValueError('nonintegral Blender native sample')
    return round(value)

def build_blender(output):
    import bpy  # Never imported by pure Python tests or --help.
    model,action=authored_input()
    # Factory-startup should be used; clear only this Blender process's scene.
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    mesh=bpy.data.meshes.new('Synthetic asymmetric connected mesh')
    mesh.from_pydata([(x/128,z/128,-y/128) for x,y,z in model['vertices']],[],model['faces'])
    mesh.update()
    obj=bpy.data.objects.new('Synthetic two-label mesh',mesh)
    bpy.context.collection.objects.link(obj)
    for color in [43051,8084]:
        material=bpy.data.materials.new(f'Native HSL {color}')
        material['native_hsl']=color
        mesh.materials.append(material)
    for polygon,color in zip(mesh.polygons,model['colors']):
        polygon.material_index=[43051,8084].index(color)
    for label in [0,1]:
        group=obj.vertex_groups.new(name=f'native_{label}')
        group.add([i for i,v in enumerate(model['vertex_skins']) if v==label],1.0,'REPLACE')
    obj['native_priority']=0
    rig=bpy.data.objects.new('Synthetic ordered native operations',None)
    bpy.context.collection.objects.link(rig)
    rig['contract']='absolute empty-label pivot; Z rotation; label1 translation; rigid labels only'
    for pose,row in enumerate(action['frames'],1):
        for label,values in row['transforms']:
            key=f'op_{label}'
            rig[key]=list(map(float,values))
            rig.keyframe_insert(data_path=f'["{key}"]',frame=pose)
    clip=rig.animation_data.action
    clip.name='Synthetic three-pose action'
    for layer in clip.layers:
        for strip in layer.strips:
            for bag in strip.channelbags:
                for curve in bag.fcurves:
                    for key in curve.keyframe_points:
                        key.interpolation='CONSTANT'
    # Export actual Blender mesh/group/material data and evaluated action values.
    model['vertices']=[[_integer(v.co.x*128),_integer(-v.co.z*128),_integer(v.co.y*128)] for v in mesh.vertices]
    model['faces']=[list(p.vertices) for p in mesh.polygons]
    model['colors']=[int(mesh.materials[p.material_index]['native_hsl']) for p in mesh.polygons]
    skins=[]
    for vertex in mesh.vertices:
        groups=[g for g in vertex.groups if g.weight==1.0]
        if len(groups)!=1:
            raise ValueError('exactly one rigid label required')
        skins.append(int(obj.vertex_groups[groups[0].group].name.split('_')[1]))
    model['vertex_skins']=skins
    for pose,row in enumerate(action['frames'],1):
        bpy.context.scene.frame_set(pose)
        evaluated=rig.evaluated_get(bpy.context.evaluated_depsgraph_get())
        row['transforms']=[[label,[_integer(x) for x in evaluated[f'op_{label}']]] for label in range(3)]
    result=observation(model,action)
    oracle.validate(result)
    bpy.ops.wm.save_as_mainfile(filepath=str(output/'synthetic.blend'))
    return result

def export_bundle(output, builder=build_blender):
    output=Path(output)
    output.mkdir(parents=True,exist_ok=False)
    def write(name,value):
        with (output/name).open('x',encoding='utf-8',newline='\n') as target:
            json.dump(value,target,sort_keys=True,indent=2,allow_nan=False)
            target.write('\n')
    # Persist independent declaration BEFORE any Blender creation/export.
    write('expectations.json',oracle.expected())
    result=builder(output)
    oracle.validate(result)
    write('observation.json',result)
    write('complete.json',dict(schema=1,status='numeric-only',namespace='unallocated',
                               cache_admitted=False,native_rendered=False))
    # Failure retains the partial directory; never overwrite or silently retry.
    return result

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args(argv)
    export_bundle(args.output)
    print('SYNTHETIC_SOURCE_EXPORT_COMPLETE: numeric only; cache/native acceptance pending')

if __name__=='__main__':
    main(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else None)
