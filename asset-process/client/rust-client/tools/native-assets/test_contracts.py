import copy
import unittest
import json
import struct
import tempfile
from pathlib import Path
from unittest.mock import patch
import convert_glb as legacy
import native_codec as extended
import mesh_validation as mesh


class InputContracts(unittest.TestCase):
    def setUp(self):
        self.g = dict(vertices=[[0,0,0],[128,0,0],[0,0,128]],faces=[[0,1,2]],colors=[43051])

    def test_legacy_rejects_missing_or_extra_colors(self):
        for colors in [[],[1,2]]:
            with self.assertRaises(ValueError):
                legacy.encode(self.g['vertices'],self.g['faces'],colors)

    def test_legacy_rejects_out_of_range_indices(self):
        for face in [[0,1,3],[-1,1,2],[0,1,1.0]]:
            with self.assertRaises(ValueError):
                legacy.encode(self.g['vertices'],[face],[43051])

    def test_native_rejects_ignored_texture_coordinates(self):
        g=copy.deepcopy(self.g);g['texture_coords']=[0]
        with self.assertRaises(ValueError):extended.model(g)

    def test_native_rejects_invalid_uniform_priority(self):
        for value in [-1,12,255]:
            g=copy.deepcopy(self.g);g['priority']=value
            with self.assertRaises(ValueError):extended.model(g)

    def test_owned_cleanup_preserves_vertex_identity(self):
        vertices=self.g['vertices'];faces=[[0,0,1],[0,1,2]]
        result,kept,report=mesh.clean_owned(vertices,faces)
        self.assertIs(result,vertices);self.assertEqual(kept,[[0,1,2]])
        self.assertEqual(report['removed_zero_area_faces'],1)

    def test_owned_cleanup_rejects_malformed_vertices(self):
        for row in [[0,0],[0,0,0,0]]:
            g=copy.deepcopy(self.g);g['vertices'][0]=row
            with self.assertRaises(ValueError):mesh.clean_owned(g['vertices'],g['faces'])

    def test_invalid_action_does_not_write_partial_files(self):
        action=dict(skeleton_id=123,frame_group=456,frames=[dict(types=[1],labels=[[0]],transforms=[[0,[1,2,3]]],length=0)])
        with tempfile.TemporaryDirectory() as folder:
            out=Path(folder)/'action'
            with self.assertRaises(ValueError):extended.classic(action,out)
            self.assertFalse(out.exists())


class GlbContracts(unittest.TestCase):
    def fixture(self,path,scale=None):
        vertices=[(0,0,0),(1,0,0),(0,1,0),(0,0,1)]
        faces=[(0,2,1),(0,1,3),(0,3,2),(1,2,3)]
        binary=b''.join(struct.pack('<3f',*v)for v in vertices)+b''.join(struct.pack('<3H',*f)for f in faces)
        node={'mesh':0}
        if scale is not None:node['scale']=scale
        doc=dict(asset={'version':'2.0'},buffers=[{'byteLength':len(binary)}],bufferViews=[{'buffer':0,'byteOffset':0,'byteLength':48},{'buffer':0,'byteOffset':48,'byteLength':24}],accessors=[{'bufferView':0,'componentType':5126,'count':4,'type':'VEC3'},{'bufferView':1,'componentType':5123,'count':12,'type':'SCALAR'}],meshes=[{'primitives':[{'attributes':{'POSITION':0},'indices':1,'material':0}]}],materials=[{'extras':{'native_render_type':0}}],nodes=[node],scenes=[{'nodes':[0]}])
        text=json.dumps(doc).encode();text+=b' '*(-len(text)%4)
        chunks=struct.pack('<II',len(text),0x4e4f534a)+text+struct.pack('<II',len(binary),0x004e4942)+binary
        path.write_bytes(struct.pack('<4sII',b'glTF',2,12+len(chunks))+chunks)

    def test_reflected_node_corrects_winding(self):
        with tempfile.TemporaryDirectory() as folder:
            p=Path(folder)/'owned.glb';self.fixture(p)
            vertices,faces,colors,shading=legacy.convert(p,with_shading=True)
            self.assertEqual(vertices,[(0,0,0),(128,0,0),(0,-128,0),(0,0,-128)])
            self.assertEqual(shading,[0]*4)
            self.fixture(p,[-1,1,1]);reflected=legacy.convert(p,with_shading=True)
            self.assertEqual(reflected[1],[(a,c,b)for a,b,c in faces])

    def test_cli_preserves_explicit_smooth_shading(self):
        with tempfile.TemporaryDirectory() as folder:
            p=Path(folder)/'owned.glb';out=Path(folder)/'result';self.fixture(p)
            with patch('sys.argv',['convert_glb.py',str(p),str(out)]):legacy.main()
            self.assertEqual((out/'archway.model').read_bytes(),legacy.encode(*legacy.convert(p,with_shading=True)))

if __name__=='__main__':unittest.main()
