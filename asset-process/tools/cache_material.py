"""Material suffixes and lossless topology export for real_to_blender meshes."""
import json
import re
from pathlib import Path

PATTERN = re.compile(r'hsl_(\d+)_(\d+)_(\d+)(_flat)?(?:_p(\d+))?(?:_d(-?\d+))?(?:_t(\d+))?(?:_a(\d+))?(?:\.\d+)?')

def parse(mat):
    match = PATTERN.fullmatch(mat.name) if mat else None
    if not match:
        raise ValueError(f'expected hsl_H_S_L material, got {mat and mat.name}')
    texture = int(match[7]) if match[7] else -1
    alpha = int(match[8] or 0)
    if texture > 210 or texture == 54 or alpha > 255:
        raise ValueError(f'invalid revision-239 material: {mat.name}')
    if int(match[1])>63 or int(match[2])>7 or int(match[3])>127:
        raise ValueError(f'invalid HSL: {mat.name}')
    return match, texture, alpha

def export_indexed(ob, deps, path, priority, vlabel, flabel, preserve=False):
    from author import native_codec, hsl
    ev = ob.evaluated_get(deps)
    me = ev.to_mesh(preserve_all_data_layers=True, depsgraph=deps)
    try:
        def values(name):
            return [v.value for v in me.attributes[name].data]
        g = dict(vertices=[], faces=[], colors=[], priorities=[], priority=priority)
        original = values('cache_vertex')
        if len(set(original)) != len(original):
            raise ValueError('duplicated mapping vertex ID after topology edit')
        indices = {old:i for i,old in enumerate(original)}
        for v in me.vertices:
            p = ev.matrix_world @ v.co
            coords=(p.x*128,-p.z*128,p.y*128)
            if any(abs(c-round(c))>0.0001 for c in coords):
                raise ValueError('fractional indexed vertex: export the raw model with its definition scale')
            g['vertices'].append([round(c) for c in coords])
        tex, alpha, depth, render = [],[],[],[]
        kinds = values('cache_render')
        for p in me.polygons:
            if len(p.vertices)!=3:
                raise ValueError('indexed import requires triangular faces')
            mt,t,a = parse(ev.material_slots[p.material_index].material)
            g['faces'].append(tuple(p.vertices))
            g['colors'].append(hsl(*map(int,mt.groups()[:3])))
            g['priorities'].append(int(mt[5]) if mt[5] else priority)
            tex.append(t); alpha.append(a); depth.append(int(mt[6] or 0)&255)
            render.append(1 if mt[4] else (0 if kinds[p.index]==1 else kinds[p.index]))
        if any(t>=0 for t in tex) or preserve or 'cache_types' in me:
            tri = [tuple(indices[i] for i in row) for row in json.loads(me['cache_basis'])]
            g.update(texture_coords=values('cache_coord'),texture_triangles=tri,
                     texture_types=json.loads(me['cache_types']) if 'cache_types' in me else [0]*len(tri))
            if any(t>=0 for t in tex):
                g['face_textures']=tex
        for key, data in [('alpha',alpha),('face_flags',depth),('render_types',render)]:
            if any(data): g[key]=data
        if me.get('cache_alpha_stream'):
            g['alpha']=alpha
        if not preserve and 'cache_priority' in me and len(set(g['priorities']))==1:
            g['priority']=g.pop('priorities')[0]
        for key, attr, override, n in [('vertex_skins','cache_vskin',vlabel,len(me.vertices)),
                                       ('face_skins','cache_fskin',flabel,len(me.polygons))]:
            if override is not None: g[key]=[override]*n
            elif attr in me.attributes: g[key]=values(attr)
        if preserve or 'cache_weights' in me or 'cache_types' in me:
            from model_packing import canonical_plan, encode
            weights=json.loads(me['cache_weights']) if 'cache_weights' in me else None
            if weights is not None:
                weights=[weights[i] for i in original]
            plan=json.loads(me['cache_packing']) if preserve else canonical_plan(g,weights)
            plan['bones']=weights
            data = encode(g,plan,json.loads(me.get('cache_complex','[]')))
        else:
            data = native_codec.model(g)
        Path(path).write_bytes(data)
        print(f'EXPORT {path}: {len(me.vertices)} indexed vertices, {len(me.polygons)} ordered faces')
    finally:
        ev.to_mesh_clear()
